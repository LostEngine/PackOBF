package dev.misieur.packobf.progress;

public final class ReadingZipProgress extends Progress {

    /**
     * The index of current file that PackOBF started to read
     */
    private final int current;
    /**
     * The total number of files to read.
     */
    private final int total;

    public ReadingZipProgress(int current, int total) {
        this.current = current;
        this.total = total;
    }

    public int current() {
        return current;
    }

    public int total() {
        return total;
    }

    @Override
    public State state() {
        return State.READING_ZIP;
    }
}
