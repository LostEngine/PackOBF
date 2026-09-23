package dev.misieur.packobf.progress;

public abstract sealed class Progress permits IdleProgress, ReadingZipProgress, ParsingProgress, OptimizingProgress, FinishingProgress, DoneProgress {
    private State state;

    public abstract State state();

    public enum State {
        IDLE(0),
        READING_ZIP(1),
        PARSING(2),
        OPTIMIZING(3),
        FINISHING(4),
        DONE(5);

        private final int value;

        public int value() {
            return value;
        }

        State(int value) {
            this.value = value;
        }
    }
}
