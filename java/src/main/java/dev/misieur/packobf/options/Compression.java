package dev.misieur.packobf.options;

public enum Compression {
    SIMPLEST(0),
    NORMAL(1),
    MAX(2);

    Compression(int value) {
        this.value = value;
    }

    public final int value;
}
