package dev.misieur.packobf.options;

public enum ShaderCompression {
    NONE(0),
    MINIFY(1),
    MINIFY_AND_OBFUSCATE(2);

    private final int value;

    ShaderCompression(int value) {
        this.value = value;
    }

    public int value() {
        return value;
    }
}
