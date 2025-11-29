use core::ffi::{c_int, c_uchar, c_void};

use super::{statement::sqlite3_stmt, types::sqlite3_int64, value::sqlite3_value};

unsafe extern "C" {
    pub fn sqlite3_column_blob(pStmt: *mut sqlite3_stmt, iCol: c_int) -> *const c_void;
    pub fn sqlite3_column_double(pStmt: *mut sqlite3_stmt, iCol: c_int) -> f64;
    pub fn sqlite3_column_int(pStmt: *mut sqlite3_stmt, iCol: c_int) -> c_int;
    pub fn sqlite3_column_int64(pStmt: *mut sqlite3_stmt, iCol: c_int) -> sqlite3_int64;
    pub fn sqlite3_column_text(pStmt: *mut sqlite3_stmt, iCol: c_int) -> *const c_uchar;
    pub fn sqlite3_column_value(pStmt: *mut sqlite3_stmt, iCol: c_int) -> *mut sqlite3_value;
    pub fn sqlite3_column_bytes(pStmt: *mut sqlite3_stmt, iCol: c_int) -> c_int;
    pub fn sqlite3_column_type(pStmt: *mut sqlite3_stmt, iCol: c_int) -> c_int;
}
