use core::ffi::{c_char, c_int, c_uchar, c_uint, c_void};

use super::types::sqlite3_int64;

/// A dynamically-typed [value object].
///
/// [value object]: https://sqlite.org/c3ref/value.html
#[repr(C)]
pub struct sqlite3_value {
    _unused: [u8; 0],
}

unsafe extern "C" {
    pub fn sqlite3_value_blob(value: *mut sqlite3_value) -> *const c_void;
    pub fn sqlite3_value_double(value: *mut sqlite3_value) -> f64;
    pub fn sqlite3_value_int(value: *mut sqlite3_value) -> c_int;
    pub fn sqlite3_value_int64(value: *mut sqlite3_value) -> sqlite3_int64;
    pub fn sqlite3_value_pointer(value: *mut sqlite3_value, arg2: *const c_char) -> *mut c_void;
    pub fn sqlite3_value_text(value: *mut sqlite3_value) -> *const c_uchar;
    pub fn sqlite3_value_bytes(value: *mut sqlite3_value) -> c_int;
    pub fn sqlite3_value_type(value: *mut sqlite3_value) -> c_int;
    pub fn sqlite3_value_numeric_type(value: *mut sqlite3_value) -> c_int;
    pub fn sqlite3_value_nochange(value: *mut sqlite3_value) -> c_int;
    pub fn sqlite3_value_frombind(value: *mut sqlite3_value) -> c_int;
    pub fn sqlite3_value_encoding(value: *mut sqlite3_value) -> c_int;
    pub fn sqlite3_value_subtype(value: *mut sqlite3_value) -> c_uint;
    pub fn sqlite3_value_dup(value: *const sqlite3_value) -> *mut sqlite3_value;
    pub fn sqlite3_value_free(value: *mut sqlite3_value);
}
