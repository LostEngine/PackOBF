use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::LazyLock;
use clap::ValueEnum;

#[repr(u8)]
#[derive(ValueEnum, Clone, Debug, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MinecraftVersion {
    V1_21_1 = 34,
    V1_21_2 = 42,
    V1_21_4 = 46,
    V1_21_5 = 55,
    V1_21_6 = 63,
    V1_21_7 = 64,
    V1_21_9 = 69,
    V1_21_11 = 75,
    V26_1 = 84,
    V26_2 = 88,
}

pub static TARGET_VERSION: LazyLock<AtomicU8> =
    LazyLock::new(|| AtomicU8::new(0));

pub fn set_target_version(version: u8) {
    TARGET_VERSION.store(version, Ordering::Relaxed);
}

pub fn get_version() -> u8 {
    TARGET_VERSION.load(Ordering::Relaxed)
}

macro_rules! version_check {
    ($name:ident, $version:ident, <) => {
        pub fn $name<T>(_: T) -> bool {
            // zero check required as if no version is set, it will be 0, which will always be less than any version
            let version = get_version();
            version != 0 && version < MinecraftVersion::$version as u8
        }
    };
    ($name:ident, $version:ident, >) => {
        pub fn $name<T>(_: T) -> bool {
            get_version() > MinecraftVersion::$version as u8 // zero check isn't needed in this case
        }
    };
    ($name:ident, $version1:ident, $version2:ident, <>) => {
        pub fn $name<T>(_: T) -> bool {
            let version = get_version();
            version != 0
            && version < MinecraftVersion::$version1 as u8
            && version > MinecraftVersion::$version2 as u8
        }
    };
}

version_check!(is_older_than_1_21_1, V1_21_1, <);
version_check!(is_older_than_1_21_2, V1_21_2, <);
version_check!(is_older_than_1_21_4, V1_21_4, <);
version_check!(is_older_than_1_21_5, V1_21_5, <);
version_check!(is_older_than_1_21_6, V1_21_6, <);
version_check!(is_older_than_1_21_7, V1_21_7, <);
version_check!(is_older_than_1_21_9, V1_21_9, <);
version_check!(is_older_than_1_21_11, V1_21_11, <);
version_check!(is_older_than_26_1, V26_1, <);
version_check!(is_older_than_26_2, V26_2, <);

version_check!(is_newer_than_1_21_1, V1_21_1, >);
version_check!(is_newer_than_1_21_2, V1_21_2, >);
version_check!(is_newer_than_1_21_4, V1_21_4, >);
version_check!(is_newer_than_1_21_5, V1_21_5, >);
version_check!(is_newer_than_1_21_6, V1_21_6, >);
version_check!(is_newer_than_1_21_7, V1_21_7, >);
version_check!(is_newer_than_1_21_9, V1_21_9, >);
version_check!(is_newer_than_1_21_11, V1_21_11, >);
version_check!(is_newer_than_26_1, V26_1, >);
version_check!(is_newer_than_26_2, V26_2, >);

version_check!(is_not_between_26_1_and_26_2, V26_1, V26_2, <>);
