use crate::version;

pub mod deserializers;

pub(crate) const fn default_one() -> f32 {
    1.0
}

pub(crate) fn is_one(f: &f32) -> bool {
    (*f - 1.0).abs() < f32::EPSILON
}

pub(crate) fn default_underscore() -> String {
    "_".to_string()
}

pub(crate) fn is_underscore(s: &String) -> bool {
    s == "_"
}

pub(crate) fn is_none_or_newer_than_26_1<T>(value: &Option<T>) -> bool {
    value.is_none() || version::is_newer_than_26_1(&())
}

pub(crate) fn is_none_or_older_than_1_21_11<T>(value: &Option<T>) -> bool {
    value.is_none() || version::is_older_than_1_21_11(&())
}

pub(crate) const fn default_one_u32() -> u32 {
    1
}

pub(crate) const fn default_16_u32() -> u32 {
    16
}

pub(crate) fn is_one_u32(f: &u32) -> bool {
    f == &1
}

pub(crate) fn is_16_u32(f: &u32) -> bool {
    f == &16
}


pub(crate) const fn is_zero(v: &i32) -> bool {
    *v == 0
}

pub(crate) const fn default_weight() -> i32 {
    1
}

pub(crate) const fn is_default_weight(v: &i32) -> bool {
    *v == 1
}

pub(crate) fn is_zero_or_older_than_1_21_11(v: &i32) -> bool {
    *v == 0 || version::is_older_than_1_21_11(&())
}

pub(crate) const fn default_true() -> bool {
    true
}
pub(crate) const fn is_true(b: &bool) -> bool {
    *b
}
pub(crate) fn default_ground() -> String {
    "ground".to_string()
}

pub(crate) fn default_ceiling_middle() -> String {
    "ceiling_middle".to_string()
}
pub(crate) fn default_single() -> String {
    "single".to_string()
}

pub(crate) fn is_false_or_older_than_1_21_6(value: &bool) -> bool {
    !*value || version::is_older_than_1_21_4(&())
}

pub(crate) fn is_one_or_older_than_1_21_11(f: &f32) -> bool {
    (*f - 1.0).abs() < f32::EPSILON || version::is_older_than_1_21_11(&())
}

pub(crate) fn is_none_or_older_than_26_1<T>(value: &Option<T>) -> bool {
    value.is_none() || version::is_older_than_26_1(&())
}
