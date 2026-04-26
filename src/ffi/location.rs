use core::{ffi::CStr, fmt};
use std::{
    ffi::{CString, OsStr, OsString},
    path::{Path, PathBuf},
};

/// A path or URI of a SQLite database to [open](super::Connection::open).
///
/// (The SQLite [API][open] requires a [null-terminated string](CStr).)
///
/// [open]: https://sqlite.org/c3ref/open.html
pub trait Location: AsRef<CStr> + Clone + fmt::Debug {}

impl<T: AsRef<CStr> + Clone + fmt::Debug> Location for T {}

/// A value which can be used as a SQLite database [`Location`].
///
/// (The SQLite [API][open] requires a [null-terminated string](CStr).)
///
/// [open]: https://sqlite.org/c3ref/open.html
pub trait IntoLocation {
    type Location: Location;

    /// Convert `self` to a [`Location`].
    fn into_location(self) -> Self::Location;
}

impl<'a> IntoLocation for &'a CStr {
    type Location = &'a CStr;

    fn into_location(self) -> Self::Location {
        self
    }
}

impl IntoLocation for CString {
    type Location = CString;

    fn into_location(self) -> Self::Location {
        self
    }
}

impl IntoLocation for &str {
    type Location = CString;

    fn into_location(self) -> Self::Location {
        allocate(self)
    }
}

impl IntoLocation for String {
    type Location = CString;

    fn into_location(self) -> Self::Location {
        allocate(self)
    }
}

impl IntoLocation for &Path {
    type Location = CString;

    fn into_location(self) -> Self::Location {
        self.as_os_str().into_location()
    }
}

impl IntoLocation for PathBuf {
    type Location = CString;

    fn into_location(self) -> Self::Location {
        self.into_os_string().into_location()
    }
}

impl IntoLocation for &OsStr {
    type Location = CString;

    fn into_location(self) -> Self::Location {
        allocate(self.as_encoded_bytes())
    }
}

impl IntoLocation for OsString {
    type Location = CString;

    fn into_location(self) -> Self::Location {
        allocate(self.into_encoded_bytes())
    }
}

#[cfg(feature = "url")]
#[cfg_attr(docsrs, doc(cfg(feature = "url")))]
impl IntoLocation for url::Url {
    type Location = CString;

    fn into_location(self) -> Self::Location {
        allocate(self.as_str())
    }
}

/// Create a [`CString`] for a [`Location`].
#[inline(always)]
fn allocate<T>(location: T) -> CString
where
    Vec<u8>: From<T>,
{
    CString::new(location).expect("no \\0 bytes in connection string")
}
