package dev.misieur.packobf.options;

import dev.misieur.packobf.annotations.NotNull;

import java.util.Optional;

public record Options(@NotNull Compression compression,
                      @NotNull ShaderCompression shaderCompression,
                      boolean renameFiles,
                      boolean blockUnzipping,
                      boolean corruptPngFiles,
                      @NotNull Optional<Integer> numThreads,
                      @NotNull Optional<MinecraftVersion> targetVersion
) {
    public static Options fastest() {
        return new Options(
                Compression.FASTEST,
                ShaderCompression.NONE,
                false,
                false,
                false,
                Optional.empty(),
                Optional.empty()
        );
    }

    public static Options fast() {
        return new Options(
                Compression.FAST,
                ShaderCompression.NONE,
                false,
                false,
                false,
                Optional.empty(),
                Optional.empty()
        );
    }

    public static Options normal() {
        return new Options(
                Compression.NORMAL,
                ShaderCompression.NONE,
                false,
                false,
                false,
                Optional.empty(),
                Optional.empty()
        );
    }


    public static Options best() {
        return new Options(
                Compression.BEST,
                ShaderCompression.NONE,
                true,
                true,
                true,
                Optional.empty(),
                Optional.empty()
        );
    }


    public static Options ultra() {
        return new Options(
                Compression.ULTRA,
                ShaderCompression.MINIFY_AND_OBFUSCATE,
                true,
                true,
                true,
                Optional.empty(),
                Optional.empty()
        );
    }
}
