package dev.misieur.packobf.progress;

public abstract sealed class Progress permits BuildingProgress, DoneProgress, IdleProgress, ParsingProgress, ReadingZipProgress {
    private State state;

    public abstract State state();

    public enum State {
        IDLE(0),
        READING_ZIP(1),
        PARSING(2),
        BUILDING(3),
        DONE(4);

        private final int value;

        public int value() {
            return value;
        }

        State(int value) {
            this.value = value;
        }
    }
}
