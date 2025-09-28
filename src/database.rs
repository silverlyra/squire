use core::{ffi::CStr, fmt};
use std::{
    ffi::{CString, OsStr, OsString},
    path::{Path, PathBuf},
};

use sqlite::{SQLITE_OPEN_MEMORY, SQLITE_OPEN_URI};

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub(crate) enum Endpoint<L>
where
    L: AsRef<CStr> + Clone + fmt::Debug,
{
    Path(L),
    Uri(L),
    Memory(Option<L>),
}

impl<L> Endpoint<L>
where
    L: AsRef<CStr> + Clone + fmt::Debug,
{
    pub(crate) fn location(&self) -> &CStr {
        match self {
            Endpoint::Path(location) => location.as_ref(),
            Endpoint::Uri(location) => location.as_ref(),
            Endpoint::Memory(Some(location)) => location.as_ref(),
            Endpoint::Memory(None) => c"",
        }
    }

    pub(crate) fn flags(&self) -> i32 {
        match self {
            Endpoint::Path(_) => 0,
            Endpoint::Uri(_) => SQLITE_OPEN_URI,
            Endpoint::Memory(_) => SQLITE_OPEN_MEMORY,
        }
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub struct Database<L = CString>
where
    L: AsRef<CStr> + Clone + fmt::Debug,
{
    endpoint: Endpoint<L>,
}

impl Database<&'static CStr> {
    pub const fn memory() -> Self {
        Self {
            endpoint: Endpoint::Memory(None),
        }
    }

    pub fn named<L>(self, name: impl IntoLocation<Location = L>) -> Database<L>
    where
        L: AsRef<CStr> + Clone + fmt::Debug,
    {
        debug_assert!(matches!(self.endpoint(), &Endpoint::Memory(_)));

        Database {
            endpoint: Endpoint::Memory(Some(name.into_location())),
        }
    }
}

impl<L> Database<L>
where
    L: AsRef<CStr> + Clone + fmt::Debug,
{
    pub fn path(path: impl IntoLocation<Location = L>) -> Self {
        Self {
            endpoint: Endpoint::Path(path.into_location()),
        }
    }

    pub fn uri(path: impl IntoLocation<Location = L>) -> Self {
        Self {
            endpoint: Endpoint::Uri(path.into_location()),
        }
    }

    pub(crate) fn endpoint(&self) -> &Endpoint<L> {
        &self.endpoint
    }

    pub(crate) fn into_endpoint(self) -> Endpoint<L> {
        self.endpoint
    }
}

impl<L> AsRef<Database<L>> for Database<L>
where
    L: AsRef<CStr> + Clone + fmt::Debug,
{
    fn as_ref(&self) -> &Database<L> {
        self
    }
}

pub trait IntoLocation {
    type Location: AsRef<CStr> + Clone + fmt::Debug;

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
        CString::new(self).expect("no \\0 bytes in connection string")
    }
}

impl IntoLocation for String {
    type Location = CString;

    fn into_location(self) -> Self::Location {
        CString::new(self).expect("no \\0 bytes in connection string")
    }
}

impl IntoLocation for &Path {
    type Location = CString;

    fn into_location(self) -> Self::Location {
        CString::new(self.as_os_str().as_encoded_bytes())
            .expect("no \\0 bytes in connection string")
    }
}

impl IntoLocation for PathBuf {
    type Location = CString;

    fn into_location(self) -> Self::Location {
        CString::new(self.into_os_string().into_encoded_bytes())
            .expect("no \\0 bytes in connection string")
    }
}

impl IntoLocation for &OsStr {
    type Location = CString;

    fn into_location(self) -> Self::Location {
        CString::new(self.as_encoded_bytes()).expect("no \\0 bytes in connection string")
    }
}

impl IntoLocation for OsString {
    type Location = CString;

    fn into_location(self) -> Self::Location {
        CString::new(self.into_encoded_bytes()).expect("no \\0 bytes in connection string")
    }
}

#[cfg(feature = "url")]
#[cfg_attr(docsrs, doc(cfg(feature = "url")))]
impl IntoLocation for url::Url {
    type Location = CString;

    fn into_location(self) -> Self::Location {
        CString::new(self.as_str()).expect("no \\0 bytes in URL")
    }
}
