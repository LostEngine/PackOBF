package dev.misieur.packobf.options;

import dev.misieur.packobf.annotations.NotNull;

import java.util.Optional;

public record Options(@NotNull Compression compression,
                      @NotNull ShaderCompression shaderCompression,
                      @NotNull SoundQuality soundQuality,
                      boolean renameFiles,
                      boolean blockUnzipping,
                      boolean corruptPngFiles,
                      @NotNull Optional<Integer> numThreads,
                      @NotNull Optional<MinecraftVersion> targetVersion
) {

    public static Options fast() {
        return new Options(
                Compression.FAST,
                ShaderCompression.NONE,
                SoundQuality.DYNAMIC,
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
                SoundQuality.DYNAMIC,
                false,
                false,
                false,
                Optional.empty(),
                Optional.empty()
        );
    }


    public static Options best() {
        return new Options(
                Compression.NORMAL,
                ShaderCompression.NONE,
                SoundQuality.DYNAMIC,
                true,
                true,
                true,
                Optional.empty(),
                Optional.empty()
        );
    }
}
