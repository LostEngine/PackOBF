package dev.misieur.packobf.progress;

public final class OptimizingProgress extends Progress {
    /**
     * The total number of files to optimize
     */
    private final int total;
    /**
     * The current file that PackOBF started to optimize
     */
    private final Current current;

    public OptimizingProgress(int total, Current current) {
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
        return State.OPTIMIZING;
    }

    public record Current(String name, int index) {
    }
}
