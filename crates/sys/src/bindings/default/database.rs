use core::ffi::{c_char, c_int};

use super::connection::sqlite3;

unsafe extern "C" {
    pub fn sqlite3_db_name(db: *mut sqlite3, n: c_int) -> *const c_char;
    pub fn sqlite3_db_readonly(db: *mut sqlite3, db_name: *const c_char) -> *const c_char;
    pub fn sqlite3_txn_state(db: *mut sqlite3, db_name: *const c_char) -> *const c_char;
}

pub const SQLITE_TXN_NONE: i32 = 0;
pub const SQLITE_TXN_READ: i32 = 1;
pub const SQLITE_TXN_WRITE: i32 = 2;
