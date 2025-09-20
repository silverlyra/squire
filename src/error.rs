use core::{
    ffi::{CStr, c_int},
    fmt,
    num::NonZero,
};

use sqlite::{
    SQLITE_ABORT, SQLITE_ABORT_ROLLBACK, SQLITE_AUTH, SQLITE_AUTH_USER, SQLITE_BUSY,
    SQLITE_BUSY_RECOVERY, SQLITE_BUSY_SNAPSHOT, SQLITE_BUSY_TIMEOUT, SQLITE_CANTOPEN,
    SQLITE_CANTOPEN_CONVPATH, SQLITE_CANTOPEN_DIRTYWAL, SQLITE_CANTOPEN_FULLPATH,
    SQLITE_CANTOPEN_ISDIR, SQLITE_CANTOPEN_NOTEMPDIR, SQLITE_CANTOPEN_SYMLINK, SQLITE_CONSTRAINT,
    SQLITE_CONSTRAINT_CHECK, SQLITE_CONSTRAINT_COMMITHOOK, SQLITE_CONSTRAINT_DATATYPE,
    SQLITE_CONSTRAINT_FOREIGNKEY, SQLITE_CONSTRAINT_FUNCTION, SQLITE_CONSTRAINT_NOTNULL,
    SQLITE_CONSTRAINT_PINNED, SQLITE_CONSTRAINT_PRIMARYKEY, SQLITE_CONSTRAINT_ROWID,
    SQLITE_CONSTRAINT_TRIGGER, SQLITE_CONSTRAINT_UNIQUE, SQLITE_CONSTRAINT_VTAB, SQLITE_CORRUPT,
    SQLITE_CORRUPT_INDEX, SQLITE_CORRUPT_SEQUENCE, SQLITE_CORRUPT_VTAB, SQLITE_EMPTY, SQLITE_ERROR,
    SQLITE_ERROR_MISSING_COLLSEQ, SQLITE_ERROR_RETRY, SQLITE_ERROR_SNAPSHOT, SQLITE_FORMAT,
    SQLITE_FULL, SQLITE_INTERNAL, SQLITE_INTERRUPT, SQLITE_IOERR, SQLITE_IOERR_ACCESS,
    SQLITE_IOERR_AUTH, SQLITE_IOERR_BEGIN_ATOMIC, SQLITE_IOERR_BLOCKED,
    SQLITE_IOERR_CHECKRESERVEDLOCK, SQLITE_IOERR_CLOSE, SQLITE_IOERR_COMMIT_ATOMIC,
    SQLITE_IOERR_CONVPATH, SQLITE_IOERR_CORRUPTFS, SQLITE_IOERR_DATA, SQLITE_IOERR_DELETE,
    SQLITE_IOERR_DELETE_NOENT, SQLITE_IOERR_DIR_CLOSE, SQLITE_IOERR_DIR_FSYNC, SQLITE_IOERR_FSTAT,
    SQLITE_IOERR_FSYNC, SQLITE_IOERR_GETTEMPPATH, SQLITE_IOERR_LOCK, SQLITE_IOERR_MMAP,
    SQLITE_IOERR_NOMEM, SQLITE_IOERR_RDLOCK, SQLITE_IOERR_READ, SQLITE_IOERR_ROLLBACK_ATOMIC,
    SQLITE_IOERR_SEEK, SQLITE_IOERR_SHMLOCK, SQLITE_IOERR_SHMMAP, SQLITE_IOERR_SHMOPEN,
    SQLITE_IOERR_SHMSIZE, SQLITE_IOERR_SHORT_READ, SQLITE_IOERR_TRUNCATE, SQLITE_IOERR_UNLOCK,
    SQLITE_IOERR_VNODE, SQLITE_IOERR_WRITE, SQLITE_LOCKED, SQLITE_LOCKED_SHAREDCACHE,
    SQLITE_LOCKED_VTAB, SQLITE_MISMATCH, SQLITE_MISUSE, SQLITE_NOLFS, SQLITE_NOMEM, SQLITE_NOTADB,
    SQLITE_NOTFOUND, SQLITE_PERM, SQLITE_PROTOCOL, SQLITE_RANGE, SQLITE_READONLY,
    SQLITE_READONLY_CANTINIT, SQLITE_READONLY_CANTLOCK, SQLITE_READONLY_DBMOVED,
    SQLITE_READONLY_DIRECTORY, SQLITE_READONLY_RECOVERY, SQLITE_READONLY_ROLLBACK, SQLITE_SCHEMA,
    SQLITE_TOOBIG, sqlite3_errstr,
};

/// A [`Result`][core::result::Result] returned by a SQLite operation.
pub type Result<T> = core::result::Result<T, Error>;

/// An [error][return-codes] returned by a SQLite operation.
///
/// Use [`category`](Error::category) to inspect the [primary](ErrorCategory)
/// SQLite result code of the error, and [`code`] for the [extended](ErrorCode)
/// result code (if available for this error).
///
/// [return-codes]: https://sqlite.org/rescode.html
#[derive(PartialEq, Eq, Copy, Clone)]
pub struct Error {
    code: NonZero<i32>,
}

