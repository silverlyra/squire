use core::ffi::{c_int, c_void};

use super::{
    connection::sqlite3,
    types::{sqlite3_int64, sqlite3_uint64},
};

unsafe extern "C" {
    pub fn sqlite3_malloc(size: c_int) -> *mut c_void;
    pub fn sqlite3_malloc64(size: sqlite3_uint64) -> *mut c_void;
    pub fn sqlite3_realloc(ptr: *mut c_void, size: c_int) -> *mut c_void;
    pub fn sqlite3_realloc64(ptr: *mut c_void, size: sqlite3_uint64) -> *mut c_void;
    pub fn sqlite3_free(ptr: *mut c_void);
    pub fn sqlite3_msize(ptr: *mut c_void) -> sqlite3_uint64;
    pub fn sqlite3_memory_used() -> sqlite3_int64;
    pub fn sqlite3_memory_highwater(resetFlag: c_int) -> sqlite3_int64;
    pub fn sqlite3_release_memory(bytes: c_int) -> c_int;
    pub fn sqlite3_db_release_memory(db: *mut sqlite3) -> c_int;
    pub fn sqlite3_soft_heap_limit64(bytes: sqlite3_int64) -> sqlite3_int64;
    pub fn sqlite3_hard_heap_limit64(bytes: sqlite3_int64) -> sqlite3_int64;
}

/// Defines [memory allocation routines][] for SQLite.
///
/// [memory allocation routines]: https://sqlite.org/c3ref/mem_methods.html
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct sqlite3_mem_methods {
    pub xMalloc: Option<unsafe extern "C" fn(arg1: c_int) -> *mut c_void>,
    pub xFree: Option<unsafe extern "C" fn(arg1: *mut c_void)>,
    pub xRealloc: Option<unsafe extern "C" fn(arg1: *mut c_void, arg2: c_int) -> *mut c_void>,
    pub xSize: Option<unsafe extern "C" fn(arg1: *mut c_void) -> c_int>,
    pub xRoundup: Option<unsafe extern "C" fn(arg1: c_int) -> c_int>,
    pub xInit: Option<unsafe extern "C" fn(arg1: *mut c_void) -> c_int>,
    pub xShutdown: Option<unsafe extern "C" fn(arg1: *mut c_void)>,
    pub pAppData: *mut c_void,
}
