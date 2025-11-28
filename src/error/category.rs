use super::ErrorCode;

/// The category of [`Error`](crate::Error) that occurred.
///
/// Error categories correspond to a SQLite primary [result code][], except for
/// those defined by Squire itself ([`Fetch`](Self::Fetch) and
/// [`Parameter`](Self::Parameter)).
///
/// [result code]: https://sqlite.org/rescode.html
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
#[non_exhaustive]
#[repr(i32)]
pub enum ErrorCategory {
    /// A generic error code that SQLite returns when no specific error
    /// code is available.
    #[doc(alias = "SQLITE_ERROR")]
    Unknown = sqlite::SQLITE_ERROR,

    /// Indicates an internal error (bug) in SQLite, an application-defined
    /// function, virtual table, VFS, or extension.
    #[doc(alias = "SQLITE_INTERNAL")]
    Internal = sqlite::SQLITE_INTERNAL,

    /// The requested access mode for a newly created database could not be
    /// provided.
    #[doc(alias = "SQLITE_PERM")]
    Permission = sqlite::SQLITE_PERM,

    /// An operation was aborted prior to completion, usually by application
    /// request.
    #[doc(alias = "SQLITE_ABORT")]
    Aborted = sqlite::SQLITE_ABORT,

    /// The database file could not be written (or in some cases read) because
    /// of concurrent activity by some other database connection.
    #[doc(alias = "SQLITE_BUSY")]
    Busy = sqlite::SQLITE_BUSY,

    /// A write operation could not continue because of a conflict within the
    /// same database connection or a conflict with a different database
    /// connection that uses a shared cache.
    #[doc(alias = "SQLITE_LOCKED")]
    Locked = sqlite::SQLITE_LOCKED,

    /// SQLite was unable to allocate all the memory it needed to complete
    /// the operation.
    #[doc(alias = "SQLITE_NOMEM")]
    OutOfMemory = sqlite::SQLITE_NOMEM,

    /// An attempt is made to alter some data for which the current database
    /// connection does not have write permission.
    #[doc(alias = "SQLITE_READONLY")]
    ReadOnly = sqlite::SQLITE_READONLY,

    /// An operation was interrupted by the sqlite3_interrupt() interface.
    #[doc(alias = "SQLITE_INTERRUPT")]
    Interrupt = sqlite::SQLITE_INTERRUPT,

    /// The operation could not finish because the operating system reported
    /// an I/O error.
    #[doc(alias = "SQLITE_IOERR")]
    Io = sqlite::SQLITE_IOERR,

    /// The database file has been corrupted.
    #[doc(alias = "SQLITE_CORRUPT")]
    Corrupt = sqlite::SQLITE_CORRUPT,

    /// Used internally by SQLite, not typically exposed to applications.
    #[doc(alias = "SQLITE_NOTFOUND")]
    NotFound = sqlite::SQLITE_NOTFOUND,

    /// A write could not complete because the disk is full.
    #[doc(alias = "SQLITE_FULL")]
    Full = sqlite::SQLITE_FULL,

    /// SQLite was unable to open a file.
    #[doc(alias = "SQLITE_CANTOPEN")]
    CantOpen = sqlite::SQLITE_CANTOPEN,

    /// A problem with the file locking protocol used by SQLite.
    #[doc(alias = "SQLITE_PROTOCOL")]
    Protocol = sqlite::SQLITE_PROTOCOL,

    /// Not currently used by SQLite.
    #[doc(alias = "SQLITE_EMPTY")]
    #[deprecated]
    Empty = sqlite::SQLITE_EMPTY,

    /// The database schema has changed.
    #[doc(alias = "SQLITE_SCHEMA")]
    Schema = sqlite::SQLITE_SCHEMA,

    /// A string or BLOB was too large.
    #[doc(alias = "SQLITE_TOOBIG")]
    TooBig = sqlite::SQLITE_TOOBIG,

    /// An SQL constraint violation occurred.
    #[doc(alias = "SQLITE_CONSTRAINT")]
    Constraint = sqlite::SQLITE_CONSTRAINT,

    /// A datatype mismatch occurred.
    #[doc(alias = "SQLITE_MISMATCH")]
    Mismatch = sqlite::SQLITE_MISMATCH,

    /// The application uses any SQLite interface in a way that is undefined
    /// or unsupported.
    #[doc(alias = "SQLITE_MISUSE")]
    Misuse = sqlite::SQLITE_MISUSE,

    /// The system does not support large files when the database grows to be
    /// larger than what the filesystem can handle.
    #[doc(alias = "SQLITE_NOLFS")]
    LargeFile = sqlite::SQLITE_NOLFS,

    /// The authorizer callback indicates that an SQL statement being prepared
    /// is not authorized.
    #[doc(alias = "SQLITE_AUTH")]
    Authorization = sqlite::SQLITE_AUTH,

