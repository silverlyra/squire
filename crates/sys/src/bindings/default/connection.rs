use core::ffi::{c_char, c_int};

/// A database [connection handle][].
///
/// [connection handle]: https://sqlite.org/c3ref/sqlite3.html
#[repr(C)]
pub struct sqlite3 {
    _unused: [u8; 0],
}

unsafe extern "C" {
    /// [Open][open] a [database connection][].
    ///
    /// [open]: https://sqlite.org/c3ref/open.html
    /// [database connection]: https://sqlite.org/c3ref/sqlite3.html
    pub fn sqlite3_open_v2(
        filename: *const c_char,
        ppDb: *mut *mut sqlite3,
        flags: c_int,
        zVfs: *const c_char,
    ) -> c_int;

    /// [Close][close] a [database connection][].
    ///
    /// [close]: https://sqlite.org/c3ref/close.html
    /// [database connection]: https://sqlite.org/c3ref/sqlite3.html
    pub fn sqlite3_close(pDb: *mut sqlite3) -> c_int;
}

pub const SQLITE_OPEN_READONLY: i32 = 0x00000001;
pub const SQLITE_OPEN_READWRITE: i32 = 0x00000002;
pub const SQLITE_OPEN_CREATE: i32 = 0x00000004;
pub const SQLITE_OPEN_DELETEONCLOSE: i32 = 0x00000008;
pub const SQLITE_OPEN_EXCLUSIVE: i32 = 0x00000010;
pub const SQLITE_OPEN_AUTOPROXY: i32 = 0x00000020;
pub const SQLITE_OPEN_URI: i32 = 0x00000040;
pub const SQLITE_OPEN_MEMORY: i32 = 0x00000080;
pub const SQLITE_OPEN_MAIN_DB: i32 = 0x00000100;
pub const SQLITE_OPEN_TEMP_DB: i32 = 0x00000200;
pub const SQLITE_OPEN_TRANSIENT_DB: i32 = 0x00000400;
pub const SQLITE_OPEN_MAIN_JOURNAL: i32 = 0x00000800;
pub const SQLITE_OPEN_TEMP_JOURNAL: i32 = 0x00001000;
pub const SQLITE_OPEN_SUBJOURNAL: i32 = 0x00002000;
pub const SQLITE_OPEN_SUPER_JOURNAL: i32 = 0x00004000;
pub const SQLITE_OPEN_NOMUTEX: i32 = 0x00008000;
pub const SQLITE_OPEN_FULLMUTEX: i32 = 0x00010000;
pub const SQLITE_OPEN_SHAREDCACHE: i32 = 0x00020000;
pub const SQLITE_OPEN_PRIVATECACHE: i32 = 0x00040000;
pub const SQLITE_OPEN_WAL: i32 = 0x00080000;
pub const SQLITE_OPEN_NOFOLLOW: i32 = 0x01000000;
pub const SQLITE_OPEN_EXRESCODE: i32 = 0x02000000;
