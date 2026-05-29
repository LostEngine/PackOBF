package dev.misieur.packobf;

import dev.misieur.packobf.annotations.NotNull;
import dev.misieur.packobf.annotations.Nullable;
import dev.misieur.packobf.log.LogCallback;
import dev.misieur.packobf.log.LogLevel;
import dev.misieur.packobf.options.Options;
import dev.misieur.packobf.progress.*;

import java.io.IOException;
import java.nio.file.Path;

/**
 * A safe way to use PackOBF through Java
 */
public class PackOBF {

    /**
     * Optimizes a resource pack using the Rust PackOBF library through Java
     *
     * @param input The bytes of your Minecraft: Java Edition resource pack (must be a conventional ZIP)
     * @param options The options that PackOBF will use to optimize the resource pack
     * @param logCallback A function called by the Rust library to print logs
     * @param progressCallback Add function called by the Rust library to give the current progress
     * @param cacheFile The path to a file that PackOBF will use to cache compressed files (`.bin` extension is recommended)
     *
     * @return The bytes of the optimized resource pack as ZIP file (may not respect ZIP conventions)
     * @throws IOException If the library had an important error that forced it to stop precessing the resource pack.
     */
    public static byte[] optimizeZip(
            @NotNull byte[] input,
            @NotNull Options options,
            @NotNull LogCallback logCallback,
            @NotNull ProgressCallback progressCallback,
            @Nullable Path cacheFile
    ) throws IOException {
        Native.load();
        return Native.optimizeZip(
                input,
                new Native.Options(
                        options.compression().value,
                        options.shaderCompression().value,
                        options.renameFiles(),
                        options.blockUnzipping(),
                        options.corruptPngFiles()
                ),
                (level, message) -> logCallback.onLog(switch (level) {
                    case 0 -> LogLevel.INFO;
                    case 1 -> LogLevel.WARNING;
                    default -> LogLevel.ERROR;
                }, message),
                (state, current, total, currentString) -> {
                    switch (state) {
                        case 0 -> progressCallback.onProgress(new IdleProgress());
                        case 1 -> progressCallback.onProgress(new ReadingZipProgress(current, total));
                        case 2 -> progressCallback.onProgress(new ParsingProgress(currentString));
                        case 3 ->
                                progressCallback.onProgress(new BuildingProgress(total, new BuildingProgress.Current(currentString, current)));
                        case 4 -> progressCallback.onProgress(new DoneProgress());
                        default -> {
                        }
                    }
                },
                cacheFile != null ? cacheFile.toAbsolutePath().toString() : null
        );
    }

}
