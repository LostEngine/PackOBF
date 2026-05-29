package dev.misieur.packobf.progress;

public final class BuildingProgress extends Progress {
    /**
     * The total number of files to write
     */
    private final int total;
    /**
     * The current file that PackOBF started to write
     */
    private final Current current;

    public BuildingProgress(int total, Current current) {
        this.total = total;
        this.current = current;
    }

    public int total() {
        return total;
    }

    public Current current() {
        return current;
    }

    @Override
    public State state() {
        return State.BUILDING;
    }

    public record Current(String name, int index) {
    }
}
