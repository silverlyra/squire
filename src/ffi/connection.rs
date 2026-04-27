#[cfg(feature = "functions")]
use core::ffi::c_void;
use core::{ffi::CStr, fmt, ptr};

#[cfg(feature = "functions")]
use sqlite::sqlite3_create_function_v2;
#[cfg(sqlite_has_error_offset)]
use sqlite::sqlite3_error_offset;
#[cfg(sqlite_has_set_error_message)]
use sqlite::sqlite3_set_errmsg;
use sqlite::{
    SQLITE_OK, SQLITE_OPEN_EXRESCODE, sqlite3, sqlite3_close, sqlite3_errcode, sqlite3_errmsg,
    sqlite3_errstr, sqlite3_open_v2,
};

use super::call::call;
#[cfg(feature = "mutex")]
use super::mutex::MutexRef;
#[cfg(feature = "functions")]
use super::{
    bind::destroy_box,
    func::{Function, call},
};
use crate::error::{Error, Result};

/// A thin wrapper around a [`sqlite3`] connection pointer.
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
    ///
    /// If `handle` is null, returns `None`.
    #[inline]
    #[must_use]
    pub const fn new(handle: *mut sqlite3) -> Option<Self> {
        match ptr::NonNull::new(handle) {
            Some(handle) => Some(Self { handle }),
            None => None,
        }
    }

    /// Adopt a raw [`sqlite3`] connection pointer without checking for null.
    ///
    /// # Safety
    ///
    /// `handle` must not be null.
    #[inline]
    pub unsafe fn new_unchecked(handle: *mut sqlite3) -> Self {
        Self {
            handle: unsafe { ptr::NonNull::new_unchecked(handle) },
        }
    }

    /// Open a new SQLite database connection.
    #[must_use = "a Connection will leak if opened and discarded"]
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
    pub fn close(mut self) -> Result<()> {
        // SAFETY: We own `self` here and will let it be dropped.
        unsafe { self.dispose() }
    }

    /// Get the [code][] and message describing the [most recent error][]
    /// on this [`Connection`].
    ///
    /// # Safety
    ///
    /// Callers must not retain the returned [`CStr`]; per the SQLite docs:
    ///
    /// > the error string might be overwritten **or deallocated** by
    /// > subsequent calls to other SQLite interface functions
    ///
    /// [code]: https://sqlite.org/rescode.html
    /// [most recent error]: https://sqlite.org/c3ref/errcode.html
    #[doc(alias = "sqlite3_errcode")]
    #[doc(alias = "sqlite3_extended_errcode")]
    #[doc(alias = "sqlite3_errmsg")]
    #[cfg_attr(feature = "utf-16", doc(alias = "sqlite3_errmsg16"))]
    pub unsafe fn last_error<'c, 'a>(&'c self) -> (i32, Option<&'a CStr>)
    where
        'c: 'a,
    {
        let code = unsafe { sqlite3_errcode(self.as_ptr()) };
        if code == SQLITE_OK {
            return (SQLITE_OK, None);
        }

        let message = unsafe { sqlite3_errmsg(self.as_ptr()) };
        let default_message = unsafe { sqlite3_errstr(code) };

        if !message.is_null() && message.addr() != default_message.addr() {
            (code, Some(unsafe { CStr::from_ptr(message) }))
        } else {
            (code, None)
        }
    }

    /// Get the SQL source offset of the [last error][].
    ///
    /// [last error]: Self::last_error
    #[doc(alias = "sqlite3_error_offset")]
    #[cfg(sqlite_has_error_offset)]
    pub fn last_error_offset(&self) -> i32 {
        unsafe { sqlite3_error_offset(self.as_ptr()) }
    }

    /// Set the [last error][] code and message associated with this [`Connection`].
    ///
    /// # Safety
    ///
    /// When the [`Connection`] can be accessed from different threads,
    /// `set_last_error` function can not ensure that its effects will be
    /// visible to any caller. (`ffi` doesn't require `&mut Connection`.)
    ///
    /// Callers must take care to ensure this call will be visible, or accept it
    /// may not take effect.
    ///
    /// [last error]: Self::last_error
    #[doc(alias = "sqlite3_set_errmsg")]
    #[cfg(sqlite_has_set_error_message)]
    pub unsafe fn set_last_error(&self, code: i32, message: Option<&CStr>) -> Result<()> {
        let message = match message {
            Some(message) => message.as_ptr(),
            None => ptr::null(),
        };

        call! { sqlite3_set_errmsg(self.as_ptr(), code, message) }
    }

    /// Define a scalar [SQL function][].
    ///
    /// [SQL function]: https://sqlite.org/c3ref/create_function.html
    #[cfg(feature = "functions")]
    #[cfg_attr(docsrs, doc(cfg(feature = "functions")))]
    pub fn define_scalar_function<F: Function>(
        &self,
        name: &CStr,
        func: F,
        arity: i32,
        flags: i32,
    ) -> Result<()> {
        let func = Box::into_raw(Box::new(func));

        let result = unsafe {
            sqlite3_create_function_v2(
                self.as_ptr(),
                name.as_ptr(),
                arity,
                flags,
                func.cast::<c_void>(),
                Some(call::<F>),
                None,
                None,
                Some(destroy_box::<F>),
            )
        };

        match Error::from_connection(self, result) {
            None => Ok(()),
            Some(err) => Err(err),
        }
    }

    #[inline]
    pub(crate) unsafe fn dispose(&mut self) -> Result<()> {
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

impl fmt::Debug for Connection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Connection({:p})", self.handle)
    }
}

