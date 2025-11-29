use core::{ffi::CStr, fmt, mem};
use std::ffi::CString;

use sqlite::{
    SQLITE_OPEN_CREATE, SQLITE_OPEN_FULLMUTEX, SQLITE_OPEN_NOFOLLOW, SQLITE_OPEN_NOMUTEX,
    SQLITE_OPEN_READONLY, SQLITE_OPEN_READWRITE, SQLITE_OPEN_URI,
};

use crate::{
    database::{Database, Endpoint, IntoLocation},
    error::Result,
    ffi,
    param::Parameters,
    statement::{PrepareOptions, Statement},
};

/// A _connection_ to one or more open SQLite database(s).
///
/// Use `Connection` to [prepare](Self::prepare) and [execute](Self::execute)
/// SQL statements. A `Connection` will remain open until [closed](Self::close)
/// or [dropped](core::ops::Drop).
///
/// (Even though SQLite is a local database, without a network or socket
/// connection to a remote server, SQLite still uses the term “connection”.)
///
/// # Examples
///
/// ```rust
/// use squire::{Connection, Database};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let connection = Connection::open(Database::memory())?;
///
/// let mut statement = connection.prepare("SELECT sqlite_version();")?;
/// let version: String = statement.query(())?.one()?;
#[doc = concat!("assert_eq!(\"", env!("SQUIRE_SQLITE_VERSION"), "\", version);")]
/// # Ok(())
/// # }
/// ```
pub struct Connection {
    inner: ffi::Connection,
}

impl Connection {
    #[inline]
    #[must_use]
    fn new(inner: ffi::Connection) -> Self {
        Self { inner }
    }

    /// Open a read/write [`Connection`] to a [`Database`].
    #[must_use = "a Connection will be closed if dropped"]
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

    /// Customize a [`Connection`] by configuring a [`ConnectionBuilder`].
    #[must_use]
    pub fn builder<L>(database: impl ToOwned<Owned = Database<L>>) -> ConnectionBuilder<L>
    where
        L: AsRef<CStr> + Clone + fmt::Debug,
    {
        ConnectionBuilder::new(database.to_owned().into_endpoint())
    }

    /// Prepare a SQL [`Statement`] to be executed as a query.
    #[must_use = "a Statement will be finalized if dropped"]
    pub fn prepare(&self, query: impl AsRef<str>) -> Result<Statement<'_>> {
        Statement::prepare(self, query, PrepareOptions::transient())
    }

    /// Execute a SQL statement and return the number of affected rows.
    pub fn execute<P: for<'a> Parameters<'a>>(
        &self,
        query: impl AsRef<str>,
        parameters: P,
    ) -> Result<isize> {
        let changes = self.prepare(query)?.query(parameters)?.run()?;
        Ok(changes)
    }

    /// Close this [`Connection`].
    ///
    /// A `Connection` is also closed when it is dropped.
    pub fn close(mut self) -> Result<()> {
        let result = unsafe { self.dispose() };
        mem::forget(self); // or Drop will close the connection agian
        result
    }

    unsafe fn dispose(&mut self) -> Result<()> {
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

impl fmt::Debug for Connection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Connection({:p})", self.internal_ref().as_ptr())
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        let _ = unsafe { self.dispose() };
    }
}

/// Configure a [`Connection`] to be opened.
///
/// Create a `ConnectionBuilder` with [`Connection::builder`].
///
/// # Example
///
/// ```no_run
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use squire::{Connection, Database};
///
/// let connection = Connection::builder(Database::path("./data.sqlite3"))
///     .read_only()
///     .follow_symbolic_links(false)
///     .open()?;
/// # let _ = connection;
/// # Ok(())
/// # }
/// ```
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub struct ConnectionBuilder<L = CString>
where
    L: AsRef<CStr> + Clone + fmt::Debug,
{
    endpoint: Endpoint<L>,
    flags: i32,
    vfs: Option<L>,
}

