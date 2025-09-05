use core::{ffi::CStr, fmt};
use std::ffi::CString;

use sqlite::{
    SQLITE_OPEN_CREATE, SQLITE_OPEN_FULLMUTEX, SQLITE_OPEN_NOFOLLOW, SQLITE_OPEN_NOMUTEX,
    SQLITE_OPEN_READONLY, SQLITE_OPEN_READWRITE,
};

use crate::database::{Database, Endpoint, IntoLocation};
use crate::error::Result;
use crate::ffi;

#[derive(Debug)]
pub struct Connection {
    inner: ffi::Connection,
}

impl Connection {
    #[cfg(feature = "nightly")]
    type Builder<L = CString>
        = ConnectionBuilder<L>
    where
        L: AsRef<CStr> + Clone + fmt::Debug;

    fn new(inner: ffi::Connection) -> Self {
        Self { inner }
    }

    pub fn open<L>(database: &Database<L>) -> Result<Self>
    where
        L: AsRef<CStr> + Clone + fmt::Debug,
    {
        let connection =
            ffi::Connection::open(database.endpoint().location(), DEFAULT_OPEN_MODE, None)?;

        Ok(Connection::new(connection))
    }

    pub fn builder<L>(database: impl ToOwned<Owned = Database<L>>) -> ConnectionBuilder<L>
    where
        L: AsRef<CStr> + Clone + fmt::Debug,
    {
        ConnectionBuilder::new(database.to_owned().into_endpoint())
    }

    pub fn close(self) -> Result<()> {
        self.inner.close()
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
