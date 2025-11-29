use core::ffi::{c_int, c_void};

use super::types::{sqlite3_int64, sqlite3_uint64};

unsafe extern "C" {
    pub fn sqlite3_malloc(size: c_int) -> *mut c_void;
    pub fn sqlite3_malloc64(size: sqlite3_uint64) -> *mut c_void;
    pub fn sqlite3_realloc(ptr: *mut c_void, size: c_int) -> *mut c_void;
    pub fn sqlite3_realloc64(ptr: *mut c_void, size: sqlite3_uint64) -> *mut c_void;
    pub fn sqlite3_free(ptr: *mut c_void);
    pub fn sqlite3_msize(ptr: *mut c_void) -> sqlite3_uint64;
    pub fn sqlite3_memory_used() -> sqlite3_int64;
    pub fn sqlite3_memory_highwater(resetFlag: c_int) -> sqlite3_int64;
}
