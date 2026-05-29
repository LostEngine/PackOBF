package dev.misieur.packobf.progress;

public final class ParsingProgress extends Progress {

    /**
     * The current file that PackOBF started to parse
     */
    private final String current;

    public ParsingProgress(String current) {
        this.current = current;
    }

    public String current() {
        return current;
    }

    @Override
    public State state() {
        return State.PARSING;
    }
}
