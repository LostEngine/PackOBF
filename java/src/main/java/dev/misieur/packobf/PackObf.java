package dev.misieur.packobf;

import java.io.IOException;
import java.lang.annotation.ElementType;
import java.lang.annotation.Retention;
import java.lang.annotation.RetentionPolicy;
import java.lang.annotation.Target;
import java.nio.file.*;
import java.nio.file.attribute.BasicFileAttributes;
import java.util.Locale;

public class PackObf {

    private PackObf() {
    }

    public static class Options {

        public Options(@Compression int compression, @ShaderCompression int shaderCompression, boolean renameFiles, boolean blockUnzipping, boolean corruptPngFiles) {
            this.compression = compression;
            this.shaderCompression = shaderCompression;
            this.renameFiles = renameFiles;
            this.blockUnzipping = blockUnzipping;
            this.corruptPngFiles = corruptPngFiles;
        }

        public @Compression int compression = 1;
        public @ShaderCompression int shaderCompression = 0;
        public boolean renameFiles = false;
        public boolean blockUnzipping = false;
        public boolean corruptPngFiles = false;

        public static final @Compression int SIMPLEST = 0;
        public static final @Compression int NORMAL = 1;
        public static final @Compression int MAX = 2;

        @Retention(RetentionPolicy.CLASS)
        @Target({ElementType.FIELD, ElementType.PARAMETER, ElementType.LOCAL_VARIABLE, ElementType.METHOD, ElementType.TYPE_USE})
        @interface Compression {
        }

        public static final @ShaderCompression int NONE = 0;
        public static final @ShaderCompression int MINIFY = 1;
        public static final @ShaderCompression int MINIFY_AND_OBFUSCATE = 2;

        @Retention(RetentionPolicy.CLASS)
        @Target({ElementType.FIELD, ElementType.PARAMETER, ElementType.LOCAL_VARIABLE, ElementType.METHOD, ElementType.TYPE_USE})
        @interface ShaderCompression {
        }
    }

    public interface LogCallback {
        void onLog(@Level int level, String message);

        public static final @Level int INFO = 0;
        public static final @Level int WARNING = 1;
        public static final @Level int ERROR = 2;

        @Retention(RetentionPolicy.CLASS)
        @Target({ElementType.FIELD, ElementType.PARAMETER, ElementType.LOCAL_VARIABLE, ElementType.METHOD, ElementType.TYPE_USE})
        @interface Level {
        }
    }

    public interface ProgressCallback {
        void onProgress(@State int state, int current, int total, @Nullable String currentString);

        /**
         * nothing
         */
        public static final @State int IDLE = 0;
        /**
         * {@code current} and {@code total}
         */
        public static final @State int READING_ZIP = 1;
        /**
         * {@code currentString}
         */
        public static final @State int PARSING = 2;
        /**
         * {@code current}, {@code currentString} and {@code total}
         */
        public static final @State int BUILDING = 3;
        /**
         * nothing
         */
        public static final @State int DONE = 4;

        @Retention(RetentionPolicy.CLASS)
        @Target({ElementType.FIELD, ElementType.PARAMETER, ElementType.LOCAL_VARIABLE, ElementType.METHOD, ElementType.TYPE_USE})
        @interface State {
        }
    }

    private static volatile boolean enabled = false;

    public static native byte[] optimizeZip(
            byte[] input,
            Options options,
            LogCallback logCallback,
            ProgressCallback progressCallback,
            String cacheFile
    ) throws IOException;

    public static void load() throws IOException {
        if (enabled) return;
        String libName = switch (OS.forName(System.getProperty("os.name"))) {
            case WINDOWS -> "rust.dll";
            case LINUX -> "librust.so";
            case MAC_OS -> "librust.dylib";
        };
        String resourcePath = "/packobf-natives/" + libName;
        Path tempDir = Files.createTempDirectory("packobf-native-");
        Path extracted = Utils.extractFile(resourcePath, tempDir);
        extracted.toFile().setReadable(true);
        extracted.toFile().setExecutable(true);

        System.load(extracted.toAbsolutePath().toString());
        registerCleanup(tempDir);
        enabled = true;
    }

    public static boolean enabled() {
        return enabled;
    }

    private static void registerCleanup(Path tempDir) {
        Runtime.getRuntime().addShutdownHook(Thread.ofPlatform().name("packobf-native-lib-cleanup").unstarted(() -> {
            try {
                Files.walkFileTree(tempDir, new SimpleFileVisitor<>() {
                    @Override
                    public FileVisitResult visitFile(Path file, BasicFileAttributes attrs)
                            throws IOException {
                        Files.deleteIfExists(file);
                        return FileVisitResult.CONTINUE;
                    }

                    @Override
                    public FileVisitResult postVisitDirectory(Path dir, IOException exc)
                            throws IOException {
                        Files.deleteIfExists(dir);
                        return FileVisitResult.CONTINUE;
                    }
                });
            } catch (IOException ignored) {
            }
        }));
    }

    enum OS {
        WINDOWS, LINUX, MAC_OS;

        public static OS forName(String os) {
            String osName = os.toLowerCase(Locale.ROOT);
            if (osName.contains("windows")) return WINDOWS;
            if (osName.contains("linux")) return LINUX;
            if (osName.contains("mac") || osName.contains("darwin") || osName.contains("osx")) return MAC_OS;
            throw new IllegalStateException("Unsupported OS: " + os);
        }
    }

}
