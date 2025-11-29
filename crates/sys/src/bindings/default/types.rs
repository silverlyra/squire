use core::ffi::{c_longlong, c_ulonglong};

pub type sqlite_int64 = c_longlong;
pub type sqlite_uint64 = c_ulonglong;
pub type sqlite3_int64 = sqlite_int64;
pub type sqlite3_uint64 = sqlite_uint64;

pub const SQLITE_INTEGER: i32 = 1;
pub const SQLITE_FLOAT: i32 = 2;
pub const SQLITE_BLOB: i32 = 4;
pub const SQLITE_NULL: i32 = 5;
pub const SQLITE_TEXT: i32 = 3;
pub const SQLITE_UTF8: i32 = 1;
pub const SQLITE_UTF16LE: i32 = 2;
pub const SQLITE_UTF16BE: i32 = 3;
pub const SQLITE_UTF16: i32 = 4;
pub const SQLITE_ANY: i32 = 5;
