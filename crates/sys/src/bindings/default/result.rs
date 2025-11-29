use core::ffi::{c_char, c_int};

use super::connection::sqlite3;

/// Successful result
pub const SQLITE_OK: i32 = 0;
/// Generic error
pub const SQLITE_ERROR: i32 = 1;
/// Internal logic error in SQLite
pub const SQLITE_INTERNAL: i32 = 2;
/// Access permission denied
pub const SQLITE_PERM: i32 = 3;
/// Callback routine requested an abort
pub const SQLITE_ABORT: i32 = 4;
/// The database file is locked
pub const SQLITE_BUSY: i32 = 5;
/// A table in the database is locked
pub const SQLITE_LOCKED: i32 = 6;
/// A malloc() failed
pub const SQLITE_NOMEM: i32 = 7;
/// Attempt to write a readonly database
pub const SQLITE_READONLY: i32 = 8;
/// Operation terminated by sqlite3_interrupt()
pub const SQLITE_INTERRUPT: i32 = 9;
/// Some kind of disk I/O error occurred
pub const SQLITE_IOERR: i32 = 10;
/// The database disk image is malformed
pub const SQLITE_CORRUPT: i32 = 11;
/// Unknown opcode in sqlite3_file_control()
pub const SQLITE_NOTFOUND: i32 = 12;
/// Insertion failed because database is full
pub const SQLITE_FULL: i32 = 13;
/// Unable to open the database file
pub const SQLITE_CANTOPEN: i32 = 14;
/// Database lock protocol error
pub const SQLITE_PROTOCOL: i32 = 15;
/// Internal use only
pub const SQLITE_EMPTY: i32 = 16;
/// The database schema changed
pub const SQLITE_SCHEMA: i32 = 17;
/// String or BLOB exceeds size limit
pub const SQLITE_TOOBIG: i32 = 18;
/// Abort due to constraint violation
pub const SQLITE_CONSTRAINT: i32 = 19;
/// Data type mismatch
pub const SQLITE_MISMATCH: i32 = 20;
/// Library used incorrectly
pub const SQLITE_MISUSE: i32 = 21;
/// Uses OS features not supported on host
pub const SQLITE_NOLFS: i32 = 22;
/// Authorization denied
pub const SQLITE_AUTH: i32 = 23;
/// Not used
pub const SQLITE_FORMAT: i32 = 24;
/// 2nd parameter to sqlite3_bind out of range
pub const SQLITE_RANGE: i32 = 25;
/// File opened that is not a database file
pub const SQLITE_NOTADB: i32 = 26;
/// Notifications from sqlite3_log()
pub const SQLITE_NOTICE: i32 = 27;
/// Warnings from sqlite3_log()
pub const SQLITE_WARNING: i32 = 28;
/// sqlite3_step() has another row ready
pub const SQLITE_ROW: i32 = 100;
/// sqlite3_step() has finished executing
pub const SQLITE_DONE: i32 = 101;

