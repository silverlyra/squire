use super::code::ErrorCode;
use super::integration::IntegrationError;
use super::location::ErrorLocation;
use crate::ffi;

#[derive(Clone, Debug)]
pub(super) struct ErrorInner {
    pub(super) code: ErrorCode,
    pub(super) detail: Option<ErrorDetail>,
}

impl ErrorInner {
    #[inline]
    pub(super) const fn new(code: ErrorCode) -> Self {
        Self { code, detail: None }
    }

    #[inline]
    pub(super) const fn with_detail(code: ErrorCode, detail: ErrorDetail) -> Self {
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

impl ErrorDetail {
    #[cold]
    pub(super) fn extract(source: impl ffi::Connected, code: ErrorCode) -> Option<Self> {
        let connection = source.as_connection();
        Self::extract_from_connection(&connection, code)
    }

    #[cold]
    pub(super) fn extract_from_connection(
        connection: &ffi::Connection,
        expected_code: ErrorCode,
    ) -> Option<Self> {
        let (code, message) = unsafe { connection.last_error() };

        if code == expected_code.raw() {
            let message =
                message.map(|message| String::from_utf8_lossy(message.to_bytes()).into_owned());

            // Clear the existing error detail now that we are consuming it into a Result
            #[cfg(sqlite_has_set_error_message)]
            let _ = unsafe { connection.set_last_error(ErrorCode::ERROR.raw(), None) };

            message.map(ErrorDetail::from)
        } else {
            None
        }
    }

    #[cold]
    pub(super) fn extract_with_location(
        source: impl ffi::Connected,
        code: ErrorCode,
    ) -> Option<Self> {
        #[cfg(sqlite_has_error_offset)]
        {
            let connection = source.as_connection();
            let detail = Self::extract_from_connection(&connection, code);
            let location = ErrorLocation::new(connection.last_error_offset());

            match (detail, location) {
                (Some(ErrorDetail::Message(message)), Some(location)) => {
                    Some(ErrorDetail::SourceMessage(message, location))
                }
                (detail, _) => detail,
            }
        }

        #[cfg(not(sqlite_has_error_offset))]
        Self::extract(source, code)
    }
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
