package dev.misieur.packobf.progress;

/**
 * No fields
 */
public final class FinishingProgress extends Progress {
    public FinishingProgress() {
    }

    @Override
    public State state() {
        return State.FINISHING;
    }

    public record Current(String name, int index) {
    }
}
