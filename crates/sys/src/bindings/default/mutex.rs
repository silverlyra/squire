use core::ffi::c_int;

use super::connection::sqlite3;

#[repr(C)]
pub struct sqlite3_mutex {
    _unused: [u8; 0],
}

pub const SQLITE_MUTEX_FAST: i32 = 0;
pub const SQLITE_MUTEX_RECURSIVE: i32 = 1;
pub const SQLITE_MUTEX_STATIC_MAIN: i32 = 2;
pub const SQLITE_MUTEX_STATIC_MEM: i32 = 3;
pub const SQLITE_MUTEX_STATIC_MEM2: i32 = 4;
pub const SQLITE_MUTEX_STATIC_OPEN: i32 = 4;
pub const SQLITE_MUTEX_STATIC_PRNG: i32 = 5;
pub const SQLITE_MUTEX_STATIC_LRU: i32 = 6;
pub const SQLITE_MUTEX_STATIC_LRU2: i32 = 7;
pub const SQLITE_MUTEX_STATIC_PMEM: i32 = 7;
pub const SQLITE_MUTEX_STATIC_APP1: i32 = 8;
pub const SQLITE_MUTEX_STATIC_APP2: i32 = 9;
pub const SQLITE_MUTEX_STATIC_APP3: i32 = 10;
pub const SQLITE_MUTEX_STATIC_VFS1: i32 = 11;
pub const SQLITE_MUTEX_STATIC_VFS2: i32 = 12;
pub const SQLITE_MUTEX_STATIC_VFS3: i32 = 13;
pub const SQLITE_MUTEX_STATIC_MASTER: i32 = 2;

unsafe extern "C" {
    pub fn sqlite3_mutex_alloc(iMode: c_int) -> *mut sqlite3_mutex;
    pub fn sqlite3_mutex_free(pMutex: *mut sqlite3_mutex);
    pub fn sqlite3_mutex_enter(pMutex: *mut sqlite3_mutex);
    pub fn sqlite3_mutex_try(pMutex: *mut sqlite3_mutex) -> c_int;
    pub fn sqlite3_mutex_leave(pMutex: *mut sqlite3_mutex);
    pub fn sqlite3_db_mutex(pDb: *mut sqlite3) -> *mut sqlite3_mutex;
}
