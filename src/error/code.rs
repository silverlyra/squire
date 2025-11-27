use core::{ffi::CStr, fmt, num::NonZero};

use sqlite::sqlite3_errstr;

/// A SQLite error [return code][].
///
/// `ErrorCode` can also represent an error returned by Squire.
///
/// [return code]: https://sqlite.org/rescode.html
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub struct ErrorCode(NonZero<i32>);

macro_rules! code {
    ($category:literal) => {
        $category | 0xc0
    };

    ($category:literal, $detail:literal) => {
        code!($category) | ($detail << 8)
    };
}

pub(crate) const SQUIRE_ERROR: i32 = code!(0);
pub(crate) const SQUIRE_ERROR_FETCH: i32 = code!(1);
pub(crate) const SQUIRE_ERROR_FETCH_PARSE: i32 = code!(1, 1);
pub(crate) const SQUIRE_ERROR_FETCH_RANGE: i32 = code!(1, 2);
pub(crate) const SQUIRE_ERROR_PARAMETER: i32 = code!(2);
pub(crate) const SQUIRE_ERROR_PARAMETER_BIND: i32 = code!(2, 1);
pub(crate) const SQUIRE_ERROR_PARAMETER_RANGE: i32 = code!(2, 2);
pub(crate) const SQUIRE_ERROR_PARAMETER_RESOLVE: i32 = code!(2, 3);
pub(crate) const SQUIRE_ERROR_PARAMETER_INVALID_INDEX: i32 = code!(2, 4);

impl ErrorCode {
    /// Wrap a return value from SQLite in an [`ErrorCode`].
    ///
    /// Returns `None` if `code` is 0.
    pub const fn new(code: i32) -> Option<Self> {
        match NonZero::new(code) {
            Some(code) => Some(Self(code)),
            None => None,
        }
    }

    /// Wrap a known-error return value from SQLite in an [`ErrorCode`].
    ///
    /// # Safety
    ///
    /// `code` must not be zero.
    pub const unsafe fn new_unchecked(code: i32) -> Self {
        Self(unsafe { NonZero::new_unchecked(code) })
    }

    const fn define(code: i32) -> Self {
        unsafe { Self::new_unchecked(code) }
    }

    /// The error code itself as an [`i32`].
    pub const fn raw(&self) -> i32 {
        self.0.get()
    }

    /// `true` if this error code originated from within SQLite.
    pub const fn is_sqlite(&self) -> bool {
        !self.is_squire()
    }

    /// `true` if this error code originated from Squire, or a crate it
    /// [integrates](crate::IntegrationError) with.
    pub const fn is_squire(&self) -> bool {
        self.raw() & Self::SQUIRE.raw() != 0
    }

