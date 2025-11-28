use super::{ErrorCategory, ErrorCode};

/// Extended SQLite result codes that provide more specific information about errors.
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
#[non_exhaustive]
pub enum ErrorReason {
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

    /// Extended [`ErrorCategory::Fetch`] error codes.
    ///
    /// (This [error code](ErrorReason) is defined by Squire; not SQLite.
    /// No SQLite [result codes][] correspond to `ErrorReason::Fetch`.)
    ///
    /// [result codes]: https://sqlite.org/rescode.html
    Fetch(FetchError),

    /// Extended [`ErrorCategory::Parameter`] error codes.
    ///
    /// (This [error code](ErrorReason) is defined by Squire; not SQLite.
    /// No SQLite [result codes][] correspond to `ErrorReason::Parameter`.)
    ///
    /// [result codes]: https://sqlite.org/rescode.html
    Parameter(ParameterError),

    /// Extended [`ErrorCategory::Row`] error codes.
    ///
    /// (This [error code](ErrorReason) is defined by Squire; not SQLite.
    /// No SQLite [result codes][] correspond to `ErrorReason::Row`.)
    ///
    /// [result codes]: https://sqlite.org/rescode.html
    Row(RowError),
}

impl ErrorReason {
    /// Returns the primary error category for this extended error code.
    pub const fn category(self) -> ErrorCategory {
        match self {
            ErrorReason::Aborted(_) => ErrorCategory::Aborted,
            ErrorReason::Authorization(_) => ErrorCategory::Authorization,
            ErrorReason::Busy(_) => ErrorCategory::Busy,
            ErrorReason::CantOpen(_) => ErrorCategory::CantOpen,
            ErrorReason::Constraint(_) => ErrorCategory::Constraint,
            ErrorReason::Corrupt(_) => ErrorCategory::Corrupt,
            ErrorReason::Error(_) => ErrorCategory::Unknown,
            ErrorReason::Io(_) => ErrorCategory::Io,
            ErrorReason::Locked(_) => ErrorCategory::Locked,
            ErrorReason::ReadOnly(_) => ErrorCategory::ReadOnly,

            ErrorReason::Fetch(_) => ErrorCategory::Fetch,
            ErrorReason::Parameter(_) => ErrorCategory::Parameter,
            ErrorReason::Row(_) => ErrorCategory::Parameter,
        }
    }

    /// Returns the underlying [`ErrorCode`].
    pub const fn code(self) -> ErrorCode {
        let code = match self {
            Self::Aborted(err) => err as i32,
            Self::Authorization(err) => err as i32,
            Self::Busy(err) => err as i32,
            Self::CantOpen(err) => err as i32,
            Self::Constraint(err) => err as i32,
            Self::Corrupt(err) => err as i32,
            Self::Error(err) => err as i32,
            Self::Io(err) => err as i32,
            Self::Locked(err) => err as i32,
            Self::ReadOnly(err) => err as i32,

            Self::Fetch(err) => err as i32,
            Self::Parameter(err) => err as i32,
            Self::Row(err) => err as i32,
        };

        unsafe { ErrorCode::new_unchecked(code) }
    }

    /// Find the [`ErrorReason`] for an [`ErrorCode`].
    pub const fn from_code(code: ErrorCode) -> Option<Self> {
        Self::from_raw_code(code.raw())
    }

