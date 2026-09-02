package dev.misieur.packobf.options;

public enum MinecraftVersion {
    V1_21_1(34),
    V1_21_2(42),
    V1_21_4(46),
    V1_21_5(55),
    V1_21_6(63),
    V1_21_7(64),
    V1_21_9(69),
    V1_21_11(75),
    V26_1(84),
    V26_2(88);

    private final int id;

    MinecraftVersion(int id) {
        this.id = id;
    }

    public int id() {
        return id;
    }
}