    /// The name of the error code constant.
    ///
    /// Errors from SQLite have a name starting with `SQLITE_`.
    /// Errors from Squire have a name starting with `SQUIRE_ERROR`.
    pub const fn name(&self) -> Option<&'static str> {
        #[allow(deprecated)]
        match *self {
            // Primary result codes
            Self::ABORT => Some("SQLITE_ABORT"),
            Self::AUTH => Some("SQLITE_AUTH"),
            Self::BUSY => Some("SQLITE_BUSY"),
            Self::CANTOPEN => Some("SQLITE_CANTOPEN"),
            Self::CONSTRAINT => Some("SQLITE_CONSTRAINT"),
            Self::CORRUPT => Some("SQLITE_CORRUPT"),
            Self::EMPTY => Some("SQLITE_EMPTY"),
            Self::ERROR => Some("SQLITE_ERROR"),
            Self::FORMAT => Some("SQLITE_FORMAT"),
            Self::FULL => Some("SQLITE_FULL"),
            Self::INTERNAL => Some("SQLITE_INTERNAL"),
            Self::INTERRUPT => Some("SQLITE_INTERRUPT"),
            Self::IOERR => Some("SQLITE_IOERR"),
            Self::LOCKED => Some("SQLITE_LOCKED"),
            Self::MISMATCH => Some("SQLITE_MISMATCH"),
            Self::MISUSE => Some("SQLITE_MISUSE"),
            Self::NOLFS => Some("SQLITE_NOLFS"),
            Self::NOMEM => Some("SQLITE_NOMEM"),
            Self::NOTADB => Some("SQLITE_NOTADB"),
            Self::NOTFOUND => Some("SQLITE_NOTFOUND"),
            Self::PERM => Some("SQLITE_PERM"),
            Self::PROTOCOL => Some("SQLITE_PROTOCOL"),
            Self::RANGE => Some("SQLITE_RANGE"),
            Self::READONLY => Some("SQLITE_READONLY"),
            Self::SCHEMA => Some("SQLITE_SCHEMA"),
            Self::TOOBIG => Some("SQLITE_TOOBIG"),

            // Extended result codes
            // Abort errors
            Self::ABORT_ROLLBACK => Some("SQLITE_ABORT_ROLLBACK"),

            // Auth errors
            Self::AUTH_USER => Some("SQLITE_AUTH_USER"),

            // Busy errors
            Self::BUSY_RECOVERY => Some("SQLITE_BUSY_RECOVERY"),
            Self::BUSY_SNAPSHOT => Some("SQLITE_BUSY_SNAPSHOT"),
            Self::BUSY_TIMEOUT => Some("SQLITE_BUSY_TIMEOUT"),

            // CantOpen errors
            Self::CANTOPEN_CONVPATH => Some("SQLITE_CANTOPEN_CONVPATH"),
            Self::CANTOPEN_DIRTYWAL => Some("SQLITE_CANTOPEN_DIRTYWAL"),
            Self::CANTOPEN_FULLPATH => Some("SQLITE_CANTOPEN_FULLPATH"),
            Self::CANTOPEN_ISDIR => Some("SQLITE_CANTOPEN_ISDIR"),
            Self::CANTOPEN_NOTEMPDIR => Some("SQLITE_CANTOPEN_NOTEMPDIR"),
            Self::CANTOPEN_SYMLINK => Some("SQLITE_CANTOPEN_SYMLINK"),

            // Constraint errors
            Self::CONSTRAINT_CHECK => Some("SQLITE_CONSTRAINT_CHECK"),
            Self::CONSTRAINT_COMMITHOOK => Some("SQLITE_CONSTRAINT_COMMITHOOK"),
            Self::CONSTRAINT_DATATYPE => Some("SQLITE_CONSTRAINT_DATATYPE"),
            Self::CONSTRAINT_FOREIGNKEY => Some("SQLITE_CONSTRAINT_FOREIGNKEY"),
            Self::CONSTRAINT_FUNCTION => Some("SQLITE_CONSTRAINT_FUNCTION"),
            Self::CONSTRAINT_NOTNULL => Some("SQLITE_CONSTRAINT_NOTNULL"),
            Self::CONSTRAINT_PINNED => Some("SQLITE_CONSTRAINT_PINNED"),
            Self::CONSTRAINT_PRIMARYKEY => Some("SQLITE_CONSTRAINT_PRIMARYKEY"),
            Self::CONSTRAINT_ROWID => Some("SQLITE_CONSTRAINT_ROWID"),
            Self::CONSTRAINT_TRIGGER => Some("SQLITE_CONSTRAINT_TRIGGER"),
            Self::CONSTRAINT_UNIQUE => Some("SQLITE_CONSTRAINT_UNIQUE"),
            Self::CONSTRAINT_VTAB => Some("SQLITE_CONSTRAINT_VTAB"),

            // Corrupt errors
            Self::CORRUPT_INDEX => Some("SQLITE_CORRUPT_INDEX"),
            Self::CORRUPT_SEQUENCE => Some("SQLITE_CORRUPT_SEQUENCE"),
            Self::CORRUPT_VTAB => Some("SQLITE_CORRUPT_VTAB"),

            // General errors
            Self::ERROR_MISSING_COLLSEQ => Some("SQLITE_ERROR_MISSING_COLLSEQ"),
            Self::ERROR_RETRY => Some("SQLITE_ERROR_RETRY"),
            Self::ERROR_SNAPSHOT => Some("SQLITE_ERROR_SNAPSHOT"),

            // IO errors
            Self::IOERR_ACCESS => Some("SQLITE_IOERR_ACCESS"),
            Self::IOERR_AUTH => Some("SQLITE_IOERR_AUTH"),
            Self::IOERR_BEGIN_ATOMIC => Some("SQLITE_IOERR_BEGIN_ATOMIC"),
            Self::IOERR_BLOCKED => Some("SQLITE_IOERR_BLOCKED"),
            Self::IOERR_CHECKRESERVEDLOCK => Some("SQLITE_IOERR_CHECKRESERVEDLOCK"),
            Self::IOERR_CLOSE => Some("SQLITE_IOERR_CLOSE"),
            Self::IOERR_COMMIT_ATOMIC => Some("SQLITE_IOERR_COMMIT_ATOMIC"),
            Self::IOERR_CONVPATH => Some("SQLITE_IOERR_CONVPATH"),
            Self::IOERR_CORRUPTFS => Some("SQLITE_IOERR_CORRUPTFS"),
            Self::IOERR_DATA => Some("SQLITE_IOERR_DATA"),
            Self::IOERR_DELETE => Some("SQLITE_IOERR_DELETE"),
            Self::IOERR_DELETE_NOENT => Some("SQLITE_IOERR_DELETE_NOENT"),
            Self::IOERR_DIR_CLOSE => Some("SQLITE_IOERR_DIR_CLOSE"),
            Self::IOERR_DIR_FSYNC => Some("SQLITE_IOERR_DIR_FSYNC"),
            Self::IOERR_FSTAT => Some("SQLITE_IOERR_FSTAT"),
            Self::IOERR_FSYNC => Some("SQLITE_IOERR_FSYNC"),
            Self::IOERR_GETTEMPPATH => Some("SQLITE_IOERR_GETTEMPPATH"),
            Self::IOERR_LOCK => Some("SQLITE_IOERR_LOCK"),
            Self::IOERR_MMAP => Some("SQLITE_IOERR_MMAP"),
            Self::IOERR_NOMEM => Some("SQLITE_IOERR_NOMEM"),
            Self::IOERR_RDLOCK => Some("SQLITE_IOERR_RDLOCK"),
            Self::IOERR_READ => Some("SQLITE_IOERR_READ"),
            Self::IOERR_ROLLBACK_ATOMIC => Some("SQLITE_IOERR_ROLLBACK_ATOMIC"),
            Self::IOERR_SEEK => Some("SQLITE_IOERR_SEEK"),
            Self::IOERR_SHMLOCK => Some("SQLITE_IOERR_SHMLOCK"),
            Self::IOERR_SHMMAP => Some("SQLITE_IOERR_SHMMAP"),
            Self::IOERR_SHMOPEN => Some("SQLITE_IOERR_SHMOPEN"),
            Self::IOERR_SHMSIZE => Some("SQLITE_IOERR_SHMSIZE"),
            Self::IOERR_SHORT_READ => Some("SQLITE_IOERR_SHORT_READ"),
            Self::IOERR_TRUNCATE => Some("SQLITE_IOERR_TRUNCATE"),
            Self::IOERR_UNLOCK => Some("SQLITE_IOERR_UNLOCK"),
            Self::IOERR_VNODE => Some("SQLITE_IOERR_VNODE"),
            Self::IOERR_WRITE => Some("SQLITE_IOERR_WRITE"),

            // Locked errors
            Self::LOCKED_SHAREDCACHE => Some("SQLITE_LOCKED_SHAREDCACHE"),
            Self::LOCKED_VTAB => Some("SQLITE_LOCKED_VTAB"),

            // ReadOnly errors
            Self::READONLY_CANTINIT => Some("SQLITE_READONLY_CANTINIT"),
            Self::READONLY_CANTLOCK => Some("SQLITE_READONLY_CANTLOCK"),
            Self::READONLY_DBMOVED => Some("SQLITE_READONLY_DBMOVED"),
            Self::READONLY_DIRECTORY => Some("SQLITE_READONLY_DIRECTORY"),
            Self::READONLY_RECOVERY => Some("SQLITE_READONLY_RECOVERY"),
            Self::READONLY_ROLLBACK => Some("SQLITE_READONLY_ROLLBACK"),

            // Squire errors
            Self::SQUIRE => Some("SQUIRE_ERROR"),
            Self::SQUIRE_FETCH => Some("SQUIRE_ERROR_FETCH"),
            Self::SQUIRE_FETCH_PARSE => Some("SQUIRE_ERROR_FETCH_PARSE"),
            Self::SQUIRE_FETCH_RANGE => Some("SQUIRE_ERROR_FETCH_RANGE"),
            Self::SQUIRE_PARAMETER => Some("SQUIRE_ERROR_PARAMETER"),
            Self::SQUIRE_PARAMETER_BIND => Some("SQUIRE_ERROR_PARAMETER_BIND"),
            Self::SQUIRE_PARAMETER_RANGE => Some("SQUIRE_ERROR_PARAMETER_RANGE"),
            Self::SQUIRE_PARAMETER_RESOLVE => Some("SQUIRE_ERROR_PARAMETER_RESOLVE"),

            // Unrecognized code
            _ => None,
        }
    }

    /// A message describing this error.
    pub fn description(&self) -> &'static str {
        match *self {
            Self::SQUIRE_FETCH => "error fetching column value",
            Self::SQUIRE_FETCH_PARSE => "error parsing column value",
            Self::SQUIRE_FETCH_RANGE => "column value out of range",
            Self::SQUIRE_PARAMETER => "error binding parameter",
            Self::SQUIRE_PARAMETER_BIND => "error binding parameter value",
            Self::SQUIRE_PARAMETER_RANGE => "parameter value out of range",
            Self::SQUIRE_PARAMETER_RESOLVE => "error resolving parameter index",
            Self::SQUIRE_PARAMETER_INVALID_INDEX => "parameter index must be > 0",

            _ => {
                let ptr = unsafe { sqlite3_errstr(self.raw()) };
                if ptr.is_null() {
                    return "unknown error";
                }

                let bytes = unsafe { CStr::from_ptr(ptr) }.to_bytes();
                unsafe { str::from_utf8_unchecked(bytes) }
            }
        }
    }

    pub(crate) const ABORT: Self = Self::define(sqlite::SQLITE_ABORT);
    pub(crate) const ABORT_ROLLBACK: Self = Self::define(sqlite::SQLITE_ABORT_ROLLBACK);
    pub(crate) const AUTH: Self = Self::define(sqlite::SQLITE_AUTH);
    pub(crate) const AUTH_USER: Self = Self::define(sqlite::SQLITE_AUTH_USER);
    pub(crate) const BUSY: Self = Self::define(sqlite::SQLITE_BUSY);
    pub(crate) const BUSY_RECOVERY: Self = Self::define(sqlite::SQLITE_BUSY_RECOVERY);
    pub(crate) const BUSY_SNAPSHOT: Self = Self::define(sqlite::SQLITE_BUSY_SNAPSHOT);
    pub(crate) const BUSY_TIMEOUT: Self = Self::define(sqlite::SQLITE_BUSY_TIMEOUT);
    pub(crate) const CANTOPEN: Self = Self::define(sqlite::SQLITE_CANTOPEN);
    pub(crate) const CANTOPEN_CONVPATH: Self = Self::define(sqlite::SQLITE_CANTOPEN_CONVPATH);
    pub(crate) const CANTOPEN_DIRTYWAL: Self = Self::define(sqlite::SQLITE_CANTOPEN_DIRTYWAL);
    pub(crate) const CANTOPEN_FULLPATH: Self = Self::define(sqlite::SQLITE_CANTOPEN_FULLPATH);
    pub(crate) const CANTOPEN_ISDIR: Self = Self::define(sqlite::SQLITE_CANTOPEN_ISDIR);
    pub(crate) const CANTOPEN_NOTEMPDIR: Self = Self::define(sqlite::SQLITE_CANTOPEN_NOTEMPDIR);
    pub(crate) const CANTOPEN_SYMLINK: Self = Self::define(sqlite::SQLITE_CANTOPEN_SYMLINK);
    pub(crate) const CONSTRAINT: Self = Self::define(sqlite::SQLITE_CONSTRAINT);
    pub(crate) const CONSTRAINT_CHECK: Self = Self::define(sqlite::SQLITE_CONSTRAINT_CHECK);
    pub(crate) const CONSTRAINT_COMMITHOOK: Self =
        Self::define(sqlite::SQLITE_CONSTRAINT_COMMITHOOK);
    pub(crate) const CONSTRAINT_DATATYPE: Self = Self::define(sqlite::SQLITE_CONSTRAINT_DATATYPE);
    pub(crate) const CONSTRAINT_FOREIGNKEY: Self =
        Self::define(sqlite::SQLITE_CONSTRAINT_FOREIGNKEY);
    pub(crate) const CONSTRAINT_FUNCTION: Self = Self::define(sqlite::SQLITE_CONSTRAINT_FUNCTION);
    pub(crate) const CONSTRAINT_NOTNULL: Self = Self::define(sqlite::SQLITE_CONSTRAINT_NOTNULL);
    pub(crate) const CONSTRAINT_PINNED: Self = Self::define(sqlite::SQLITE_CONSTRAINT_PINNED);
    pub(crate) const CONSTRAINT_PRIMARYKEY: Self =
        Self::define(sqlite::SQLITE_CONSTRAINT_PRIMARYKEY);
    pub(crate) const CONSTRAINT_ROWID: Self = Self::define(sqlite::SQLITE_CONSTRAINT_ROWID);
    pub(crate) const CONSTRAINT_TRIGGER: Self = Self::define(sqlite::SQLITE_CONSTRAINT_TRIGGER);
    pub(crate) const CONSTRAINT_UNIQUE: Self = Self::define(sqlite::SQLITE_CONSTRAINT_UNIQUE);
    pub(crate) const CONSTRAINT_VTAB: Self = Self::define(sqlite::SQLITE_CONSTRAINT_VTAB);
    pub(crate) const CORRUPT: Self = Self::define(sqlite::SQLITE_CORRUPT);
    pub(crate) const CORRUPT_INDEX: Self = Self::define(sqlite::SQLITE_CORRUPT_INDEX);
    pub(crate) const CORRUPT_SEQUENCE: Self = Self::define(sqlite::SQLITE_CORRUPT_SEQUENCE);
    pub(crate) const CORRUPT_VTAB: Self = Self::define(sqlite::SQLITE_CORRUPT_VTAB);
    pub(crate) const EMPTY: Self = Self::define(sqlite::SQLITE_EMPTY);
    pub(crate) const ERROR: Self = Self::define(sqlite::SQLITE_ERROR);
    pub(crate) const ERROR_MISSING_COLLSEQ: Self =
        Self::define(sqlite::SQLITE_ERROR_MISSING_COLLSEQ);
    pub(crate) const ERROR_RETRY: Self = Self::define(sqlite::SQLITE_ERROR_RETRY);
    pub(crate) const ERROR_SNAPSHOT: Self = Self::define(sqlite::SQLITE_ERROR_SNAPSHOT);
    pub(crate) const FORMAT: Self = Self::define(sqlite::SQLITE_FORMAT);
    pub(crate) const FULL: Self = Self::define(sqlite::SQLITE_FULL);
    pub(crate) const INTERNAL: Self = Self::define(sqlite::SQLITE_INTERNAL);
    pub(crate) const INTERRUPT: Self = Self::define(sqlite::SQLITE_INTERRUPT);
    pub(crate) const IOERR: Self = Self::define(sqlite::SQLITE_IOERR);
    pub(crate) const IOERR_ACCESS: Self = Self::define(sqlite::SQLITE_IOERR_ACCESS);
    pub(crate) const IOERR_AUTH: Self = Self::define(sqlite::SQLITE_IOERR_AUTH);
    pub(crate) const IOERR_BEGIN_ATOMIC: Self = Self::define(sqlite::SQLITE_IOERR_BEGIN_ATOMIC);
    pub(crate) const IOERR_BLOCKED: Self = Self::define(sqlite::SQLITE_IOERR_BLOCKED);
    pub(crate) const IOERR_CHECKRESERVEDLOCK: Self =
        Self::define(sqlite::SQLITE_IOERR_CHECKRESERVEDLOCK);
    pub(crate) const IOERR_CLOSE: Self = Self::define(sqlite::SQLITE_IOERR_CLOSE);
    pub(crate) const IOERR_COMMIT_ATOMIC: Self = Self::define(sqlite::SQLITE_IOERR_COMMIT_ATOMIC);
    pub(crate) const IOERR_CONVPATH: Self = Self::define(sqlite::SQLITE_IOERR_CONVPATH);
    pub(crate) const IOERR_CORRUPTFS: Self = Self::define(sqlite::SQLITE_IOERR_CORRUPTFS);
    pub(crate) const IOERR_DATA: Self = Self::define(sqlite::SQLITE_IOERR_DATA);
    pub(crate) const IOERR_DELETE: Self = Self::define(sqlite::SQLITE_IOERR_DELETE);
    pub(crate) const IOERR_DELETE_NOENT: Self = Self::define(sqlite::SQLITE_IOERR_DELETE_NOENT);
    pub(crate) const IOERR_DIR_CLOSE: Self = Self::define(sqlite::SQLITE_IOERR_DIR_CLOSE);
    pub(crate) const IOERR_DIR_FSYNC: Self = Self::define(sqlite::SQLITE_IOERR_DIR_FSYNC);
    pub(crate) const IOERR_FSTAT: Self = Self::define(sqlite::SQLITE_IOERR_FSTAT);
    pub(crate) const IOERR_FSYNC: Self = Self::define(sqlite::SQLITE_IOERR_FSYNC);
    pub(crate) const IOERR_GETTEMPPATH: Self = Self::define(sqlite::SQLITE_IOERR_GETTEMPPATH);
    pub(crate) const IOERR_LOCK: Self = Self::define(sqlite::SQLITE_IOERR_LOCK);
    pub(crate) const IOERR_MMAP: Self = Self::define(sqlite::SQLITE_IOERR_MMAP);
    pub(crate) const IOERR_NOMEM: Self = Self::define(sqlite::SQLITE_IOERR_NOMEM);
    pub(crate) const IOERR_RDLOCK: Self = Self::define(sqlite::SQLITE_IOERR_RDLOCK);
    pub(crate) const IOERR_READ: Self = Self::define(sqlite::SQLITE_IOERR_READ);
    pub(crate) const IOERR_ROLLBACK_ATOMIC: Self =
        Self::define(sqlite::SQLITE_IOERR_ROLLBACK_ATOMIC);
    pub(crate) const IOERR_SEEK: Self = Self::define(sqlite::SQLITE_IOERR_SEEK);
    pub(crate) const IOERR_SHMLOCK: Self = Self::define(sqlite::SQLITE_IOERR_SHMLOCK);
    pub(crate) const IOERR_SHMMAP: Self = Self::define(sqlite::SQLITE_IOERR_SHMMAP);
    pub(crate) const IOERR_SHMOPEN: Self = Self::define(sqlite::SQLITE_IOERR_SHMOPEN);
    pub(crate) const IOERR_SHMSIZE: Self = Self::define(sqlite::SQLITE_IOERR_SHMSIZE);
    pub(crate) const IOERR_SHORT_READ: Self = Self::define(sqlite::SQLITE_IOERR_SHORT_READ);
    pub(crate) const IOERR_TRUNCATE: Self = Self::define(sqlite::SQLITE_IOERR_TRUNCATE);
    pub(crate) const IOERR_UNLOCK: Self = Self::define(sqlite::SQLITE_IOERR_UNLOCK);
    pub(crate) const IOERR_VNODE: Self = Self::define(sqlite::SQLITE_IOERR_VNODE);
    pub(crate) const IOERR_WRITE: Self = Self::define(sqlite::SQLITE_IOERR_WRITE);
    pub(crate) const LOCKED: Self = Self::define(sqlite::SQLITE_LOCKED);
    pub(crate) const LOCKED_SHAREDCACHE: Self = Self::define(sqlite::SQLITE_LOCKED_SHAREDCACHE);
    pub(crate) const LOCKED_VTAB: Self = Self::define(sqlite::SQLITE_LOCKED_VTAB);
    pub(crate) const MISMATCH: Self = Self::define(sqlite::SQLITE_MISMATCH);
    pub(crate) const MISUSE: Self = Self::define(sqlite::SQLITE_MISUSE);
    pub(crate) const NOLFS: Self = Self::define(sqlite::SQLITE_NOLFS);
    pub(crate) const NOMEM: Self = Self::define(sqlite::SQLITE_NOMEM);
    pub(crate) const NOTADB: Self = Self::define(sqlite::SQLITE_NOTADB);
    pub(crate) const NOTFOUND: Self = Self::define(sqlite::SQLITE_NOTFOUND);
    pub(crate) const PERM: Self = Self::define(sqlite::SQLITE_PERM);
    pub(crate) const PROTOCOL: Self = Self::define(sqlite::SQLITE_PROTOCOL);
    pub(crate) const RANGE: Self = Self::define(sqlite::SQLITE_RANGE);
    pub(crate) const READONLY: Self = Self::define(sqlite::SQLITE_READONLY);
    pub(crate) const READONLY_CANTINIT: Self = Self::define(sqlite::SQLITE_READONLY_CANTINIT);
    pub(crate) const READONLY_CANTLOCK: Self = Self::define(sqlite::SQLITE_READONLY_CANTLOCK);
    pub(crate) const READONLY_DBMOVED: Self = Self::define(sqlite::SQLITE_READONLY_DBMOVED);
    pub(crate) const READONLY_DIRECTORY: Self = Self::define(sqlite::SQLITE_READONLY_DIRECTORY);
    pub(crate) const READONLY_RECOVERY: Self = Self::define(sqlite::SQLITE_READONLY_RECOVERY);
    pub(crate) const READONLY_ROLLBACK: Self = Self::define(sqlite::SQLITE_READONLY_ROLLBACK);
    pub(crate) const SCHEMA: Self = Self::define(sqlite::SQLITE_SCHEMA);
    pub(crate) const TOOBIG: Self = Self::define(sqlite::SQLITE_TOOBIG);

    pub(crate) const SQUIRE: Self = Self::define(SQUIRE_ERROR);
    pub(crate) const SQUIRE_FETCH: Self = Self::define(SQUIRE_ERROR_FETCH);
    pub(crate) const SQUIRE_FETCH_PARSE: Self = Self::define(SQUIRE_ERROR_FETCH_PARSE);
    pub(crate) const SQUIRE_FETCH_RANGE: Self = Self::define(SQUIRE_ERROR_FETCH_RANGE);
    pub(crate) const SQUIRE_PARAMETER: Self = Self::define(SQUIRE_ERROR_PARAMETER);
    pub(crate) const SQUIRE_PARAMETER_BIND: Self = Self::define(SQUIRE_ERROR_PARAMETER_BIND);
    pub(crate) const SQUIRE_PARAMETER_RANGE: Self = Self::define(SQUIRE_ERROR_PARAMETER_RANGE);
    pub(crate) const SQUIRE_PARAMETER_RESOLVE: Self = Self::define(SQUIRE_ERROR_PARAMETER_RESOLVE);
    pub(crate) const SQUIRE_PARAMETER_INVALID_INDEX: Self =
        Self::define(SQUIRE_ERROR_PARAMETER_INVALID_INDEX);
}

impl Default for ErrorCode {
    fn default() -> Self {
        Self::ERROR
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.name() {
            Some(name) => write!(f, "{name}"),
            None => write!(f, "0x{:04x}", self.raw()),
        }
    }
}