/// Default open mode flags for new connections.
///
/// When the `serialized` feature is enabled, connections are opened with
/// `SQLITE_OPEN_FULLMUTEX` to ensure full mutex protection even if the
/// underlying SQLite library was built with a less restrictive threading mode.
#[cfg(feature = "serialized")]
const DEFAULT_OPEN_MODE: i32 = SQLITE_OPEN_READWRITE | SQLITE_OPEN_CREATE | SQLITE_OPEN_FULLMUTEX;

/// Default open mode flags for new connections.
///
/// When only `multi-thread` is enabled (not `serialized`), connections are
/// opened with `SQLITE_OPEN_NOMUTEX` to disable the recursive mutexes on
/// database connections, matching the expected threading model.
#[cfg(all(feature = "multi-thread", not(feature = "serialized")))]
const DEFAULT_OPEN_MODE: i32 = SQLITE_OPEN_READWRITE | SQLITE_OPEN_CREATE | SQLITE_OPEN_NOMUTEX;

/// Default open mode flags for new connections.
///
/// When no threading features are enabled, connections use the default
/// SQLite behavior without explicit mutex flags.
#[cfg(not(feature = "multi-thread"))]
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
        let vfs = self.vfs.as_ref().map(|vfs| vfs.as_ref());

        let connection = ffi::Connection::open(
            self.endpoint.location(),
            self.flags | self.endpoint.flags(),
            vfs,
        )?;

        Ok(Connection::new(connection))
    }

    /// Open the connection in read-only mode.
    #[doc(alias = "SQLITE_OPEN_READONLY")]
    pub fn read_only(self) -> Self {
        self.with_open_mode(SQLITE_OPEN_READONLY)
    }

    /// Open the connection in read/write mode.
    ///
    /// If `create` is true, the database will be created if it doesn’t already exist.
    #[doc(alias = "SQLITE_OPEN_CREATE")]
    #[doc(alias = "SQLITE_OPEN_READWRITE")]
    pub fn read_write(self, create: bool) -> Self {
        self.with_open_mode(if create {
            SQLITE_OPEN_READWRITE | SQLITE_OPEN_CREATE
        } else {
            SQLITE_OPEN_READWRITE
        })
    }

    /// Enable or disable travsersing symbolic links to load a database file.
    #[doc(alias = "SQLITE_OPEN_NOFOLLOW")]
    pub fn follow_symbolic_links(self, follow: bool) -> Self {
        let flags = if follow {
            self.flags | SQLITE_OPEN_NOFOLLOW
        } else {
            self.flags & !SQLITE_OPEN_NOFOLLOW
        };
        self.with_flags(flags)
    }

    /// Enable or disable the serialized [threading mode][].
    ///
    /// > In serialized mode, API calls to affect or use any SQLite database
    /// > [connection](crate::Connection) or any object derived from such a
    /// > database connection can be made safely from multiple threads. The effect
    /// > on an individual object is the same as if the API calls had all been
    /// > made in the same order from a single thread. The name ”serialized”
    /// > arises from the fact that SQLite uses mutexes to serialize access to
    /// > each object.
    ///
    /// [threading mode]: https://sqlite.org/threadsafe.html
    #[doc(alias = "SQLITE_OPEN_FULLMUTEX")]
    #[doc(alias = "SQLITE_OPEN_NOMUTEX")]
    pub fn mutex(self, enable: bool) -> Self {
        self.with_concurrency_mode(if enable {
            SQLITE_OPEN_FULLMUTEX
        } else {
            SQLITE_OPEN_NOMUTEX
        })
    }

    /// Enable or disable [URI][] database filename support.
    ///
    /// [URI]: https://sqlite.org/uri.html
    #[doc(alias = "SQLITE_OPEN_URI")]
    pub fn uri_filenames(self, enable: bool) -> Self {
        let flags = if enable {
            self.flags | SQLITE_OPEN_URI
        } else {
            self.flags & !SQLITE_OPEN_URI
        };
        self.with_flags(flags)
    }

    /// Select which [virtual filesystem][vfs] to use for the connection.
    ///
    /// [vfs]: https://sqlite.org/vfs.html
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