    /// Not currently used by SQLite.
    #[doc(alias = "SQLITE_FORMAT")]
    #[deprecated]
    Format = sqlite::SQLITE_FORMAT,

    /// The parameter number argument to one of the sqlite3_bind routines or
    /// the column number in one of the sqlite3_column routines is out of range.
    #[doc(alias = "SQLITE_RANGE")]
    Range = sqlite::SQLITE_RANGE,

    /// When attempting to open a file, the file being opened does not appear
    /// to be an SQLite database file.
    #[doc(alias = "SQLITE_NOTADB")]
    InvalidDatabase = sqlite::SQLITE_NOTADB,

    /// A column value stored in SQLite could not be read into a Rust type.
    ///
    /// (This [error category](ErrorCategory) is defined by Squire; not SQLite.
    /// No SQLite [result codes][] correspond to `ErrorCategory::Fetch`.)
    ///
    /// [result codes]: https://sqlite.org/rescode.html
    Fetch = super::code::SQUIRE_ERROR_FETCH,

    /// A parameter could not be [bound](crate::Bind); the error message gives
    /// more detail about the underlying error.
    ///
    /// (This [error category](ErrorCategory) is defined by Squire; not SQLite.
    /// No SQLite [result codes][] correspond to `ErrorCategory::Parameter`.)
    ///
    /// [result codes]: https://sqlite.org/rescode.html
    Parameter = super::code::SQUIRE_ERROR_PARAMETER,

    /// A column value stored in SQLite could not be read into a Rust type.
    ///
    /// (This [error category](ErrorCategory) is defined by Squire; not SQLite.
    /// No SQLite [result codes][] correspond to `ErrorCategory::Row`.)
    ///
    /// [result codes]: https://sqlite.org/rescode.html
    Row = super::code::SQUIRE_ERROR_ROW,
}

impl ErrorCategory {
    /// Find the [`ErrorCategory`] for an [`ErrorCode`].
    pub const fn from_code(code: ErrorCode) -> Option<Self> {
        Self::from_raw_code(code.raw())
    }

    /// Find the [`ErrorCategory`] for a SQLite [result code][].
    ///
    /// [result code]: https://sqlite.org/rescode.html
    pub const fn from_raw_code(code: i32) -> Option<Self> {
        #[allow(deprecated)]
        match code & 0xFF {
            sqlite::SQLITE_ERROR => Some(Self::Unknown),
            sqlite::SQLITE_INTERNAL => Some(Self::Internal),
            sqlite::SQLITE_PERM => Some(Self::Permission),
            sqlite::SQLITE_ABORT => Some(Self::Aborted),
            sqlite::SQLITE_BUSY => Some(Self::Busy),
            sqlite::SQLITE_LOCKED => Some(Self::Locked),
            sqlite::SQLITE_NOMEM => Some(Self::OutOfMemory),
            sqlite::SQLITE_READONLY => Some(Self::ReadOnly),
            sqlite::SQLITE_INTERRUPT => Some(Self::Interrupt),
            sqlite::SQLITE_IOERR => Some(Self::Io),
            sqlite::SQLITE_CORRUPT => Some(Self::Corrupt),
            sqlite::SQLITE_NOTFOUND => Some(Self::NotFound),
            sqlite::SQLITE_FULL => Some(Self::Full),
            sqlite::SQLITE_CANTOPEN => Some(Self::CantOpen),
            sqlite::SQLITE_PROTOCOL => Some(Self::Protocol),
            sqlite::SQLITE_EMPTY => Some(Self::Empty),
            sqlite::SQLITE_SCHEMA => Some(Self::Schema),
            sqlite::SQLITE_TOOBIG => Some(Self::TooBig),
            sqlite::SQLITE_CONSTRAINT => Some(Self::Constraint),
            sqlite::SQLITE_MISMATCH => Some(Self::Mismatch),
            sqlite::SQLITE_MISUSE => Some(Self::Misuse),
            sqlite::SQLITE_NOLFS => Some(Self::LargeFile),
            sqlite::SQLITE_AUTH => Some(Self::Authorization),
            sqlite::SQLITE_FORMAT => Some(Self::Format),
            sqlite::SQLITE_RANGE => Some(Self::Range),
            sqlite::SQLITE_NOTADB => Some(Self::InvalidDatabase),
            super::code::SQUIRE_ERROR_FETCH => Some(Self::Fetch),
            super::code::SQUIRE_ERROR_PARAMETER => Some(Self::Parameter),
            super::code::SQUIRE_ERROR_ROW => Some(Self::Row),
            _ => None,
        }
    }

    /// Returns the underlying [`ErrorCode`].
    pub const fn code(self) -> ErrorCode {
        unsafe { ErrorCode::new_unchecked(self as i32) }
    }
}
