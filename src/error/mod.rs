use core::{ffi::CStr, fmt};

use sqlite::{sqlite3, sqlite3_errmsg, sqlite3_errstr};

use crate::ffi;

mod category;
mod code;
mod integration;
mod location;
mod reason;

pub use category::ErrorCategory;
pub use code::ErrorCode;
pub use integration::{ErrorContainer, IntegrationError};
pub use location::ErrorLocation;
pub use reason::{
    AbortError, AuthorizationError, BusyError, CantOpenError, ConstraintError, CorruptError,
    ErrorReason, FetchError, GeneralError, IoError, LockedError, ParameterError, ReadOnlyError,
};

/// A [`Result`](core::result::Result) returned by Squire.
///
/// Squire‘s [`Result`] has a default `Err` type of [`Error`].
pub type Result<T, E = Error> = core::result::Result<T, E>;

/// An error returned by Squire.
///
/// `Error` encapsulates errors returned by SQLite, translating its [return codes][]
/// into general [category](ErrorCategory) and extended [code](ErrorCode).
///
/// `Error` can also represent an error encountered by Squire itself, or with any of
/// the crates it integrates with (primarily to let their types be [bound](crate::Bind)
/// as parameters and [fetched](crate::Fetch) from column values).
///
/// [return codes]: https://sqlite.org/rescode.html
#[derive(Clone)]
pub struct Error {
    inner: Box<ErrorInner>,
}

impl Error {
    #[cold]
    pub fn new(code: ErrorCode) -> Self {
        Self {
            inner: Box::new(ErrorInner::new(code)),
        }
    }

    #[cold]
    pub(crate) fn with_detail(code: ErrorCode, detail: impl Into<ErrorDetail>) -> Self {
        Self {
            inner: Box::new(ErrorInner::with_detail(code, detail.into())),
        }
    }

    #[cold]
    #[inline(never)]
    pub(crate) fn from_code(code: i32) -> Option<Self> {
        ErrorCode::new(code).map(Self::new)
    }

    #[cold]
    #[inline(never)]
    pub(crate) fn from_connection(source: impl ffi::Connected, code: i32) -> Option<Self> {
        ErrorCode::new(code).map(|code| {
            let connection = source.as_connection_ptr();

            match get_message(connection, code) {
                Some(message) => Self::with_detail(code, message),
                None => Self::new(code),
            }
        })
    }

    #[cold]
    #[inline(never)]
    pub(crate) fn from_prepare(source: impl ffi::Connected, code: i32) -> Option<Self> {
        ErrorCode::new(code).map(|code| {
            let connection = source.as_connection_ptr();

            match get_message(connection, code) {
                Some(message) => {
                    let detail = match unsafe { ErrorLocation::capture(connection) } {
                        Some(location) => ErrorDetail::SourceMessage(message, location),
                        None => ErrorDetail::Message(message),
                    };
                    Self::with_detail(code, detail)
                }
                None => Self::new(code),
            }
        })
    }

    #[allow(dead_code, unreachable_code)]
    #[cold]
    #[inline(never)]
    pub(crate) fn from_bind(source: impl Into<IntegrationError>) -> Self {
        Self::with_detail(ErrorCode::SQUIRE_PARAMETER_BIND, source.into())
    }

    #[allow(dead_code, unreachable_code)]
    #[cold]
    #[inline(never)]
    pub(crate) fn from_fetch(source: impl Into<IntegrationError>) -> Self {
        Self::with_detail(ErrorCode::SQUIRE_FETCH_PARSE, source.into())
    }

    #[cold]
    #[inline(never)]
    pub(crate) fn row_not_returned() -> Self {
        Self::new(ErrorCode::SQUIRE_ROW_NOT_RETURNED)
    }

    /// The [`ErrorCode`] identifying what error occurred.
    pub const fn code(&self) -> ErrorCode {
        self.inner.code
    }

    /// The [category](ErrorCategory) of error that occurred, or `None` if unknown.
    pub const fn category(&self) -> Option<ErrorCategory> {
        ErrorCategory::from_code(self.code())
    }

    /// The [specific reason](ErrorReason) the error occurred, or `None` if unknown.
    pub const fn reason(&self) -> Option<ErrorReason> {
        ErrorReason::from_code(self.code())
    }

    /// `true` if this error originated from within SQLite;
    /// `false` for errors originating [in Squire](Self::is_squire).
    pub const fn is_sqlite(&self) -> bool {
        self.code().is_sqlite()
    }

    /// `true` if this error originated from Squire, or a crate it
    /// [integrates](IntegrationError) with; `false` for errors
    /// originating [from SQLite](Self::is_sqlite).
    pub const fn is_squire(&self) -> bool {
        self.code().is_squire()
    }

