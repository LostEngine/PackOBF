package dev.misieur.packobf.options;

public enum ShaderCompression {
    NONE(0),
    MINIFY(1),
    MINIFY_AND_OBFUSCATE(2);

    ShaderCompression(int value) {
        this.value = value;
    }

    public final int value;
}
