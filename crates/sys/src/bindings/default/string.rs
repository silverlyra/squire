use core::ffi::{c_char, c_int};

use super::connection::sqlite3;

#[repr(C)]
pub struct sqlite3_str {
    _unused: [u8; 0],
}

unsafe extern "C" {
    pub fn sqlite3_str_new(pStr: *mut sqlite3) -> *mut sqlite3_str;
    pub fn sqlite3_str_finish(pStr: *mut sqlite3_str) -> *mut c_char;
    pub fn sqlite3_str_append(pStr: *mut sqlite3_str, zIn: *const c_char, N: c_int);
    pub fn sqlite3_str_appendall(pStr: *mut sqlite3_str, zIn: *const c_char);
    pub fn sqlite3_str_appendchar(pStr: *mut sqlite3_str, N: c_int, C: c_char);
    pub fn sqlite3_str_reset(pStr: *mut sqlite3_str);
    pub fn sqlite3_str_errcode(pStr: *mut sqlite3_str) -> c_int;
    pub fn sqlite3_str_length(pStr: *mut sqlite3_str) -> c_int;
    pub fn sqlite3_str_value(pStr: *mut sqlite3_str) -> *mut c_char;
}
