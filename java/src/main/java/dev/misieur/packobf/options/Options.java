package dev.misieur.packobf.options;

import dev.misieur.packobf.annotations.Nullable;

public record Options(Compression compression, ShaderCompression shaderCompression, boolean renameFiles, boolean blockUnzipping, boolean corruptPngFiles, @Nullable Integer numThreads) {
    public static Options simplest() {
        return new Options(
                Compression.SIMPLEST,
                ShaderCompression.NONE,
                false,
                false,
                false
        );
    }

    public static Options normal() {
        return new Options(
                Compression.NORMAL,
                ShaderCompression.NONE,
                false,
                false,
                false
        );
    }


    public static Options max() {
        return new Options(
                Compression.MAX,
                ShaderCompression.MINIFY_AND_OBFUSCATE,
                true,
                true,
                true
        );
    }
}
