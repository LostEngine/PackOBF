package dev.misieur.packobf.progress;

/**
 * No fields
 */
public final class IdleProgress extends Progress {

    @Override
    public State state() {
        return State.IDLE;
    }
}