impl Error {
    /// Creates a new [error](Error) from a SQLite result code.
    pub const fn new(code: i32) -> Option<Self> {
        match NonZero::new(code) {
            Some(code) => Some(Self { code }),
            None => None,
        }
    }

    pub(crate) const unsafe fn new_unchecked(code: i32) -> Self {
        Self {
            code: unsafe { NonZero::new_unchecked(code) },
        }
    }

    /// Returns a non-specific `SQLITE_ERROR` [error](Error).
    pub(crate) const fn unknown() -> Self {
        unsafe { Self::new_unchecked(SQLITE_ERROR) }
    }

    /// Returns a `SQLITE_MISUSE` [error](Error).
    pub(crate) const fn misuse() -> Self {
        unsafe { Self::new_unchecked(SQLITE_MISUSE) }
    }

    /// Returns a `SQLITE_RANGE` [error](Error).
    pub(crate) const fn range() -> Self {
        unsafe { Self::new_unchecked(SQLITE_RANGE) }
    }

    /// Returns a `SQLITE_TOOBIG` [error](Error).
    pub(crate) const fn too_big() -> Self {
        unsafe { Self::new_unchecked(SQLITE_TOOBIG) }
    }

    /// Returns the raw SQLite result code.
    pub const fn into_inner(self) -> i32 {
        self.code.get()
    }

    /// Returns the [primary result code][] for this error.
    ///
    /// [primary result code]: https://sqlite.org/rescode.html#primary_result_codes_versus_extended_result_codes
    pub const fn category(&self) -> ErrorCategory {
        let primary_code = self.into_inner() & 0xFF;

        #[allow(deprecated)]
        match primary_code {
            SQLITE_ABORT => ErrorCategory::Aborted,
            SQLITE_AUTH => ErrorCategory::Authorization,
            SQLITE_BUSY => ErrorCategory::Busy,
            SQLITE_CANTOPEN => ErrorCategory::CantOpen,
            SQLITE_CONSTRAINT => ErrorCategory::Constraint,
            SQLITE_CORRUPT => ErrorCategory::Corrupt,
            SQLITE_EMPTY => ErrorCategory::Empty,
            SQLITE_ERROR => ErrorCategory::Unknown,
            SQLITE_FORMAT => ErrorCategory::Format,
            SQLITE_FULL => ErrorCategory::Full,
            SQLITE_INTERNAL => ErrorCategory::Internal,
            SQLITE_INTERRUPT => ErrorCategory::Interrupt,
            SQLITE_IOERR => ErrorCategory::Io,
            SQLITE_LOCKED => ErrorCategory::Locked,
            SQLITE_MISMATCH => ErrorCategory::Mismatch,
            SQLITE_MISUSE => ErrorCategory::Misuse,
            SQLITE_NOLFS => ErrorCategory::LargeFile,
            SQLITE_NOMEM => ErrorCategory::OutOfMemory,
            SQLITE_NOTADB => ErrorCategory::InvalidDatabase,
            SQLITE_NOTFOUND => ErrorCategory::NotFound,
            SQLITE_PERM => ErrorCategory::Permission,
            SQLITE_PROTOCOL => ErrorCategory::Protocol,
            SQLITE_RANGE => ErrorCategory::Range,
            SQLITE_READONLY => ErrorCategory::ReadOnly,
            SQLITE_SCHEMA => ErrorCategory::Schema,
            SQLITE_TOOBIG => ErrorCategory::TooBig,
            _ => ErrorCategory::Unknown,
        }
    }