pub const SQLITE_ERROR_MISSING_COLLSEQ: i32 = SQLITE_ERROR | 1 << 8;
pub const SQLITE_ERROR_RETRY: i32 = SQLITE_ERROR | 2 << 8;
pub const SQLITE_ERROR_SNAPSHOT: i32 = SQLITE_ERROR | 3 << 8;
pub const SQLITE_ERROR_RESERVESIZE: i32 = SQLITE_ERROR | 4 << 8;
pub const SQLITE_ERROR_KEY: i32 = SQLITE_ERROR | 5 << 8;
pub const SQLITE_ERROR_UNABLE: i32 = SQLITE_ERROR | 6 << 8;
pub const SQLITE_IOERR_READ: i32 = SQLITE_IOERR | 1 << 8;
pub const SQLITE_IOERR_SHORT_READ: i32 = SQLITE_IOERR | 2 << 8;
pub const SQLITE_IOERR_WRITE: i32 = SQLITE_IOERR | 3 << 8;
pub const SQLITE_IOERR_FSYNC: i32 = SQLITE_IOERR | 4 << 8;
pub const SQLITE_IOERR_DIR_FSYNC: i32 = SQLITE_IOERR | 5 << 8;
pub const SQLITE_IOERR_TRUNCATE: i32 = SQLITE_IOERR | 6 << 8;
pub const SQLITE_IOERR_FSTAT: i32 = SQLITE_IOERR | 7 << 8;
pub const SQLITE_IOERR_UNLOCK: i32 = SQLITE_IOERR | 8 << 8;
pub const SQLITE_IOERR_RDLOCK: i32 = SQLITE_IOERR | 9 << 8;
pub const SQLITE_IOERR_DELETE: i32 = SQLITE_IOERR | 10 << 8;
pub const SQLITE_IOERR_BLOCKED: i32 = SQLITE_IOERR | 11 << 8;
pub const SQLITE_IOERR_NOMEM: i32 = SQLITE_IOERR | 12 << 8;
pub const SQLITE_IOERR_ACCESS: i32 = SQLITE_IOERR | 13 << 8;
pub const SQLITE_IOERR_CHECKRESERVEDLOCK: i32 = SQLITE_IOERR | 14 << 8;
pub const SQLITE_IOERR_LOCK: i32 = SQLITE_IOERR | 15 << 8;
pub const SQLITE_IOERR_CLOSE: i32 = SQLITE_IOERR | 16 << 8;
pub const SQLITE_IOERR_DIR_CLOSE: i32 = SQLITE_IOERR | 17 << 8;
pub const SQLITE_IOERR_SHMOPEN: i32 = SQLITE_IOERR | 18 << 8;
pub const SQLITE_IOERR_SHMSIZE: i32 = SQLITE_IOERR | 19 << 8;
pub const SQLITE_IOERR_SHMLOCK: i32 = SQLITE_IOERR | 20 << 8;
pub const SQLITE_IOERR_SHMMAP: i32 = SQLITE_IOERR | 21 << 8;
pub const SQLITE_IOERR_SEEK: i32 = SQLITE_IOERR | 22 << 8;
pub const SQLITE_IOERR_DELETE_NOENT: i32 = SQLITE_IOERR | 23 << 8;
pub const SQLITE_IOERR_MMAP: i32 = SQLITE_IOERR | 24 << 8;
pub const SQLITE_IOERR_GETTEMPPATH: i32 = SQLITE_IOERR | 25 << 8;
pub const SQLITE_IOERR_CONVPATH: i32 = SQLITE_IOERR | 26 << 8;
pub const SQLITE_IOERR_VNODE: i32 = SQLITE_IOERR | 27 << 8;
pub const SQLITE_IOERR_AUTH: i32 = SQLITE_IOERR | 28 << 8;
pub const SQLITE_IOERR_BEGIN_ATOMIC: i32 = SQLITE_IOERR | 29 << 8;
pub const SQLITE_IOERR_COMMIT_ATOMIC: i32 = SQLITE_IOERR | 30 << 8;
pub const SQLITE_IOERR_ROLLBACK_ATOMIC: i32 = SQLITE_IOERR | 31 << 8;
pub const SQLITE_IOERR_DATA: i32 = SQLITE_IOERR | 32 << 8;
pub const SQLITE_IOERR_CORRUPTFS: i32 = SQLITE_IOERR | 33 << 8;
pub const SQLITE_IOERR_IN_PAGE: i32 = SQLITE_IOERR | 34 << 8;
pub const SQLITE_IOERR_BADKEY: i32 = SQLITE_IOERR | 35 << 8;
pub const SQLITE_IOERR_CODEC: i32 = SQLITE_IOERR | 36 << 8;
pub const SQLITE_LOCKED_SHAREDCACHE: i32 = SQLITE_LOCKED | 1 << 8;
pub const SQLITE_LOCKED_VTAB: i32 = SQLITE_LOCKED | 2 << 8;
pub const SQLITE_BUSY_RECOVERY: i32 = SQLITE_BUSY | 1 << 8;
pub const SQLITE_BUSY_SNAPSHOT: i32 = SQLITE_BUSY | 2 << 8;
pub const SQLITE_BUSY_TIMEOUT: i32 = SQLITE_BUSY | 3 << 8;
pub const SQLITE_CANTOPEN_NOTEMPDIR: i32 = SQLITE_CANTOPEN | 1 << 8;
pub const SQLITE_CANTOPEN_ISDIR: i32 = SQLITE_CANTOPEN | 2 << 8;
pub const SQLITE_CANTOPEN_FULLPATH: i32 = SQLITE_CANTOPEN | 3 << 8;
pub const SQLITE_CANTOPEN_CONVPATH: i32 = SQLITE_CANTOPEN | 4 << 8;
pub const SQLITE_CANTOPEN_DIRTYWAL: i32 = SQLITE_CANTOPEN | 5 << 8; /* Not Used */
pub const SQLITE_CANTOPEN_SYMLINK: i32 = SQLITE_CANTOPEN | 6 << 8;
pub const SQLITE_CORRUPT_VTAB: i32 = SQLITE_CORRUPT | 1 << 8;
pub const SQLITE_CORRUPT_SEQUENCE: i32 = SQLITE_CORRUPT | 2 << 8;
pub const SQLITE_CORRUPT_INDEX: i32 = SQLITE_CORRUPT | 3 << 8;
pub const SQLITE_READONLY_RECOVERY: i32 = SQLITE_READONLY | 1 << 8;
pub const SQLITE_READONLY_CANTLOCK: i32 = SQLITE_READONLY | 2 << 8;
pub const SQLITE_READONLY_ROLLBACK: i32 = SQLITE_READONLY | 3 << 8;
pub const SQLITE_READONLY_DBMOVED: i32 = SQLITE_READONLY | 4 << 8;
pub const SQLITE_READONLY_CANTINIT: i32 = SQLITE_READONLY | 5 << 8;
pub const SQLITE_READONLY_DIRECTORY: i32 = SQLITE_READONLY | 6 << 8;
pub const SQLITE_ABORT_ROLLBACK: i32 = SQLITE_ABORT | 2 << 8;
pub const SQLITE_CONSTRAINT_CHECK: i32 = SQLITE_CONSTRAINT | 1 << 8;
pub const SQLITE_CONSTRAINT_COMMITHOOK: i32 = SQLITE_CONSTRAINT | 2 << 8;
pub const SQLITE_CONSTRAINT_FOREIGNKEY: i32 = SQLITE_CONSTRAINT | 3 << 8;
pub const SQLITE_CONSTRAINT_FUNCTION: i32 = SQLITE_CONSTRAINT | 4 << 8;
pub const SQLITE_CONSTRAINT_NOTNULL: i32 = SQLITE_CONSTRAINT | 5 << 8;
pub const SQLITE_CONSTRAINT_PRIMARYKEY: i32 = SQLITE_CONSTRAINT | 6 << 8;
pub const SQLITE_CONSTRAINT_TRIGGER: i32 = SQLITE_CONSTRAINT | 7 << 8;
pub const SQLITE_CONSTRAINT_UNIQUE: i32 = SQLITE_CONSTRAINT | 8 << 8;
pub const SQLITE_CONSTRAINT_VTAB: i32 = SQLITE_CONSTRAINT | 9 << 8;
pub const SQLITE_CONSTRAINT_ROWID: i32 = SQLITE_CONSTRAINT | 10 << 8;
pub const SQLITE_CONSTRAINT_PINNED: i32 = SQLITE_CONSTRAINT | 11 << 8;
pub const SQLITE_CONSTRAINT_DATATYPE: i32 = SQLITE_CONSTRAINT | 12 << 8;
pub const SQLITE_NOTICE_RECOVER_WAL: i32 = SQLITE_NOTICE | 1 << 8;
pub const SQLITE_NOTICE_RECOVER_ROLLBACK: i32 = SQLITE_NOTICE | 2 << 8;
pub const SQLITE_NOTICE_RBU: i32 = SQLITE_NOTICE | 3 << 8;
pub const SQLITE_WARNING_AUTOINDEX: i32 = SQLITE_WARNING | 1 << 8;
pub const SQLITE_AUTH_USER: i32 = SQLITE_AUTH | 1 << 8;
pub const SQLITE_OK_LOAD_PERMANENTLY: i32 = SQLITE_OK | 1 << 8;
pub const SQLITE_OK_SYMLINK: i32 = SQLITE_OK | 2 << 8;

unsafe extern "C" {
    pub fn sqlite3_errcode(db: *mut sqlite3) -> c_int;
    pub fn sqlite3_extended_errcode(db: *mut sqlite3) -> c_int;
    pub fn sqlite3_errmsg(arg1: *mut sqlite3) -> *const c_char;
    pub fn sqlite3_errstr(arg1: c_int) -> *const c_char;
    pub fn sqlite3_error_offset(db: *mut sqlite3) -> c_int;
    pub fn sqlite3_set_errmsg(db: *mut sqlite3, errcode: c_int, zErrMsg: *const c_char) -> c_int;
}