    /// Find the [`ErrorReason`] for a SQLite [result code][].
    ///
    /// [result code]: https://sqlite.org/rescode.html
    pub const fn from_raw_code(code: i32) -> Option<Self> {
        #[allow(deprecated)]
        match code {
            // Abort errors
            sqlite::SQLITE_ABORT_ROLLBACK => Some(Self::Aborted(AbortError::Rollback)),

            // Authorization errors
            sqlite::SQLITE_AUTH_USER => Some(Self::Authorization(AuthorizationError::User)),

            // Busy errors
            sqlite::SQLITE_BUSY_RECOVERY => Some(Self::Busy(BusyError::Recovery)),
            sqlite::SQLITE_BUSY_SNAPSHOT => Some(Self::Busy(BusyError::Snapshot)),
            sqlite::SQLITE_BUSY_TIMEOUT => Some(Self::Busy(BusyError::Timeout)),

            // CantOpen errors
            sqlite::SQLITE_CANTOPEN_FULLPATH => Some(Self::CantOpen(CantOpenError::FullPath)),
            sqlite::SQLITE_CANTOPEN_ISDIR => Some(Self::CantOpen(CantOpenError::IsDir)),
            sqlite::SQLITE_CANTOPEN_NOTEMPDIR => Some(Self::CantOpen(CantOpenError::NoTempDir)),
            sqlite::SQLITE_CANTOPEN_CONVPATH => Some(Self::CantOpen(CantOpenError::ConvPath)),
            sqlite::SQLITE_CANTOPEN_DIRTYWAL => Some(Self::CantOpen(CantOpenError::DirtyWal)),
            sqlite::SQLITE_CANTOPEN_SYMLINK => Some(Self::CantOpen(CantOpenError::Symlink)),

            // Constraint errors
            sqlite::SQLITE_CONSTRAINT_CHECK => Some(Self::Constraint(ConstraintError::Check)),
            sqlite::SQLITE_CONSTRAINT_COMMITHOOK => {
                Some(Self::Constraint(ConstraintError::CommitHook))
            }
            sqlite::SQLITE_CONSTRAINT_DATATYPE => Some(Self::Constraint(ConstraintError::DataType)),
            sqlite::SQLITE_CONSTRAINT_FOREIGNKEY => {
                Some(Self::Constraint(ConstraintError::ForeignKey))
            }
            sqlite::SQLITE_CONSTRAINT_FUNCTION => Some(Self::Constraint(ConstraintError::Function)),
            sqlite::SQLITE_CONSTRAINT_NOTNULL => Some(Self::Constraint(ConstraintError::NotNull)),
            sqlite::SQLITE_CONSTRAINT_PINNED => Some(Self::Constraint(ConstraintError::Pinned)),
            sqlite::SQLITE_CONSTRAINT_PRIMARYKEY => {
                Some(Self::Constraint(ConstraintError::PrimaryKey))
            }
            sqlite::SQLITE_CONSTRAINT_ROWID => Some(Self::Constraint(ConstraintError::RowId)),
            sqlite::SQLITE_CONSTRAINT_TRIGGER => Some(Self::Constraint(ConstraintError::Trigger)),
            sqlite::SQLITE_CONSTRAINT_UNIQUE => Some(Self::Constraint(ConstraintError::Unique)),
            sqlite::SQLITE_CONSTRAINT_VTAB => Some(Self::Constraint(ConstraintError::VTab)),

            // Corrupt errors
            sqlite::SQLITE_CORRUPT_INDEX => Some(Self::Corrupt(CorruptError::Index)),
            sqlite::SQLITE_CORRUPT_SEQUENCE => Some(Self::Corrupt(CorruptError::Sequence)),
            sqlite::SQLITE_CORRUPT_VTAB => Some(Self::Corrupt(CorruptError::VTab)),

            // General errors
            sqlite::SQLITE_ERROR_MISSING_COLLSEQ => Some(Self::Error(GeneralError::MissingCollSeq)),
            sqlite::SQLITE_ERROR_RETRY => Some(Self::Error(GeneralError::Retry)),
            sqlite::SQLITE_ERROR_SNAPSHOT => Some(Self::Error(GeneralError::Snapshot)),

            // IO errors
            sqlite::SQLITE_IOERR_READ => Some(Self::Io(IoError::Read)),
            sqlite::SQLITE_IOERR_WRITE => Some(Self::Io(IoError::Write)),
            sqlite::SQLITE_IOERR_FSYNC => Some(Self::Io(IoError::FSync)),
            sqlite::SQLITE_IOERR_FSTAT => Some(Self::Io(IoError::FStat)),
            sqlite::SQLITE_IOERR_TRUNCATE => Some(Self::Io(IoError::Truncate)),
            sqlite::SQLITE_IOERR_UNLOCK => Some(Self::Io(IoError::Unlock)),
            sqlite::SQLITE_IOERR_RDLOCK => Some(Self::Io(IoError::ReadLock)),
            sqlite::SQLITE_IOERR_DELETE => Some(Self::Io(IoError::Delete)),
            sqlite::SQLITE_IOERR_BLOCKED => Some(Self::Io(IoError::Blocked)),
            sqlite::SQLITE_IOERR_NOMEM => Some(Self::Io(IoError::NoMem)),
            sqlite::SQLITE_IOERR_ACCESS => Some(Self::Io(IoError::Access)),
            sqlite::SQLITE_IOERR_CHECKRESERVEDLOCK => Some(Self::Io(IoError::CheckReservedLock)),
            sqlite::SQLITE_IOERR_LOCK => Some(Self::Io(IoError::Lock)),
            sqlite::SQLITE_IOERR_CLOSE => Some(Self::Io(IoError::Close)),
            sqlite::SQLITE_IOERR_DIR_CLOSE => Some(Self::Io(IoError::DirClose)),
            sqlite::SQLITE_IOERR_SHMOPEN => Some(Self::Io(IoError::ShmOpen)),
            sqlite::SQLITE_IOERR_SHMSIZE => Some(Self::Io(IoError::ShmSize)),
            sqlite::SQLITE_IOERR_SHMLOCK => Some(Self::Io(IoError::ShmLock)),
            sqlite::SQLITE_IOERR_SHMMAP => Some(Self::Io(IoError::ShmMap)),
            sqlite::SQLITE_IOERR_SEEK => Some(Self::Io(IoError::Seek)),
            sqlite::SQLITE_IOERR_DELETE_NOENT => Some(Self::Io(IoError::DeleteNoEnt)),
            sqlite::SQLITE_IOERR_MMAP => Some(Self::Io(IoError::MMap)),
            sqlite::SQLITE_IOERR_GETTEMPPATH => Some(Self::Io(IoError::GetTempPath)),
            sqlite::SQLITE_IOERR_CONVPATH => Some(Self::Io(IoError::ConvPath)),
            sqlite::SQLITE_IOERR_VNODE => Some(Self::Io(IoError::VNode)),
            sqlite::SQLITE_IOERR_AUTH => Some(Self::Io(IoError::Auth)),
            sqlite::SQLITE_IOERR_BEGIN_ATOMIC => Some(Self::Io(IoError::BeginAtomic)),
            sqlite::SQLITE_IOERR_COMMIT_ATOMIC => Some(Self::Io(IoError::CommitAtomic)),
            sqlite::SQLITE_IOERR_ROLLBACK_ATOMIC => Some(Self::Io(IoError::RollbackAtomic)),
            sqlite::SQLITE_IOERR_DATA => Some(Self::Io(IoError::Data)),
            sqlite::SQLITE_IOERR_CORRUPTFS => Some(Self::Io(IoError::CorruptFS)),
            sqlite::SQLITE_IOERR_SHORT_READ => Some(Self::Io(IoError::ShortRead)),
            sqlite::SQLITE_IOERR_DIR_FSYNC => Some(Self::Io(IoError::DirFSync)),

            // Locked errors
            sqlite::SQLITE_LOCKED_SHAREDCACHE => Some(Self::Locked(LockedError::SharedCache)),
            sqlite::SQLITE_LOCKED_VTAB => Some(Self::Locked(LockedError::VTab)),

            // ReadOnly errors
            sqlite::SQLITE_READONLY_RECOVERY => Some(Self::ReadOnly(ReadOnlyError::Recovery)),
            sqlite::SQLITE_READONLY_CANTLOCK => Some(Self::ReadOnly(ReadOnlyError::CantLock)),
            sqlite::SQLITE_READONLY_ROLLBACK => Some(Self::ReadOnly(ReadOnlyError::Rollback)),
            sqlite::SQLITE_READONLY_DBMOVED => Some(Self::ReadOnly(ReadOnlyError::DbMoved)),
            sqlite::SQLITE_READONLY_CANTINIT => Some(Self::ReadOnly(ReadOnlyError::CantInit)),
            sqlite::SQLITE_READONLY_DIRECTORY => Some(Self::ReadOnly(ReadOnlyError::Directory)),

            // Squire errors
            super::code::SQUIRE_ERROR_FETCH_PARSE => Some(Self::Fetch(FetchError::Parse)),
            super::code::SQUIRE_ERROR_FETCH_RANGE => Some(Self::Fetch(FetchError::Range)),
            super::code::SQUIRE_ERROR_PARAMETER_BIND => Some(Self::Parameter(ParameterError::Bind)),
            super::code::SQUIRE_ERROR_PARAMETER_RANGE => {
                Some(Self::Parameter(ParameterError::Range))
            }
            super::code::SQUIRE_ERROR_PARAMETER_RESOLVE => {
                Some(Self::Parameter(ParameterError::Resolve))
            }
            super::code::SQUIRE_ERROR_PARAMETER_INVALID_INDEX => {
                Some(Self::Parameter(ParameterError::InvalidIndex))
            }
            super::code::SQUIRE_ERROR_ROW_NOT_RETURNED => Some(Self::Row(RowError::NotReturned)),

            _ => None,
        }
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
#[repr(i32)]
pub enum AbortError {
    /// An SQL statement aborted because the transaction that was active when
    /// the SQL statement first started was rolled back.
    #[doc(alias = "SQLITE_ABORT_ROLLBACK")]
    Rollback = sqlite::SQLITE_ABORT_ROLLBACK,
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
#[repr(i32)]
pub enum AuthorizationError {
    /// An operation was attempted on a database for which the logged in user
    /// lacks sufficient authorization.
    #[doc(alias = "SQLITE_AUTH_USER")]
    User = sqlite::SQLITE_AUTH_USER,
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
#[repr(i32)]
pub enum BusyError {
    /// An operation could not continue because another process is busy
    /// recovering a WAL mode database file following a crash.
    #[doc(alias = "SQLITE_BUSY_RECOVERY")]
    Recovery = sqlite::SQLITE_BUSY_RECOVERY,

    /// A database connection tries to promote a read transaction into a write
    /// transaction but finds that another database connection has already
    /// written to the database.
    #[doc(alias = "SQLITE_BUSY_SNAPSHOT")]
    Snapshot = sqlite::SQLITE_BUSY_SNAPSHOT,

    /// A blocking Posix advisory file lock request in the VFS layer failed
    /// due to a timeout.
    #[doc(alias = "SQLITE_BUSY_TIMEOUT")]
    Timeout = sqlite::SQLITE_BUSY_TIMEOUT,
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
#[repr(i32)]
pub enum CantOpenError {
    /// The operating system was unable to convert the filename into a full pathname.
    #[doc(alias = "SQLITE_CANTOPEN_FULLPATH")]
    FullPath = sqlite::SQLITE_CANTOPEN_FULLPATH,

    /// A file open operation failed because the file is really a directory.
    #[doc(alias = "SQLITE_CANTOPEN_ISDIR")]
    IsDir = sqlite::SQLITE_CANTOPEN_ISDIR,

    /// No longer used.
    #[doc(alias = "SQLITE_CANTOPEN_NOTEMPDIR")]
    #[deprecated]
    NoTempDir = sqlite::SQLITE_CANTOPEN_NOTEMPDIR,

    /// Used only by Cygwin VFS indicating that the cygwin_conv_path() system
    /// call failed while trying to open a file.
    #[doc(alias = "SQLITE_CANTOPEN_CONVPATH")]
    ConvPath = sqlite::SQLITE_CANTOPEN_CONVPATH,

    /// Not used at this time.
    #[doc(alias = "SQLITE_CANTOPEN_DIRTYWAL")]
    #[deprecated]
    DirtyWal = sqlite::SQLITE_CANTOPEN_DIRTYWAL,

    /// The database file is a symbolic link and SQLITE_OPEN_NOFOLLOW flag was used.
    #[doc(alias = "SQLITE_CANTOPEN_SYMLINK")]
    Symlink = sqlite::SQLITE_CANTOPEN_SYMLINK,
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
#[repr(i32)]
pub enum ConstraintError {
    /// A CHECK constraint failed.
    #[doc(alias = "SQLITE_CONSTRAINT_CHECK")]
    Check = sqlite::SQLITE_CONSTRAINT_CHECK,

    /// A commit hook callback returned non-zero that thus caused the SQL
    /// statement to be rolled back.
    #[doc(alias = "SQLITE_CONSTRAINT_COMMITHOOK")]
    CommitHook = sqlite::SQLITE_CONSTRAINT_COMMITHOOK,

    /// An insert or update attempted to store a value inconsistent with the
    /// column's declared type in a table defined as STRICT.
    #[doc(alias = "SQLITE_CONSTRAINT_DATATYPE")]
    DataType = sqlite::SQLITE_CONSTRAINT_DATATYPE,

    /// A foreign key constraint failed.
    #[doc(alias = "SQLITE_CONSTRAINT_FOREIGNKEY")]
    ForeignKey = sqlite::SQLITE_CONSTRAINT_FOREIGNKEY,

    /// Available for use by extension functions.
    #[doc(alias = "SQLITE_CONSTRAINT_FUNCTION")]
    Function = sqlite::SQLITE_CONSTRAINT_FUNCTION,

    /// A NOT NULL constraint failed.
    #[doc(alias = "SQLITE_CONSTRAINT_NOTNULL")]
    NotNull = sqlite::SQLITE_CONSTRAINT_NOTNULL,

    /// An UPDATE trigger attempted to delete the row that was being updated
    /// in the middle of the update.
    #[doc(alias = "SQLITE_CONSTRAINT_PINNED")]
    Pinned = sqlite::SQLITE_CONSTRAINT_PINNED,

    /// A PRIMARY KEY constraint failed.
    #[doc(alias = "SQLITE_CONSTRAINT_PRIMARYKEY")]
    PrimaryKey = sqlite::SQLITE_CONSTRAINT_PRIMARYKEY,

    /// A rowid is not unique.
    #[doc(alias = "SQLITE_CONSTRAINT_ROWID")]
    RowId = sqlite::SQLITE_CONSTRAINT_ROWID,

    /// A RAISE function within a trigger fired, causing the SQL statement to abort.
    #[doc(alias = "SQLITE_CONSTRAINT_TRIGGER")]
    Trigger = sqlite::SQLITE_CONSTRAINT_TRIGGER,

    /// A UNIQUE constraint failed.
    #[doc(alias = "SQLITE_CONSTRAINT_UNIQUE")]
    Unique = sqlite::SQLITE_CONSTRAINT_UNIQUE,

    /// Available for use by application-defined virtual tables.
    #[doc(alias = "SQLITE_CONSTRAINT_VTAB")]
    VTab = sqlite::SQLITE_CONSTRAINT_VTAB,
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
#[repr(i32)]
pub enum CorruptError {
    /// SQLite detected an entry is or was missing from an index.
    #[doc(alias = "SQLITE_CORRUPT_INDEX")]
    Index = sqlite::SQLITE_CORRUPT_INDEX,

    /// The schema of the sqlite_sequence table is corrupt.
    #[doc(alias = "SQLITE_CORRUPT_SEQUENCE")]
    Sequence = sqlite::SQLITE_CORRUPT_SEQUENCE,

    /// Used by virtual tables. A virtual table might return this to indicate
    /// that content in the virtual table is corrupt.
    #[doc(alias = "SQLITE_CORRUPT_VTAB")]
    VTab = sqlite::SQLITE_CORRUPT_VTAB,
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
#[repr(i32)]
pub enum GeneralError {
    /// An SQL statement could not be prepared because a collating sequence
    /// named in that SQL statement could not be located.
    #[doc(alias = "SQLITE_ERROR_MISSING_COLLSEQ")]
    MissingCollSeq = sqlite::SQLITE_ERROR_MISSING_COLLSEQ,

    /// Used internally to provoke sqlite3_prepare_v2() to try again to prepare
    /// a statement that failed with an error on the previous attempt.
    #[doc(alias = "SQLITE_ERROR_RETRY")]
    Retry = sqlite::SQLITE_ERROR_RETRY,

    /// Returned when attempting to start a read transaction on an historical
    /// version of the database by using sqlite3_snapshot_open() interface.
    #[doc(alias = "SQLITE_ERROR_SNAPSHOT")]
    Snapshot = sqlite::SQLITE_ERROR_SNAPSHOT,
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
#[repr(i32)]
pub enum IoError {
    /// I/O error while trying to read from a file on disk.
    #[doc(alias = "SQLITE_IOERR_READ")]
    Read = sqlite::SQLITE_IOERR_READ,

    /// I/O error while trying to write into a file on disk.
    #[doc(alias = "SQLITE_IOERR_WRITE")]
    Write = sqlite::SQLITE_IOERR_WRITE,

    /// I/O error while trying to flush previously written content out of OS
    /// and/or disk-control buffers and into persistent storage.
    #[doc(alias = "SQLITE_IOERR_FSYNC")]
    FSync = sqlite::SQLITE_IOERR_FSYNC,

    /// I/O error while trying to invoke fstat() on a file to determine
    /// information such as the file size or access permissions.
    #[doc(alias = "SQLITE_IOERR_FSTAT")]
    FStat = sqlite::SQLITE_IOERR_FSTAT,

    /// I/O error while trying to truncate a file to a smaller size.
    #[doc(alias = "SQLITE_IOERR_TRUNCATE")]
    Truncate = sqlite::SQLITE_IOERR_TRUNCATE,

    /// I/O error within the xUnlock method on the sqlite3_io_methods object.
    #[doc(alias = "SQLITE_IOERR_UNLOCK")]
    Unlock = sqlite::SQLITE_IOERR_UNLOCK,

    /// I/O error within the xLock method while trying to obtain a read lock.
    #[doc(alias = "SQLITE_IOERR_RDLOCK")]
    ReadLock = sqlite::SQLITE_IOERR_RDLOCK,

    /// I/O error within the xDelete method on the sqlite3_vfs object.
    #[doc(alias = "SQLITE_IOERR_DELETE")]
    Delete = sqlite::SQLITE_IOERR_DELETE,

    /// No longer used.
    #[doc(alias = "SQLITE_IOERR_BLOCKED")]
    #[deprecated]
    Blocked = sqlite::SQLITE_IOERR_BLOCKED,

    /// Sometimes returned by the VFS layer to indicate that an operation
    /// could not be completed due to the inability to allocate sufficient memory.
    #[doc(alias = "SQLITE_IOERR_NOMEM")]
    NoMem = sqlite::SQLITE_IOERR_NOMEM,

    /// I/O error within the xAccess method on the sqlite3_vfs object.
    #[doc(alias = "SQLITE_IOERR_ACCESS")]
    Access = sqlite::SQLITE_IOERR_ACCESS,

    /// I/O error within the xCheckReservedLock method on the sqlite3_io_methods object.
    #[doc(alias = "SQLITE_IOERR_CHECKRESERVEDLOCK")]
    CheckReservedLock = sqlite::SQLITE_IOERR_CHECKRESERVEDLOCK,

    /// I/O error in the advisory file locking logic.
    #[doc(alias = "SQLITE_IOERR_LOCK")]
    Lock = sqlite::SQLITE_IOERR_LOCK,

    /// I/O error within the xClose method on the sqlite3_io_methods object.
    #[doc(alias = "SQLITE_IOERR_CLOSE")]
    Close = sqlite::SQLITE_IOERR_CLOSE,

    /// No longer used.
    #[doc(alias = "SQLITE_IOERR_DIR_CLOSE")]
    #[deprecated]
    DirClose = sqlite::SQLITE_IOERR_DIR_CLOSE,

    /// I/O error within the xShmMap method while trying to open a new shared
    /// memory segment.
    #[doc(alias = "SQLITE_IOERR_SHMOPEN")]
    ShmOpen = sqlite::SQLITE_IOERR_SHMOPEN,

    /// I/O error within the xShmMap method while trying to enlarge a "shm"
    /// file as part of WAL mode transaction processing.
    #[doc(alias = "SQLITE_IOERR_SHMSIZE")]
    ShmSize = sqlite::SQLITE_IOERR_SHMSIZE,

    /// No longer used.
    #[doc(alias = "SQLITE_IOERR_SHMLOCK")]
    #[deprecated]
    ShmLock = sqlite::SQLITE_IOERR_SHMLOCK,

    /// I/O error within the xShmMap method while trying to map a shared memory
    /// segment into the process address space.
    #[doc(alias = "SQLITE_IOERR_SHMMAP")]
    ShmMap = sqlite::SQLITE_IOERR_SHMMAP,

    /// I/O error within the xRead or xWrite methods while trying to seek a
    /// file descriptor to the beginning point of the file.
    #[doc(alias = "SQLITE_IOERR_SEEK")]
    Seek = sqlite::SQLITE_IOERR_SEEK,

    /// I/O error indicating that the xDelete method failed because the file
    /// being deleted does not exist.
    #[doc(alias = "SQLITE_IOERR_DELETE_NOENT")]
    DeleteNoEnt = sqlite::SQLITE_IOERR_DELETE_NOENT,

    /// I/O error within the xFetch or xUnfetch methods while trying to map
    /// or unmap part of the database file into the process address space.
    #[doc(alias = "SQLITE_IOERR_MMAP")]
    MMap = sqlite::SQLITE_IOERR_MMAP,

    /// The VFS is unable to determine a suitable directory in which to place
    /// temporary files.
    #[doc(alias = "SQLITE_IOERR_GETTEMPPATH")]
    GetTempPath = sqlite::SQLITE_IOERR_GETTEMPPATH,

    /// Used only by Cygwin VFS indicating that the cygwin_conv_path() system
    /// call failed.
    #[doc(alias = "SQLITE_IOERR_CONVPATH")]
    ConvPath = sqlite::SQLITE_IOERR_CONVPATH,

    /// Code reserved for use by extensions.
    #[doc(alias = "SQLITE_IOERR_VNODE")]
    VNode = sqlite::SQLITE_IOERR_VNODE,

    /// Code reserved for use by extensions.
    #[doc(alias = "SQLITE_IOERR_AUTH")]
    Auth = sqlite::SQLITE_IOERR_AUTH,

    /// The underlying operating system reported an error on the
    /// SQLITE_FCNTL_BEGIN_ATOMIC_WRITE file-control.
    #[doc(alias = "SQLITE_IOERR_BEGIN_ATOMIC")]
    BeginAtomic = sqlite::SQLITE_IOERR_BEGIN_ATOMIC,

    /// The underlying operating system reported an error on the
    /// SQLITE_FCNTL_COMMIT_ATOMIC_WRITE file-control.
    #[doc(alias = "SQLITE_IOERR_COMMIT_ATOMIC")]
    CommitAtomic = sqlite::SQLITE_IOERR_COMMIT_ATOMIC,

    /// The underlying operating system reported an error on the
    /// SQLITE_FCNTL_ROLLBACK_ATOMIC_WRITE file-control.
    #[doc(alias = "SQLITE_IOERR_ROLLBACK_ATOMIC")]
    RollbackAtomic = sqlite::SQLITE_IOERR_ROLLBACK_ATOMIC,

    /// Used only by the checksum VFS shim to indicate that the checksum on
    /// a page of the database file is incorrect.
    #[doc(alias = "SQLITE_IOERR_DATA")]
    Data = sqlite::SQLITE_IOERR_DATA,

    /// A seek or read failure was due to the request not falling within the
    /// file's boundary rather than an ordinary device failure.
    #[doc(alias = "SQLITE_IOERR_CORRUPTFS")]
    CorruptFS = sqlite::SQLITE_IOERR_CORRUPTFS,

    /// A read attempt in the VFS layer was unable to obtain as many bytes
    /// as was requested.
    #[doc(alias = "SQLITE_IOERR_SHORT_READ")]
    ShortRead = sqlite::SQLITE_IOERR_SHORT_READ,

    /// I/O error while trying to invoke fsync() on a directory.
    #[doc(alias = "SQLITE_IOERR_DIR_FSYNC")]
    DirFSync = sqlite::SQLITE_IOERR_DIR_FSYNC,
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
#[repr(i32)]
pub enum LockedError {
    /// Access to an SQLite data record is blocked by another database connection
    /// that is using the same record in shared cache mode.
    #[doc(alias = "SQLITE_LOCKED_SHAREDCACHE")]
    SharedCache = sqlite::SQLITE_LOCKED_SHAREDCACHE,

    /// Not used by the SQLite core, but available for use by extensions.
    #[doc(alias = "SQLITE_LOCKED_VTAB")]
    VTab = sqlite::SQLITE_LOCKED_VTAB,
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
#[repr(i32)]
pub enum ReadOnlyError {
    /// A WAL mode database cannot be opened because the database file needs
    /// to be recovered and recovery requires write access but only read access
    /// is available.
    #[doc(alias = "SQLITE_READONLY_RECOVERY")]
    Recovery = sqlite::SQLITE_READONLY_RECOVERY,

    /// SQLite is unable to obtain a read lock on a WAL mode database because
    /// the shared-memory file associated with that database is read-only.
    #[doc(alias = "SQLITE_READONLY_CANTLOCK")]
    CantLock = sqlite::SQLITE_READONLY_CANTLOCK,

    /// A database cannot be opened because it has a hot journal that needs to
    /// be rolled back but cannot because the database is readonly.
    #[doc(alias = "SQLITE_READONLY_ROLLBACK")]
    Rollback = sqlite::SQLITE_READONLY_ROLLBACK,

    /// A database cannot be modified because the database file has been moved
    /// since it was opened.
    #[doc(alias = "SQLITE_READONLY_DBMOVED")]
    DbMoved = sqlite::SQLITE_READONLY_DBMOVED,

    /// The shared memory region used by WAL mode exists but its content is
    /// unreliable and unusable by the current process since the current process
    /// does not have write permission on the shared memory region.
    #[doc(alias = "SQLITE_READONLY_CANTINIT")]
    CantInit = sqlite::SQLITE_READONLY_CANTINIT,

    /// The database is read-only because process does not have permission to
    /// create a journal file in the same directory as the database.
    #[doc(alias = "SQLITE_READONLY_DIRECTORY")]
    Directory = sqlite::SQLITE_READONLY_DIRECTORY,
}

/// An error reading a SQLite column value into its Rust type.
///
/// (This [error category](ErrorCategory) is defined by Squire; not SQLite.
/// No SQLite [result codes][] correspond to `FetchError`.)
///
/// [result codes]: https://sqlite.org/rescode.html
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
#[repr(i32)]
pub enum FetchError {
    /// [Fetching](crate::Fetch) a column value failed; the value stored in
    /// SQLite cannot be parsed into the desired type.
    Parse = super::code::SQUIRE_ERROR_FETCH_PARSE,

    /// [Fetching](crate::Fetch) a column value failed; the value stored in
    /// SQLite is out of the range the destination Rust type can represent
    /// (e.g., fetching into a `u8` a value > 255).
    Range = super::code::SQUIRE_ERROR_FETCH_RANGE,
}

/// An error passing prepared statement parameter(s) to SQLite.
///
/// (This [error category](ErrorCategory) is defined by Squire; not SQLite.
/// No SQLite [result codes][] correspond to `ParameterError`.)
///
/// [result codes]: https://sqlite.org/rescode.html
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
#[repr(i32)]
pub enum ParameterError {
    /// [Binding](crate::Bind) a parameter failed;
    /// [`into_bind_value`](crate::Bind::into_bind_value()) returned an error.
    Bind = super::code::SQUIRE_ERROR_PARAMETER_BIND,

    /// [Binding](crate::Bind) a parameter failed; the value is out of range for
    /// the SQLite data type it would be bound as. (For example, trying to bind
    /// a `u64` value that doesn't fit in an `i64`.)
    Range = super::code::SQUIRE_ERROR_PARAMETER_RANGE,

    /// [Resolving](crate::Parameters) parameter index(es) failed;
    /// [`resolve`](crate::Parameters::resolve()) returned an error.
    Resolve = super::code::SQUIRE_ERROR_PARAMETER_RESOLVE,

    /// Creating a [`BindIndex`](crate::BindIndex) failed because the input
    /// value was zero or negative.
    InvalidIndex = super::code::SQUIRE_ERROR_PARAMETER_INVALID_INDEX,
}

/// An error retrieving a row from SQLite.
///
/// (This [error category](ErrorCategory) is defined by Squire; not SQLite.
/// No SQLite [result codes][] correspond to `RowError`.)
///
/// [result codes]: https://sqlite.org/rescode.html
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
#[repr(i32)]
pub enum RowError {
    /// The query didn't return a row.
    NotReturned = super::code::SQUIRE_ERROR_ROW_NOT_RETURNED,
}
