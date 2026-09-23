package dev.misieur.packobf.options;

public enum Compression {
    FAST(0),
    NORMAL(1);

    private final int value;

    Compression(int value) {
        this.value = value;
    }

    public int value() {
        return value;
    }
}