    /// `true` if this error carries an [`IntegrationError`].
    pub const fn is_integration(&self) -> bool {
        matches!(self.detail(), Some(ErrorDetail::Integration(_)))
    }

    /// The [`IntegrationError`] returned by a crate with which Squire
    /// integrates, or `None`.
    pub const fn as_integration(&self) -> Option<&IntegrationError> {
        match self.detail() {
            Some(ErrorDetail::Integration(error)) => Some(error),
            _ => None,
        }
    }

    /// The offset in the input SQL where the error was found.
    pub const fn source_location(&self) -> Option<ErrorLocation> {
        match self.detail() {
            Some(ErrorDetail::SourceMessage(_, location)) => Some(*location),
            _ => None,
        }
    }

    const fn detail(&self) -> Option<&ErrorDetail> {
        self.inner.detail.as_ref()
    }

    const fn message(&self) -> Option<&str> {
        match self.detail() {
            Some(ErrorDetail::Message(message)) => Some(message.as_str()),
            Some(ErrorDetail::SourceMessage(message, _)) => Some(message.as_str()),
            _ => None,
        }
    }
}

impl Default for Error {
    fn default() -> Self {
        Self::new(ErrorCode::default())
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let code = self::code::ErrorCodeName(self.code());
        let message = self.message().unwrap_or_else(|| self.code().description());

        let mut tuple = f.debug_tuple("Error");

        tuple.field(&code);
        tuple.field(&message);

        if let Some(location) = self.source_location() {
            tuple.field(&location);
        }
        if let Some(integration) = self.as_integration() {
            tuple.field(&integration);
        }

        tuple.finish()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_sqlite() {
            let code = self.code();
            let message = self.message().unwrap_or_else(|| self.code().description());

            write!(f, "{message} [{code}]")
        } else {
            let description = self.code().description();

            match self.message() {
                Some(message) => write!(f, "{description}: {message}"),
                None => write!(f, "{description}"),
            }
        }
    }
}

impl core::error::Error for Error {
    fn description(&self) -> &str {
        self.code().description()
    }

    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        if let Some(integration) = self.as_integration() {
            match *integration {
                #[cfg(feature = "jiff")]
                IntegrationError::Jiff(ref error) => Some(error),
                #[cfg(all(feature = "serde", feature = "json"))]
                IntegrationError::Json(ref container) => Some(container.as_ref()),
                #[cfg(all(feature = "serde", feature = "jsonb"))]
                IntegrationError::Jsonb(ref container) => Some(container.as_ref()),
                #[cfg(feature = "url")]
                IntegrationError::Url(ref error) => Some(error),
            }
        } else {
            None
        }
    }
}

impl From<i32> for Error {
    #[cold]
    fn from(code: i32) -> Self {
        Error::from_code(code).unwrap_or_default()
    }
}

impl From<ErrorCategory> for Error {
    #[cold]
    fn from(category: ErrorCategory) -> Self {
        Error::new(category.code())
    }
}

impl From<ErrorReason> for Error {
    #[cold]
    fn from(reason: ErrorReason) -> Self {
        Error::new(reason.code())
    }
}

fn get_message(connection: *mut sqlite3, code: ErrorCode) -> Option<String> {
    let ptr = unsafe { sqlite3_errmsg(connection) };
    let static_ptr = unsafe { sqlite3_errstr(code.raw()) };

    if !ptr.is_null() && ptr != static_ptr {
        let message = unsafe { str::from_utf8_unchecked(CStr::from_ptr(ptr).to_bytes()) };
        Some(message.to_owned())
    } else {
        None
    }
}

#[derive(Clone, Debug)]
struct ErrorInner {
    code: ErrorCode,
    detail: Option<ErrorDetail>,
}

impl ErrorInner {
    fn new(code: ErrorCode) -> Self {
        Self { code, detail: None }
    }

    fn with_detail(code: ErrorCode, detail: ErrorDetail) -> Self {
        Self {
            code,
            detail: Some(detail),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) enum ErrorDetail {
    Message(String),
    SourceMessage(String, ErrorLocation),
    Integration(IntegrationError),
}

impl From<String> for ErrorDetail {
    fn from(message: String) -> Self {
        Self::Message(message)
    }
}

impl From<&str> for ErrorDetail {
    fn from(message: &str) -> Self {
        Self::Message(message.into())
    }
}

impl<E> From<E> for ErrorDetail
where
    IntegrationError: From<E>,
{
    #[allow(unreachable_code)]
    fn from(error: E) -> Self {
        Self::Integration(error.into())
    }
}
