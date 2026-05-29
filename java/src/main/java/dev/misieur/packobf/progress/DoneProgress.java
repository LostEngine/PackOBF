package dev.misieur.packobf.progress;

/**
 * No fields
 */
public final class DoneProgress extends Progress {

    @Override
    public State state() {
        return State.DONE;
    }
}
