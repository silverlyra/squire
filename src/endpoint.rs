use core::{ffi::CStr, fmt, ops::Deref};
use std::ffi::CString;

use sqlite::{SQLITE_OPEN_MEMORY, SQLITE_OPEN_URI};

use crate::ffi;

/// Specifies which SQLite database to [open](crate::Connection::open).
///
/// Squire uses the [`IntoEndpoint`] trait to make many types usable as an
/// `Endpoint`, including `&str`, `String`, `PathBuf`, and `OsString`.
///
/// ```no_run
/// # use std::path::PathBuf;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use squire::{Connection, Memory, Local};
///
/// let connection = Connection::open(Memory)?;
/// let connection = Connection::open("./database.sqlite3")?;
/// let connection = Connection::open(PathBuf::from("./database.sqlite3"))?;
/// let connection = Connection::open(Local::new("./database.sqlite3"))?;
///
/// # let _: Connection = connection;
/// # Ok(())
/// # }
/// ```
pub trait Endpoint: Clone + fmt::Debug {
    /// The database location to pass to the SQLite [`open`] function.
    ///
    /// [`open`]: https://sqlite.org/c3ref/open.html
    fn location(&self) -> &CStr;

    /// Flags to pass to the SQLite [`open`] function.
    ///
    /// [`open`]: https://sqlite.org/c3ref/open.html
    fn flags(&self) -> i32 {
        0
    }

    /// The name of the virtual filesystem (VFS) to use.
    fn vfs(&self) -> Option<&CStr> {
        None
    }
}

/// An in-memory, ephemeral SQLite database [`Endpoint`].
///
/// ```rust
/// # use std::path::PathBuf;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use squire::{Connection, Memory};
///
/// let connection = Connection::open(Memory)?;
///
/// # let _: Connection = connection;
/// # Ok(())
/// # }
/// ```
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
#[cfg(sqlite_has_memory_database)]
pub struct Memory;

#[cfg(sqlite_has_memory_database)]
impl Endpoint for Memory {
    fn location(&self) -> &CStr {
        c""
    }

    fn flags(&self) -> i32 {
        SQLITE_OPEN_MEMORY
    }
}

/// A local filesystem SQLite database [`Endpoint`].
///
/// To use a `Local` endpoint, pass any string or path to
/// [`open`](crate::Connection::open).
///
/// ```no_run
/// # use std::path::PathBuf;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use squire::{Connection, Local};
///
/// let connection = Connection::open("./database.sqlite3")?;
/// let connection = Connection::open(PathBuf::from("./database.sqlite3"))?;
/// let connection = Connection::open(Local::new("./database.sqlite3"))?;
///
/// # let _: Connection = connection;
/// # Ok(())
/// # }
/// ```
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
#[repr(transparent)]
pub struct Local<L: ffi::Location = CString> {
    path: L,
}

impl<L: ffi::Location> Local<L> {
    pub fn new(path: impl ffi::IntoLocation<Location = L>) -> Self {
        Self {
            path: path.into_location(),
        }
    }
}

impl Local<&'static CStr> {
    pub const fn define(path: &'static CStr) -> Self {
        Self { path }
    }
}

impl<L: ffi::Location> Endpoint for Local<L> {
    fn location(&self) -> &CStr {
        self.path.as_ref()
    }
}

impl<L: ffi::Location> AsRef<L> for Local<L> {
    fn as_ref(&self) -> &L {
        &self.path
    }
}

impl<L: ffi::Location> Deref for Local<L> {
    type Target = L;

    fn deref(&self) -> &Self::Target {
        &self.path
    }
}

/// A SQLite [database URI][] [`Endpoint`].
///
/// [database URI]: https://sqlite.org/uri.html
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
#[repr(transparent)]
pub struct Uri<L: ffi::Location = CString> {
    uri: L,
}

impl<L: ffi::Location> Uri<L> {
    pub fn new(uri: impl ffi::IntoLocation<Location = L>) -> Self {
        Self {
            uri: uri.into_location(),
        }
    }
}

impl Uri<&'static CStr> {
    pub const fn define(uri: &'static CStr) -> Self {
        Self { uri }
    }
}

impl<L: ffi::Location> Endpoint for Uri<L> {
    fn location(&self) -> &CStr {
        self.uri.as_ref()
    }

    fn flags(&self) -> i32 {
        SQLITE_OPEN_URI
    }
}

impl<L: ffi::Location> AsRef<L> for Uri<L> {
    fn as_ref(&self) -> &L {
        &self.uri
    }
}

impl<L: ffi::Location> Deref for Uri<L> {
    type Target = L;

    fn deref(&self) -> &Self::Target {
        &self.uri
    }
}

#[cfg(feature = "url")]
#[cfg_attr(docsrs, doc(cfg(feature = "url")))]
impl From<url::Url> for Uri {
    fn from(value: url::Url) -> Self {
        Self::new(value.as_str())
    }
}

/// Selects a SQLite [virtual filesystem][vfs] for an [`Endpoint`].
///
/// [vfs]: https://sqlite.org/vfs.html
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub struct Vfs<E: Endpoint = Local, L: ffi::Location = CString> {
    endpoint: E,
    vfs: L,
}

impl<E: Endpoint, L: ffi::Location> Vfs<E, L> {
    pub fn new(endpoint: impl Into<E>, vfs: impl ffi::IntoLocation<Location = L>) -> Self {
        Self {
            endpoint: endpoint.into(),
            vfs: vfs.into_location(),
        }
    }
}

impl<E: Endpoint> Vfs<E, &'static CStr> {
    pub const fn define(endpoint: E, vfs: &'static CStr) -> Self {
        Self { endpoint, vfs }
    }
}

impl<E: Endpoint, L: ffi::Location> Endpoint for Vfs<E, L> {
    fn location(&self) -> &CStr {
        self.endpoint.location()
    }

    fn vfs(&self) -> Option<&CStr> {
        Some(self.vfs.as_ref())
    }
}

impl<E: Endpoint, L: ffi::Location> AsRef<E> for Vfs<E, L> {
    fn as_ref(&self) -> &E {
        &self.endpoint
    }
}

impl<E: Endpoint, L: ffi::Location> Deref for Vfs<E, L> {
    type Target = E;

    fn deref(&self) -> &Self::Target {
        &self.endpoint
    }
}

/// A value which can be used as a database [`Endpoint`].
pub trait IntoEndpoint {
    type Endpoint: Endpoint;

    fn into_endpoint(self) -> Self::Endpoint;
}

#[cfg(sqlite_has_memory_database)]
impl IntoEndpoint for Memory {
    type Endpoint = Self;

    fn into_endpoint(self) -> Self::Endpoint {
        self
    }
}

impl<L: ffi::Location> IntoEndpoint for Local<L> {
    type Endpoint = Self;

    fn into_endpoint(self) -> Self::Endpoint {
        self
    }
}

impl<L: ffi::Location> IntoEndpoint for Uri<L> {
    type Endpoint = Self;

    fn into_endpoint(self) -> Self::Endpoint {
        self
    }
}

impl<E: Endpoint, L: ffi::Location> IntoEndpoint for Vfs<E, L> {
    type Endpoint = Self;

    fn into_endpoint(self) -> Self::Endpoint {
        self
    }
}

impl<L: ffi::IntoLocation> IntoEndpoint for L {
    type Endpoint = Local<L::Location>;

    fn into_endpoint(self) -> Self::Endpoint {
        Local::new(self)
    }
}
