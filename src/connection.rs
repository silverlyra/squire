use core::{ffi::CStr, fmt, mem};
use std::ffi::CString;

use sqlite::{
    SQLITE_OPEN_CREATE, SQLITE_OPEN_FULLMUTEX, SQLITE_OPEN_NOFOLLOW, SQLITE_OPEN_NOMUTEX,
    SQLITE_OPEN_READONLY, SQLITE_OPEN_READWRITE, SQLITE_OPEN_URI,
};

use crate::{
    database::{Database, Endpoint, IntoLocation},
    error::{ErrorLocation, ErrorMessage, Result},
    ffi,
    param::Parameters,
    statement::{PrepareOptions, Statement},
};

#[derive(Debug)]
pub struct Connection {
    inner: ffi::Connection,
}

impl Connection {
    #[cfg(feature = "lang-iat")]
    type Builder<L = CString>
        = ConnectionBuilder<L>
    where
        L: AsRef<CStr> + Clone + fmt::Debug;

    #[inline]
    #[must_use]
    fn new(inner: ffi::Connection) -> Self {
        Self { inner }
    }

    #[must_use]
    pub fn open<L>(database: impl AsRef<Database<L>>) -> Result<Self>
    where
        L: AsRef<CStr> + Clone + fmt::Debug,
    {
        let connection = ffi::Connection::open(
            database.as_ref().endpoint().location(),
            DEFAULT_OPEN_MODE,
            None,
        )?;

        Ok(Connection::new(connection))
    }

    #[must_use]
    pub fn builder<L>(database: impl ToOwned<Owned = Database<L>>) -> ConnectionBuilder<L>
    where
        L: AsRef<CStr> + Clone + fmt::Debug,
    {
        ConnectionBuilder::new(database.to_owned().into_endpoint())
    }

    #[must_use]
    pub fn prepare(
        &self,
        query: impl AsRef<str>,
    ) -> Result<Statement<'_>, (ErrorMessage, Option<ErrorLocation>)> {
        Statement::prepare(self, query, PrepareOptions::transient())
    }

    #[must_use]
    pub fn execute<P: for<'a> Parameters<'a>>(
        &self,
        query: impl AsRef<str>,
        parameters: P,
    ) -> Result<isize, (ErrorMessage, Option<ErrorLocation>)> {
        let changes = self.prepare(query)?.query(parameters)?.run()?;
        Ok(changes)
    }

    pub fn close(mut self) -> Result<(), ()> {
        let result = unsafe { self.dispose() };
        mem::forget(self); // or Drop will close the connection agian
        result
    }

    unsafe fn dispose(&mut self) -> Result<(), ()> {
        unsafe { self.inner.dispose() }
    }

    /// Access the [`ffi::Connection`] underlying this [`Connection`].
    #[inline]
    pub fn internal_ref(&self) -> &ffi::Connection {
        &self.inner
    }
}

impl ffi::Connected for Connection {
    fn as_connection_ptr(&self) -> *mut sqlite::sqlite3 {
        self.internal_ref().as_ptr()
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        let _ = unsafe { self.dispose() };
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub struct ConnectionBuilder<L = CString>
where
    L: AsRef<CStr> + Clone + fmt::Debug,
{
    endpoint: Endpoint<L>,
    flags: i32,
    vfs: Option<L>,
}

const DEFAULT_OPEN_MODE: i32 = SQLITE_OPEN_READWRITE | SQLITE_OPEN_CREATE;

const FILE_OPEN_MODES: i32 = SQLITE_OPEN_READONLY | SQLITE_OPEN_READWRITE | SQLITE_OPEN_CREATE;
const CONCURRENCY_MODES: i32 = SQLITE_OPEN_FULLMUTEX | SQLITE_OPEN_NOMUTEX;

impl<L> ConnectionBuilder<L>
where
    L: AsRef<CStr> + Clone + fmt::Debug,
{
    const fn new(endpoint: Endpoint<L>) -> Self {
        Self {
            endpoint,
            flags: DEFAULT_OPEN_MODE,
            vfs: None,
        }
    }

    /// Open a [`Connection`] using the configuration set on this
    /// [builder](Self).
    pub fn open(&self) -> Result<Connection> {
        let vfs = match &self.vfs {
            Some(vfs) => Some(vfs.as_ref()),
            None => None,
        };

        let connection = ffi::Connection::open(
            self.endpoint.location(),
            self.flags | self.endpoint.flags(),
            vfs,
        )?;

        Ok(Connection::new(connection))
    }

    #[doc(alias = "SQLITE_OPEN_READONLY")]
    pub fn read_only(self) -> Self {
        self.with_open_mode(SQLITE_OPEN_READONLY)
    }

    #[doc(alias = "SQLITE_OPEN_CREATE")]
    #[doc(alias = "SQLITE_OPEN_READWRITE")]
    pub fn read_write(self, create: bool) -> Self {
        self.with_open_mode(if create {
            SQLITE_OPEN_READWRITE | SQLITE_OPEN_CREATE
        } else {
            SQLITE_OPEN_READWRITE
        })
    }

    #[doc(alias = "SQLITE_OPEN_NOFOLLOW")]
    pub fn follow_symbolic_links(self, follow: bool) -> Self {
        let flags = if follow {
            self.flags | SQLITE_OPEN_NOFOLLOW
        } else {
            self.flags & !SQLITE_OPEN_NOFOLLOW
        };
        self.with_flags(flags)
    }

    #[doc(alias = "SQLITE_OPEN_FULLMUTEX")]
    #[doc(alias = "SQLITE_OPEN_NOMUTEX")]
    pub fn mutex(self, enable: bool) -> Self {
        self.with_concurrency_mode(if enable {
            SQLITE_OPEN_FULLMUTEX
        } else {
            SQLITE_OPEN_NOMUTEX
        })
    }

    #[doc(alias = "SQLITE_OPEN_URI")]
    pub fn uri_filenames(self, enable: bool) -> Self {
        let flags = if enable {
            self.flags | SQLITE_OPEN_URI
        } else {
            self.flags & !SQLITE_OPEN_URI
        };
        self.with_flags(flags)
    }

    pub fn vfs(self, vfs: impl IntoLocation<Location = L>) -> Self {
        Self {
            endpoint: self.endpoint,
            flags: self.flags,
            vfs: Some(vfs.into_location()),
        }
    }

    #[inline]
    fn with_open_mode(self, flags: i32) -> Self {
        let flags = (self.flags & !FILE_OPEN_MODES) | flags;
        self.with_flags(flags)
    }

    #[inline]
    fn with_concurrency_mode(self, flags: i32) -> Self {
        let flags = (self.flags & !CONCURRENCY_MODES) | flags;
        self.with_flags(flags)
    }

    #[inline]
    fn with_flags(self, flags: i32) -> Self {
        Self {
            endpoint: self.endpoint,
            flags,
            vfs: self.vfs,
        }
    }
}