    /// Returns the [extended result code][] for this error.
    ///
    /// Returns `None` if this is a primary result code or if the extended code
    /// is not recognized.
    pub const fn code(&self) -> Option<ErrorCode> {
        #[allow(deprecated)]
        match self.into_inner() {
            // Abort errors
            SQLITE_ABORT_ROLLBACK => Some(ErrorCode::Aborted(AbortError::Rollback)),

            // Auth errors
            SQLITE_AUTH_USER => Some(ErrorCode::Authorization(AuthorizationError::User)),

            // Busy errors
            SQLITE_BUSY_RECOVERY => Some(ErrorCode::Busy(BusyError::Recovery)),
            SQLITE_BUSY_SNAPSHOT => Some(ErrorCode::Busy(BusyError::Snapshot)),
            SQLITE_BUSY_TIMEOUT => Some(ErrorCode::Busy(BusyError::Timeout)),

            // CantOpen errors
            SQLITE_CANTOPEN_FULLPATH => Some(ErrorCode::CantOpen(CantOpenError::FullPath)),
            SQLITE_CANTOPEN_ISDIR => Some(ErrorCode::CantOpen(CantOpenError::IsDir)),
            SQLITE_CANTOPEN_NOTEMPDIR => Some(ErrorCode::CantOpen(CantOpenError::NoTempDir)),
            SQLITE_CANTOPEN_CONVPATH => Some(ErrorCode::CantOpen(CantOpenError::ConvPath)),
            SQLITE_CANTOPEN_DIRTYWAL => Some(ErrorCode::CantOpen(CantOpenError::DirtyWal)),
            SQLITE_CANTOPEN_SYMLINK => Some(ErrorCode::CantOpen(CantOpenError::Symlink)),

            // Constraint errors
            SQLITE_CONSTRAINT_CHECK => Some(ErrorCode::Constraint(ConstraintError::Check)),
            SQLITE_CONSTRAINT_COMMITHOOK => {
                Some(ErrorCode::Constraint(ConstraintError::CommitHook))
            }
            SQLITE_CONSTRAINT_DATATYPE => Some(ErrorCode::Constraint(ConstraintError::DataType)),
            SQLITE_CONSTRAINT_FOREIGNKEY => {
                Some(ErrorCode::Constraint(ConstraintError::ForeignKey))
            }
            SQLITE_CONSTRAINT_FUNCTION => Some(ErrorCode::Constraint(ConstraintError::Function)),
            SQLITE_CONSTRAINT_NOTNULL => Some(ErrorCode::Constraint(ConstraintError::NotNull)),
            SQLITE_CONSTRAINT_PINNED => Some(ErrorCode::Constraint(ConstraintError::Pinned)),
            SQLITE_CONSTRAINT_PRIMARYKEY => {
                Some(ErrorCode::Constraint(ConstraintError::PrimaryKey))
            }
            SQLITE_CONSTRAINT_ROWID => Some(ErrorCode::Constraint(ConstraintError::RowId)),
            SQLITE_CONSTRAINT_TRIGGER => Some(ErrorCode::Constraint(ConstraintError::Trigger)),
            SQLITE_CONSTRAINT_UNIQUE => Some(ErrorCode::Constraint(ConstraintError::Unique)),
            SQLITE_CONSTRAINT_VTAB => Some(ErrorCode::Constraint(ConstraintError::VTab)),

            // Corrupt errors
            SQLITE_CORRUPT_INDEX => Some(ErrorCode::Corrupt(CorruptError::Index)),
            SQLITE_CORRUPT_SEQUENCE => Some(ErrorCode::Corrupt(CorruptError::Sequence)),
            SQLITE_CORRUPT_VTAB => Some(ErrorCode::Corrupt(CorruptError::VTab)),

            // General errors
            SQLITE_ERROR_MISSING_COLLSEQ => Some(ErrorCode::Error(GeneralError::MissingCollSeq)),
            SQLITE_ERROR_RETRY => Some(ErrorCode::Error(GeneralError::Retry)),
            SQLITE_ERROR_SNAPSHOT => Some(ErrorCode::Error(GeneralError::Snapshot)),

            // IO errors
            SQLITE_IOERR_READ => Some(ErrorCode::Io(IoError::Read)),
            SQLITE_IOERR_WRITE => Some(ErrorCode::Io(IoError::Write)),
            SQLITE_IOERR_FSYNC => Some(ErrorCode::Io(IoError::FSync)),
            SQLITE_IOERR_FSTAT => Some(ErrorCode::Io(IoError::FStat)),
            SQLITE_IOERR_TRUNCATE => Some(ErrorCode::Io(IoError::Truncate)),
            SQLITE_IOERR_UNLOCK => Some(ErrorCode::Io(IoError::Unlock)),
            SQLITE_IOERR_RDLOCK => Some(ErrorCode::Io(IoError::ReadLock)),
            SQLITE_IOERR_DELETE => Some(ErrorCode::Io(IoError::Delete)),
            SQLITE_IOERR_BLOCKED => Some(ErrorCode::Io(IoError::Blocked)),
            SQLITE_IOERR_NOMEM => Some(ErrorCode::Io(IoError::NoMem)),
            SQLITE_IOERR_ACCESS => Some(ErrorCode::Io(IoError::Access)),
            SQLITE_IOERR_CHECKRESERVEDLOCK => Some(ErrorCode::Io(IoError::CheckReservedLock)),
            SQLITE_IOERR_LOCK => Some(ErrorCode::Io(IoError::Lock)),
            SQLITE_IOERR_CLOSE => Some(ErrorCode::Io(IoError::Close)),
            SQLITE_IOERR_DIR_CLOSE => Some(ErrorCode::Io(IoError::DirClose)),
            SQLITE_IOERR_SHMOPEN => Some(ErrorCode::Io(IoError::ShmOpen)),
            SQLITE_IOERR_SHMSIZE => Some(ErrorCode::Io(IoError::ShmSize)),
            SQLITE_IOERR_SHMLOCK => Some(ErrorCode::Io(IoError::ShmLock)),
            SQLITE_IOERR_SHMMAP => Some(ErrorCode::Io(IoError::ShmMap)),
            SQLITE_IOERR_SEEK => Some(ErrorCode::Io(IoError::Seek)),
            SQLITE_IOERR_DELETE_NOENT => Some(ErrorCode::Io(IoError::DeleteNoEnt)),
            SQLITE_IOERR_MMAP => Some(ErrorCode::Io(IoError::MMap)),
            SQLITE_IOERR_GETTEMPPATH => Some(ErrorCode::Io(IoError::GetTempPath)),
            SQLITE_IOERR_CONVPATH => Some(ErrorCode::Io(IoError::ConvPath)),
            SQLITE_IOERR_VNODE => Some(ErrorCode::Io(IoError::VNode)),
            SQLITE_IOERR_AUTH => Some(ErrorCode::Io(IoError::Auth)),
            SQLITE_IOERR_BEGIN_ATOMIC => Some(ErrorCode::Io(IoError::BeginAtomic)),
            SQLITE_IOERR_COMMIT_ATOMIC => Some(ErrorCode::Io(IoError::CommitAtomic)),
            SQLITE_IOERR_ROLLBACK_ATOMIC => Some(ErrorCode::Io(IoError::RollbackAtomic)),
            SQLITE_IOERR_DATA => Some(ErrorCode::Io(IoError::Data)),
            SQLITE_IOERR_CORRUPTFS => Some(ErrorCode::Io(IoError::CorruptFS)),
            SQLITE_IOERR_SHORT_READ => Some(ErrorCode::Io(IoError::ShortRead)),
            SQLITE_IOERR_DIR_FSYNC => Some(ErrorCode::Io(IoError::DirFSync)),

            // Locked errors
            SQLITE_LOCKED_SHAREDCACHE => Some(ErrorCode::Locked(LockedError::SharedCache)),
            SQLITE_LOCKED_VTAB => Some(ErrorCode::Locked(LockedError::VTab)),

            // ReadOnly errors
            SQLITE_READONLY_RECOVERY => Some(ErrorCode::ReadOnly(ReadOnlyError::Recovery)),
            SQLITE_READONLY_CANTLOCK => Some(ErrorCode::ReadOnly(ReadOnlyError::CantLock)),
            SQLITE_READONLY_ROLLBACK => Some(ErrorCode::ReadOnly(ReadOnlyError::Rollback)),
            SQLITE_READONLY_DBMOVED => Some(ErrorCode::ReadOnly(ReadOnlyError::DbMoved)),
            SQLITE_READONLY_CANTINIT => Some(ErrorCode::ReadOnly(ReadOnlyError::CantInit)),
            SQLITE_READONLY_DIRECTORY => Some(ErrorCode::ReadOnly(ReadOnlyError::Directory)),

            _ => None,
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = unsafe {
            let ptr = sqlite3_errstr(self.into_inner());
            CStr::from_ptr(ptr).to_str().unwrap_unchecked()
        };
        f.debug_tuple("Error")
            .field(&message)
            .field(&self.code)
            .finish()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = unsafe {
            let ptr = sqlite3_errstr(self.into_inner());
            CStr::from_ptr(ptr).to_str().unwrap_unchecked()
        };
        write!(f, "{message}")
    }
}

impl Default for Error {
    fn default() -> Self {
        Self::unknown()
    }
}

impl core::error::Error for Error {}

impl From<c_int> for Error {
    fn from(value: c_int) -> Self {
        Error::new(value).unwrap_or_default()
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
#[repr(i32)]
pub enum ErrorCategory {
    /// A generic error code that returned by SQLite when no other more specific
    /// error code is available.
    #[doc(alias = "SQLITE_ERROR")]
    Unknown = SQLITE_ERROR,

    /// Indicates an internal error (bug) in SQLite, an application-defined
    /// function, virtual table, VFS, or extension.
    #[doc(alias = "SQLITE_INTERNAL")]
    Internal = SQLITE_INTERNAL,

    /// The requested access mode for a newly created database could not be
    /// provided.
    #[doc(alias = "SQLITE_PERM")]
    Permission = SQLITE_PERM,

    /// An operation was aborted prior to completion, usually by application
    /// request.
    #[doc(alias = "SQLITE_ABORT")]
    Aborted = SQLITE_ABORT,

    /// The database file could not be written (or in some cases read) because
    /// of concurrent activity by some other database connection.
    #[doc(alias = "SQLITE_BUSY")]
    Busy = SQLITE_BUSY,

    /// A write operation could not continue because of a conflict within the
    /// same database connection or a conflict with a different database
    /// connection that uses a shared cache.
    #[doc(alias = "SQLITE_LOCKED")]
    Locked = SQLITE_LOCKED,

    /// SQLite was unable to allocate all the memory it needed to complete
    /// the operation.
    #[doc(alias = "SQLITE_NOMEM")]
    OutOfMemory = SQLITE_NOMEM,

    /// An attempt is made to alter some data for which the current database
    /// connection does not have write permission.
    #[doc(alias = "SQLITE_READONLY")]
    ReadOnly = SQLITE_READONLY,

    /// An operation was interrupted by the sqlite3_interrupt() interface.
    #[doc(alias = "SQLITE_INTERRUPT")]
    Interrupt = SQLITE_INTERRUPT,

    /// The operation could not finish because the operating system reported
    /// an I/O error.
    #[doc(alias = "SQLITE_IOERR")]
    Io = SQLITE_IOERR,

    /// The database file has been corrupted.
    #[doc(alias = "SQLITE_CORRUPT")]
    Corrupt = SQLITE_CORRUPT,

    /// Used internally by SQLite, not typically exposed to applications.
    #[doc(alias = "SQLITE_NOTFOUND")]
    NotFound = SQLITE_NOTFOUND,

    /// A write could not complete because the disk is full.
    #[doc(alias = "SQLITE_FULL")]
    Full = SQLITE_FULL,

    /// SQLite was unable to open a file.
    #[doc(alias = "SQLITE_CANTOPEN")]
    CantOpen = SQLITE_CANTOPEN,

    /// A problem with the file locking protocol used by SQLite.
    #[doc(alias = "SQLITE_PROTOCOL")]
    Protocol = SQLITE_PROTOCOL,

    /// Not currently used by SQLite.
    #[doc(alias = "SQLITE_EMPTY")]
    #[deprecated]
    Empty = SQLITE_EMPTY,

    /// The database schema has changed.
    #[doc(alias = "SQLITE_SCHEMA")]
    Schema = SQLITE_SCHEMA,

    /// A string or BLOB was too large.
    #[doc(alias = "SQLITE_TOOBIG")]
    TooBig = SQLITE_TOOBIG,

    /// An SQL constraint violation occurred.
    #[doc(alias = "SQLITE_CONSTRAINT")]
    Constraint = SQLITE_CONSTRAINT,

    /// A datatype mismatch occurred.
    #[doc(alias = "SQLITE_MISMATCH")]
    Mismatch = SQLITE_MISMATCH,

    /// The application uses any SQLite interface in a way that is undefined
    /// or unsupported.
    #[doc(alias = "SQLITE_MISUSE")]
    Misuse = SQLITE_MISUSE,

    /// The system does not support large files when the database grows to be
    /// larger than what the filesystem can handle.
    #[doc(alias = "SQLITE_NOLFS")]
    LargeFile = SQLITE_NOLFS,

    /// The authorizer callback indicates that an SQL statement being prepared
    /// is not authorized.
    #[doc(alias = "SQLITE_AUTH")]
    Authorization = SQLITE_AUTH,

    /// Not currently used by SQLite.
    #[doc(alias = "SQLITE_FORMAT")]
    #[deprecated]
    Format = SQLITE_FORMAT,

    /// The parameter number argument to one of the sqlite3_bind routines or
    /// the column number in one of the sqlite3_column routines is out of range.
    #[doc(alias = "SQLITE_RANGE")]
    Range = SQLITE_RANGE,

    /// When attempting to open a file, the file being opened does not appear
    /// to be an SQLite database file.
    #[doc(alias = "SQLITE_NOTADB")]
    InvalidDatabase = SQLITE_NOTADB,
}

/// Extended SQLite result codes that provide more specific information about errors.
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum ErrorCode {
    /// Extended abort error codes.
    Aborted(AbortError),

    /// Extended authorization error codes.
    Authorization(AuthorizationError),

    /// Extended busy error codes.
    Busy(BusyError),

    /// Extended "can't open" error codes.
    CantOpen(CantOpenError),

    /// Extended constraint error codes.
    Constraint(ConstraintError),

    /// Extended corruption error codes.
    Corrupt(CorruptError),

    /// Extended general error codes.
    Error(GeneralError),

    /// Extended I/O error codes.
    Io(IoError),

    /// Extended locked error codes.
    Locked(LockedError),

    /// Extended read-only error codes.
    ReadOnly(ReadOnlyError),
}

impl ErrorCode {
    /// Returns the numeric value of this error code.
    pub fn code(self) -> i32 {
        match self {
            ErrorCode::Aborted(err) => err as i32,
            ErrorCode::Authorization(err) => err as i32,
            ErrorCode::Busy(err) => err as i32,
            ErrorCode::CantOpen(err) => err as i32,
            ErrorCode::Constraint(err) => err as i32,
            ErrorCode::Corrupt(err) => err as i32,
            ErrorCode::Error(err) => err as i32,
            ErrorCode::Io(err) => err as i32,
            ErrorCode::Locked(err) => err as i32,
            ErrorCode::ReadOnly(err) => err as i32,
        }
    }

    /// Returns the primary error category for this extended error code.
    pub fn primary_category(self) -> ErrorCategory {
        match self {
            ErrorCode::Aborted(_) => ErrorCategory::Aborted,
            ErrorCode::Authorization(_) => ErrorCategory::Authorization,
            ErrorCode::Busy(_) => ErrorCategory::Busy,
            ErrorCode::CantOpen(_) => ErrorCategory::CantOpen,
            ErrorCode::Constraint(_) => ErrorCategory::Constraint,
            ErrorCode::Corrupt(_) => ErrorCategory::Corrupt,
            ErrorCode::Error(_) => ErrorCategory::Unknown,
            ErrorCode::Io(_) => ErrorCategory::Io,
            ErrorCode::Locked(_) => ErrorCategory::Locked,
            ErrorCode::ReadOnly(_) => ErrorCategory::ReadOnly,
        }
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
#[repr(i32)]
pub enum AbortError {
    /// An SQL statement aborted because the transaction that was active when
    /// the SQL statement first started was rolled back.
    #[doc(alias = "SQLITE_ABORT_ROLLBACK")]
    Rollback = SQLITE_ABORT_ROLLBACK,
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
#[repr(i32)]
pub enum AuthorizationError {
    /// An operation was attempted on a database for which the logged in user
    /// lacks sufficient authorization.
    #[doc(alias = "SQLITE_AUTH_USER")]
    User = SQLITE_AUTH_USER,
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
#[repr(i32)]
pub enum BusyError {
    /// An operation could not continue because another process is busy
    /// recovering a WAL mode database file following a crash.
    #[doc(alias = "SQLITE_BUSY_RECOVERY")]
    Recovery = SQLITE_BUSY_RECOVERY,

    /// A database connection tries to promote a read transaction into a write
    /// transaction but finds that another database connection has already
    /// written to the database.
    #[doc(alias = "SQLITE_BUSY_SNAPSHOT")]
    Snapshot = SQLITE_BUSY_SNAPSHOT,

    /// A blocking Posix advisory file lock request in the VFS layer failed
    /// due to a timeout.
    #[doc(alias = "SQLITE_BUSY_TIMEOUT")]
    Timeout = SQLITE_BUSY_TIMEOUT,
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
#[repr(i32)]
pub enum CantOpenError {
    /// The operating system was unable to convert the filename into a full pathname.
    #[doc(alias = "SQLITE_CANTOPEN_FULLPATH")]
    FullPath = SQLITE_CANTOPEN_FULLPATH,

    /// A file open operation failed because the file is really a directory.
    #[doc(alias = "SQLITE_CANTOPEN_ISDIR")]
    IsDir = SQLITE_CANTOPEN_ISDIR,

    /// No longer used.
    #[doc(alias = "SQLITE_CANTOPEN_NOTEMPDIR")]
    #[deprecated]
    NoTempDir = SQLITE_CANTOPEN_NOTEMPDIR,

    /// Used only by Cygwin VFS indicating that the cygwin_conv_path() system
    /// call failed while trying to open a file.
    #[doc(alias = "SQLITE_CANTOPEN_CONVPATH")]
    ConvPath = SQLITE_CANTOPEN_CONVPATH,

    /// Not used at this time.
    #[doc(alias = "SQLITE_CANTOPEN_DIRTYWAL")]
    #[deprecated]
    DirtyWal = SQLITE_CANTOPEN_DIRTYWAL,

    /// The database file is a symbolic link and SQLITE_OPEN_NOFOLLOW flag was used.
    #[doc(alias = "SQLITE_CANTOPEN_SYMLINK")]
    Symlink = SQLITE_CANTOPEN_SYMLINK,
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
#[repr(i32)]
pub enum ConstraintError {
    /// A CHECK constraint failed.
    #[doc(alias = "SQLITE_CONSTRAINT_CHECK")]
    Check = SQLITE_CONSTRAINT_CHECK,

    /// A commit hook callback returned non-zero that thus caused the SQL
    /// statement to be rolled back.
    #[doc(alias = "SQLITE_CONSTRAINT_COMMITHOOK")]
    CommitHook = SQLITE_CONSTRAINT_COMMITHOOK,

    /// An insert or update attempted to store a value inconsistent with the
    /// column's declared type in a table defined as STRICT.
    #[doc(alias = "SQLITE_CONSTRAINT_DATATYPE")]
    DataType = SQLITE_CONSTRAINT_DATATYPE,

    /// A foreign key constraint failed.
    #[doc(alias = "SQLITE_CONSTRAINT_FOREIGNKEY")]
    ForeignKey = SQLITE_CONSTRAINT_FOREIGNKEY,

    /// Available for use by extension functions.
    #[doc(alias = "SQLITE_CONSTRAINT_FUNCTION")]
    Function = SQLITE_CONSTRAINT_FUNCTION,

    /// A NOT NULL constraint failed.
    #[doc(alias = "SQLITE_CONSTRAINT_NOTNULL")]
    NotNull = SQLITE_CONSTRAINT_NOTNULL,

    /// An UPDATE trigger attempted to delete the row that was being updated
    /// in the middle of the update.
    #[doc(alias = "SQLITE_CONSTRAINT_PINNED")]
    Pinned = SQLITE_CONSTRAINT_PINNED,

    /// A PRIMARY KEY constraint failed.
    #[doc(alias = "SQLITE_CONSTRAINT_PRIMARYKEY")]
    PrimaryKey = SQLITE_CONSTRAINT_PRIMARYKEY,

    /// A rowid is not unique.
    #[doc(alias = "SQLITE_CONSTRAINT_ROWID")]
    RowId = SQLITE_CONSTRAINT_ROWID,

    /// A RAISE function within a trigger fired, causing the SQL statement to abort.
    #[doc(alias = "SQLITE_CONSTRAINT_TRIGGER")]
    Trigger = SQLITE_CONSTRAINT_TRIGGER,

    /// A UNIQUE constraint failed.
    #[doc(alias = "SQLITE_CONSTRAINT_UNIQUE")]
    Unique = SQLITE_CONSTRAINT_UNIQUE,

    /// Available for use by application-defined virtual tables.
    #[doc(alias = "SQLITE_CONSTRAINT_VTAB")]
    VTab = SQLITE_CONSTRAINT_VTAB,
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
#[repr(i32)]
pub enum CorruptError {
    /// SQLite detected an entry is or was missing from an index.
    #[doc(alias = "SQLITE_CORRUPT_INDEX")]
    Index = SQLITE_CORRUPT_INDEX,

    /// The schema of the sqlite_sequence table is corrupt.
    #[doc(alias = "SQLITE_CORRUPT_SEQUENCE")]
    Sequence = SQLITE_CORRUPT_SEQUENCE,

    /// Used by virtual tables. A virtual table might return this to indicate
    /// that content in the virtual table is corrupt.
    #[doc(alias = "SQLITE_CORRUPT_VTAB")]
    VTab = SQLITE_CORRUPT_VTAB,
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
#[repr(i32)]
pub enum GeneralError {
    /// An SQL statement could not be prepared because a collating sequence
    /// named in that SQL statement could not be located.
    #[doc(alias = "SQLITE_ERROR_MISSING_COLLSEQ")]
    MissingCollSeq = SQLITE_ERROR_MISSING_COLLSEQ,

    /// Used internally to provoke sqlite3_prepare_v2() to try again to prepare
    /// a statement that failed with an error on the previous attempt.
    #[doc(alias = "SQLITE_ERROR_RETRY")]
    Retry = SQLITE_ERROR_RETRY,

    /// Returned when attempting to start a read transaction on an historical
    /// version of the database by using sqlite3_snapshot_open() interface.
    #[doc(alias = "SQLITE_ERROR_SNAPSHOT")]
    Snapshot = SQLITE_ERROR_SNAPSHOT,
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
#[repr(i32)]
pub enum IoError {
    /// I/O error while trying to read from a file on disk.
    #[doc(alias = "SQLITE_IOERR_READ")]
    Read = SQLITE_IOERR_READ,

    /// I/O error while trying to write into a file on disk.
    #[doc(alias = "SQLITE_IOERR_WRITE")]
    Write = SQLITE_IOERR_WRITE,

    /// I/O error while trying to flush previously written content out of OS
    /// and/or disk-control buffers and into persistent storage.
    #[doc(alias = "SQLITE_IOERR_FSYNC")]
    FSync = SQLITE_IOERR_FSYNC,

    /// I/O error while trying to invoke fstat() on a file to determine
    /// information such as the file size or access permissions.
    #[doc(alias = "SQLITE_IOERR_FSTAT")]
    FStat = SQLITE_IOERR_FSTAT,

    /// I/O error while trying to truncate a file to a smaller size.
    #[doc(alias = "SQLITE_IOERR_TRUNCATE")]
    Truncate = SQLITE_IOERR_TRUNCATE,

    /// I/O error within the xUnlock method on the sqlite3_io_methods object.
    #[doc(alias = "SQLITE_IOERR_UNLOCK")]
    Unlock = SQLITE_IOERR_UNLOCK,

    /// I/O error within the xLock method while trying to obtain a read lock.
    #[doc(alias = "SQLITE_IOERR_RDLOCK")]
    ReadLock = SQLITE_IOERR_RDLOCK,

    /// I/O error within the xDelete method on the sqlite3_vfs object.
    #[doc(alias = "SQLITE_IOERR_DELETE")]
    Delete = SQLITE_IOERR_DELETE,

    /// No longer used.
    #[doc(alias = "SQLITE_IOERR_BLOCKED")]
    #[deprecated]
    Blocked = SQLITE_IOERR_BLOCKED,

    /// Sometimes returned by the VFS layer to indicate that an operation
    /// could not be completed due to the inability to allocate sufficient memory.
    #[doc(alias = "SQLITE_IOERR_NOMEM")]
    NoMem = SQLITE_IOERR_NOMEM,

    /// I/O error within the xAccess method on the sqlite3_vfs object.
    #[doc(alias = "SQLITE_IOERR_ACCESS")]
    Access = SQLITE_IOERR_ACCESS,

    /// I/O error within the xCheckReservedLock method on the sqlite3_io_methods object.
    #[doc(alias = "SQLITE_IOERR_CHECKRESERVEDLOCK")]
    CheckReservedLock = SQLITE_IOERR_CHECKRESERVEDLOCK,

    /// I/O error in the advisory file locking logic.
    #[doc(alias = "SQLITE_IOERR_LOCK")]
    Lock = SQLITE_IOERR_LOCK,

    /// I/O error within the xClose method on the sqlite3_io_methods object.
    #[doc(alias = "SQLITE_IOERR_CLOSE")]
    Close = SQLITE_IOERR_CLOSE,

    /// No longer used.
    #[doc(alias = "SQLITE_IOERR_DIR_CLOSE")]
    #[deprecated]
    DirClose = SQLITE_IOERR_DIR_CLOSE,

    /// I/O error within the xShmMap method while trying to open a new shared
    /// memory segment.
    #[doc(alias = "SQLITE_IOERR_SHMOPEN")]
    ShmOpen = SQLITE_IOERR_SHMOPEN,

    /// I/O error within the xShmMap method while trying to enlarge a "shm"
    /// file as part of WAL mode transaction processing.
    #[doc(alias = "SQLITE_IOERR_SHMSIZE")]
    ShmSize = SQLITE_IOERR_SHMSIZE,

    /// No longer used.
    #[doc(alias = "SQLITE_IOERR_SHMLOCK")]
    #[deprecated]
    ShmLock = SQLITE_IOERR_SHMLOCK,

    /// I/O error within the xShmMap method while trying to map a shared memory
    /// segment into the process address space.
    #[doc(alias = "SQLITE_IOERR_SHMMAP")]
    ShmMap = SQLITE_IOERR_SHMMAP,

    /// I/O error within the xRead or xWrite methods while trying to seek a
    /// file descriptor to the beginning point of the file.
    #[doc(alias = "SQLITE_IOERR_SEEK")]
    Seek = SQLITE_IOERR_SEEK,

    /// I/O error indicating that the xDelete method failed because the file
    /// being deleted does not exist.
    #[doc(alias = "SQLITE_IOERR_DELETE_NOENT")]
    DeleteNoEnt = SQLITE_IOERR_DELETE_NOENT,

    /// I/O error within the xFetch or xUnfetch methods while trying to map
    /// or unmap part of the database file into the process address space.
    #[doc(alias = "SQLITE_IOERR_MMAP")]
    MMap = SQLITE_IOERR_MMAP,

    /// The VFS is unable to determine a suitable directory in which to place
    /// temporary files.
    #[doc(alias = "SQLITE_IOERR_GETTEMPPATH")]
    GetTempPath = SQLITE_IOERR_GETTEMPPATH,

    /// Used only by Cygwin VFS indicating that the cygwin_conv_path() system
    /// call failed.
    #[doc(alias = "SQLITE_IOERR_CONVPATH")]
    ConvPath = SQLITE_IOERR_CONVPATH,

    /// Code reserved for use by extensions.
    #[doc(alias = "SQLITE_IOERR_VNODE")]
    VNode = SQLITE_IOERR_VNODE,

    /// Code reserved for use by extensions.
    #[doc(alias = "SQLITE_IOERR_AUTH")]
    Auth = SQLITE_IOERR_AUTH,

    /// The underlying operating system reported an error on the
    /// SQLITE_FCNTL_BEGIN_ATOMIC_WRITE file-control.
    #[doc(alias = "SQLITE_IOERR_BEGIN_ATOMIC")]
    BeginAtomic = SQLITE_IOERR_BEGIN_ATOMIC,

    /// The underlying operating system reported an error on the
    /// SQLITE_FCNTL_COMMIT_ATOMIC_WRITE file-control.
    #[doc(alias = "SQLITE_IOERR_COMMIT_ATOMIC")]
    CommitAtomic = SQLITE_IOERR_COMMIT_ATOMIC,

    /// The underlying operating system reported an error on the
    /// SQLITE_FCNTL_ROLLBACK_ATOMIC_WRITE file-control.
    #[doc(alias = "SQLITE_IOERR_ROLLBACK_ATOMIC")]
    RollbackAtomic = SQLITE_IOERR_ROLLBACK_ATOMIC,

    /// Used only by the checksum VFS shim to indicate that the checksum on
    /// a page of the database file is incorrect.
    #[doc(alias = "SQLITE_IOERR_DATA")]
    Data = SQLITE_IOERR_DATA,

    /// A seek or read failure was due to the request not falling within the
    /// file's boundary rather than an ordinary device failure.
    #[doc(alias = "SQLITE_IOERR_CORRUPTFS")]
    CorruptFS = SQLITE_IOERR_CORRUPTFS,

    /// A read attempt in the VFS layer was unable to obtain as many bytes
    /// as was requested.
    #[doc(alias = "SQLITE_IOERR_SHORT_READ")]
    ShortRead = SQLITE_IOERR_SHORT_READ,

    /// I/O error while trying to invoke fsync() on a directory.
    #[doc(alias = "SQLITE_IOERR_DIR_FSYNC")]
    DirFSync = SQLITE_IOERR_DIR_FSYNC,
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
#[repr(i32)]
pub enum LockedError {
    /// Access to an SQLite data record is blocked by another database connection
    /// that is using the same record in shared cache mode.
    #[doc(alias = "SQLITE_LOCKED_SHAREDCACHE")]
    SharedCache = SQLITE_LOCKED_SHAREDCACHE,

    /// Not used by the SQLite core, but available for use by extensions.
    #[doc(alias = "SQLITE_LOCKED_VTAB")]
    VTab = SQLITE_LOCKED_VTAB,
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
#[repr(i32)]
pub enum ReadOnlyError {
    /// A WAL mode database cannot be opened because the database file needs
    /// to be recovered and recovery requires write access but only read access
    /// is available.
    #[doc(alias = "SQLITE_READONLY_RECOVERY")]
    Recovery = SQLITE_READONLY_RECOVERY,

    /// SQLite is unable to obtain a read lock on a WAL mode database because
    /// the shared-memory file associated with that database is read-only.
    #[doc(alias = "SQLITE_READONLY_CANTLOCK")]
    CantLock = SQLITE_READONLY_CANTLOCK,

    /// A database cannot be opened because it has a hot journal that needs to
    /// be rolled back but cannot because the database is readonly.
    #[doc(alias = "SQLITE_READONLY_ROLLBACK")]
    Rollback = SQLITE_READONLY_ROLLBACK,

    /// A database cannot be modified because the database file has been moved
    /// since it was opened.
    #[doc(alias = "SQLITE_READONLY_DBMOVED")]
    DbMoved = SQLITE_READONLY_DBMOVED,

    /// The shared memory region used by WAL mode exists but its content is
    /// unreliable and unusable by the current process since the current process
    /// does not have write permission on the shared memory region.
    #[doc(alias = "SQLITE_READONLY_CANTINIT")]
    CantInit = SQLITE_READONLY_CANTINIT,

    /// The database is read-only because process does not have permission to
    /// create a journal file in the same directory as the database.
    #[doc(alias = "SQLITE_READONLY_DIRECTORY")]
    Directory = SQLITE_READONLY_DIRECTORY,
}