/// An [`ffi`](crate::ffi) type through which a [`sqlite3`] pointer
/// can be accessed.
pub trait Connected {
    /// The [`sqlite3`] connection pointer.
    ///
    /// The returned pointer is never null.
    fn as_connection_ptr(&self) -> *mut sqlite3;

    /// Wrap the connection pointer as an [`ffi::Connection`](Connection).
    ///
    /// The returned [`Connection`] borrows the underlying handle and must not
    /// be [closed](Connection::close).
    fn as_connection(&self) -> Connection {
        // SAFETY: as_connection_ptr() is guaranteed to return a non-null pointer.
        unsafe { Connection::new_unchecked(self.as_connection_ptr()) }
    }
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

    #[cfg(feature = "functions")]
    #[test]
    fn test_define_function() {
        use sqlite::{SQLITE_DETERMINISTIC, SQLITE_INNOCUOUS, SQLITE_UTF8};

        use crate::ffi::{ContextRef, Function, Statement, ValueRef};
        use crate::types::ColumnIndex;

        struct Maximum;

        impl Function for Maximum {
            fn call<'a>(&self, context: &'a mut ContextRef<'a>, arguments: &'a [ValueRef<'a>]) {
                if arguments.is_empty() {
                    return;
                }

                let value = arguments
                    .iter()
                    .map(|arg| unsafe { arg.fetch::<i64>() })
                    .max();

                match value {
                    Some(value) => unsafe { context.set_result(value) },
                    None => {
                        context.set_error("no arguments to maximum()");
                        context.set_error_code(sqlite::SQLITE_ERROR_UNABLE);
                    }
                }
            }
        }

        let connection = Connection::open(
            c":memory:",
            SQLITE_OPEN_READWRITE | SQLITE_OPEN_CREATE,
            None,
        )
        .expect("open SQLite connection");

        connection
            .define_scalar_function(
                c"maximum",
                Maximum,
                -1,
                SQLITE_UTF8 | SQLITE_DETERMINISTIC | SQLITE_INNOCUOUS,
            )
            .expect("define function");

        let (check, _) = Statement::prepare(&connection, "SELECT maximum(-1, 10, 510, 2)", 0)
            .expect("prepare statement");

        assert!(unsafe { check.row().expect("next row") });

        let value: i64 = unsafe { check.fetch(ColumnIndex::INITIAL) };
        assert_eq!(value, 510);

        check.close().expect("finalize SQLite statement");

        connection.close().expect("close SQLite connection");
    }
}
