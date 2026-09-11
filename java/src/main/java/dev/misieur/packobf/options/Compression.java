package dev.misieur.packobf.options;

public enum Compression {
    FASTEST(0),
    FAST(1),
    NORMAL(2),
    BEST(3),
    ULTRA(4);

    private final int value;

    Compression(int value) {
        this.value = value;
    }

    public int value() {
        return value;
    }
}
