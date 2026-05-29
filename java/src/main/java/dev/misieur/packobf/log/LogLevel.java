package dev.misieur.packobf.log;

public enum LogLevel {
    INFO(0),
    WARNING(1),
    ERROR(2);

    private final int value;

    public int value() {
        return this.value;
    }

    LogLevel(int value) {
        this.value = value;
    }
}
