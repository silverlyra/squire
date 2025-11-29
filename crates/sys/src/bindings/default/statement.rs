use core::ffi::{c_char, c_int, c_uint};

use super::{connection::sqlite3, types::sqlite3_int64};

/// A [prepared statement][].
///
/// [prepared statement]: https://sqlite.org/c3ref/stmt.html
#[repr(C)]
pub struct sqlite3_stmt {
    _unused: [u8; 0],
}

unsafe extern "C" {
    /// [Prepare][prepare] a SQL [statement][].
    ///
    /// [prepare]: https://sqlite.org/c3ref/prepare.html
    /// [statement]: https://sqlite.org/c3ref/stmt.html
    pub fn sqlite3_prepare_v3(
        db: *mut sqlite3,
        zSql: *const c_char,
        nByte: c_int,
        prepFlags: c_uint,
        ppStmt: *mut *mut sqlite3_stmt,
        pzTail: *mut *const c_char,
    ) -> c_int;

    pub fn sqlite3_step(pStmt: *mut sqlite3_stmt) -> c_int;

    pub fn sqlite3_bind_parameter_count(pStmt: *mut sqlite3_stmt) -> c_int;
    pub fn sqlite3_bind_parameter_name(pStmt: *mut sqlite3_stmt, arg2: c_int) -> *const c_char;
    pub fn sqlite3_bind_parameter_index(pStmt: *mut sqlite3_stmt, zName: *const c_char) -> c_int;

    pub fn sqlite3_column_count(pStmt: *mut sqlite3_stmt) -> c_int;
    pub fn sqlite3_column_name(pStmt: *mut sqlite3_stmt, n: c_int) -> *const c_char;
    pub fn sqlite3_column_database_name(pStmt: *mut sqlite3_stmt, n: c_int) -> *const c_char;
    pub fn sqlite3_column_table_name(pStmt: *mut sqlite3_stmt, n: c_int) -> *const c_char;
    pub fn sqlite3_column_origin_name(pStmt: *mut sqlite3_stmt, n: c_int) -> *const c_char;
    pub fn sqlite3_column_decltype(pStmt: *mut sqlite3_stmt, n: c_int) -> *const c_char;
    pub fn sqlite3_data_count(pStmt: *mut sqlite3_stmt) -> c_int;

    pub fn sqlite3_db_handle(pStmt: *mut sqlite3_stmt) -> *mut sqlite3;

    pub fn sqlite3_changes(pStmt: *mut sqlite3) -> c_int;
    pub fn sqlite3_changes64(pStmt: *mut sqlite3) -> sqlite3_int64;
    pub fn sqlite3_last_insert_rowid(pStmt: *mut sqlite3) -> sqlite3_int64;
    pub fn sqlite3_set_last_insert_rowid(pStmt: *mut sqlite3, id: sqlite3_int64);

    pub fn sqlite3_clear_bindings(pStmt: *mut sqlite3_stmt) -> c_int;
    pub fn sqlite3_reset(pStmt: *mut sqlite3_stmt) -> c_int;
    pub fn sqlite3_finalize(pStmt: *mut sqlite3_stmt) -> c_int;
}

pub const SQLITE_PREPARE_PERSISTENT: i32 = 1;
pub const SQLITE_PREPARE_NORMALIZE: i32 = 2;
pub const SQLITE_PREPARE_NO_VTAB: i32 = 4;
pub const SQLITE_PREPARE_DONT_LOG: i32 = 16;
