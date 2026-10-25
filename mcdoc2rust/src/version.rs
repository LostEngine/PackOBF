macro_rules! version_check {
    ($name:ident, $version:ident, <) => {
        #[allow(dead_code)]
        pub fn $name<T>(_: T) -> bool {
            false
        }
    };
    ($name:ident, $version:ident, >) => {
        #[allow(dead_code)]
        pub fn $name<T>(_: T) -> bool {
            false
        }
    };
    ($name:ident, $version1:ident, $version2:ident, <>) => {
        #[allow(dead_code)]
        pub fn $name<T>(_: T) -> bool {
            false
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
version_check!(is_older_than_26_3, V26_3, <);

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
version_check!(is_newer_than_26_3, V26_3, >);
