use core::ffi::{c_char, c_int, c_uchar, c_void};

use super::{
    sqlite3_destructor_type,
    statement::sqlite3_stmt,
    types::{sqlite3_int64, sqlite3_uint64},
    value::sqlite3_value,
};

unsafe extern "C" {
    pub fn sqlite3_bind_blob(
        pStmt: *mut sqlite3_stmt,
        parameter: c_int,
        value: *const c_void,
        len: c_int,
        destructor: sqlite3_destructor_type,
    ) -> c_int;
    pub fn sqlite3_bind_blob64(
        pStmt: *mut sqlite3_stmt,
        parameter: c_int,
        value: *const c_void,
        len: sqlite3_uint64,
        destructor: sqlite3_destructor_type,
    ) -> c_int;
    pub fn sqlite3_bind_double(pStmt: *mut sqlite3_stmt, parameter: c_int, arg3: f64) -> c_int;
    pub fn sqlite3_bind_int(pStmt: *mut sqlite3_stmt, parameter: c_int, arg3: c_int) -> c_int;
    pub fn sqlite3_bind_int64(
        pStmt: *mut sqlite3_stmt,
        parameter: c_int,
        arg3: sqlite3_int64,
    ) -> c_int;
    pub fn sqlite3_bind_null(pStmt: *mut sqlite3_stmt, parameter: c_int) -> c_int;
    pub fn sqlite3_bind_text(
        pStmt: *mut sqlite3_stmt,
        parameter: c_int,
        value: *const c_char,
        len: c_int,
        destructor: sqlite3_destructor_type,
    ) -> c_int;
    pub fn sqlite3_bind_text64(
        pStmt: *mut sqlite3_stmt,
        parameter: c_int,
        value: *const c_char,
        len: sqlite3_uint64,
        destructor: sqlite3_destructor_type,
        encoding: c_uchar,
    ) -> c_int;
    pub fn sqlite3_bind_value(
        pStmt: *mut sqlite3_stmt,
        parameter: c_int,
        value: *const sqlite3_value,
    ) -> c_int;
    pub fn sqlite3_bind_pointer(
        pStmt: *mut sqlite3_stmt,
        parameter: c_int,
        value: *mut c_void,
        type_name: *const c_char,
        destructor: sqlite3_destructor_type,
    ) -> c_int;
    pub fn sqlite3_bind_zeroblob(pStmt: *mut sqlite3_stmt, parameter: c_int, n: c_int) -> c_int;
    pub fn sqlite3_bind_zeroblob64(
        pStmt: *mut sqlite3_stmt,
        parameter: c_int,
        arg3: sqlite3_uint64,
    ) -> c_int;
}
