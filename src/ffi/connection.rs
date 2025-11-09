use core::{ffi::CStr, ptr};

use sqlite::{SQLITE_OK, SQLITE_OPEN_EXRESCODE, sqlite3, sqlite3_close, sqlite3_open_v2};

#[cfg(feature = "mutex")]
use super::{call::call, mutex::MutexRef};
use crate::error::{Error, Result};

/// A thin wrapper around a [`sqlite3`] connection pointer.
#[derive(Debug)]
#[repr(transparent)]
pub struct Connection {
    handle: ptr::NonNull<sqlite3>,
}

#[cfg(any(feature = "multi-thread", feature = "serialized"))]
#[cfg_attr(
    docsrs,
    doc(cfg(any(feature = "multi-thread", feature = "serialized")))
)]
unsafe impl Send for Connection {}

#[cfg(feature = "serialized")]
#[cfg_attr(docsrs, doc(cfg(feature = "serialized")))]
unsafe impl Sync for Connection {}

impl Connection {
    /// Adopt a raw [`sqlite3`] connection pointer.
    #[inline]
    #[must_use]
    pub const fn new(handle: *mut sqlite3) -> Option<Self> {
        match ptr::NonNull::new(handle) {
            Some(handle) => Some(Self { handle }),
            None => None,
        }
    }

    /// Open a new SQLite database connection.
    #[must_use]
    #[doc(alias = "sqlite3_open_v2")]
    pub fn open(path: &CStr, flags: i32, vfs: Option<&CStr>) -> Result<Self> {
        let path = path.as_ptr();
        let vfs = vfs.map(|vfs| vfs.as_ptr()).unwrap_or(ptr::null());

        let mut db: *mut sqlite3 = ptr::null_mut();
        let result = unsafe { sqlite3_open_v2(path, &mut db, flags | SQLITE_OPEN_EXRESCODE, vfs) };

        match Self::new(db) {
            Some(db) if result == SQLITE_OK => Ok(db),
            Some(db) => Err(Error::from_connection(db, result).unwrap_or_default()),
            None => Err(Error::from(result)),
        }
    }

    /// Close the SQLite database connection.
    #[inline]
    #[doc(alias = "sqlite3_close")]
    pub fn close(self) -> Result<(), ()> {
        call! { sqlite3_close(self.as_ptr()) }
    }

    #[cfg(feature = "mutex")]
    #[cfg_attr(docsrs, doc(cfg(any(feature = "mutex", feature = "serialized"))))]
    #[doc(alias = "sqlite3_db_mutex")]
    pub fn mutex(&self) -> Option<MutexRef<'_>> {
        MutexRef::from_connection(self.as_ptr())
    }

    /// Access the raw [`sqlite3`] connection pointer.
    #[inline]
    pub const fn as_ptr(&self) -> *mut sqlite3 {
        self.handle.as_ptr()
    }
}

pub trait Connected {
    fn as_connection_ptr(&self) -> *mut sqlite3;
}

impl Connected for Connection {
    fn as_connection_ptr(&self) -> *mut sqlite3 {
        self.as_ptr()
    }
}

impl Connected for &Connection {
    fn as_connection_ptr(&self) -> *mut sqlite3 {
        self.as_ptr()
    }
}

#[cfg(test)]
mod test {
    use sqlite::{SQLITE_OPEN_CREATE, SQLITE_OPEN_READWRITE};

    use super::Connection;

    #[test]
    fn test_open_memory() {
        let connection = Connection::open(
            c":memory:",
            SQLITE_OPEN_READWRITE | SQLITE_OPEN_CREATE,
            None,
        )
        .expect("open SQLite connection");
        connection.close().expect("close SQLite connection");
    }
}
