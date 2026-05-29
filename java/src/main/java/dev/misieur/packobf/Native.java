package dev.misieur.packobf;

import dev.misieur.packobf.annotations.Nullable;

import java.io.IOException;
import java.nio.file.*;
import java.nio.file.attribute.BasicFileAttributes;
import java.util.Locale;

public class Native {

    private Native() {
    }

    static class Options {

        public Options(int compression, int shaderCompression, boolean renameFiles, boolean blockUnzipping, boolean corruptPngFiles) {
            this.compression = compression;
            this.shaderCompression = shaderCompression;
            this.renameFiles = renameFiles;
            this.blockUnzipping = blockUnzipping;
            this.corruptPngFiles = corruptPngFiles;
        }

        public int compression;
        public int shaderCompression;
        public boolean renameFiles;
        public boolean blockUnzipping;
        public boolean corruptPngFiles;
    }

    interface LogCallback {
        void onLog(int level, String message);
    }

    interface ProgressCallback {
        void onProgress(int state, int current, int total, @Nullable String currentString);
    }

    private static volatile boolean enabled = false;

    static native byte[] optimizeZip(
            byte[] input,
            Options options,
            LogCallback logCallback,
            ProgressCallback progressCallback,
            String cacheFile
    ) throws IOException;

    static void load() throws IOException {
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
