use core::{ops::Deref, slice};

use sqlite::{SQLITE_STATIC, sqlite3_column_blob, sqlite3_column_bytes, sqlite3_column_text};
#[cfg(target_pointer_width = "64")]
use sqlite::{SQLITE_UTF8, sqlite3_bind_blob64, sqlite3_bind_text64, sqlite3_uint64};
#[cfg(target_pointer_width = "32")]
use sqlite::{sqlite3_bind_blob, sqlite3_bind_text};

use crate::{
    error::Result,
    ffi::{Bind, Fetch, Statement},
    types::{BindIndex, ColumnIndex},
};

#[cfg(target_pointer_width = "64")]
const ENCODING_UTF8: u8 = SQLITE_UTF8 as u8;

/// A borrowed reference that can be used for zero-copy parameter binding and
/// column value access.
///
/// When used for [binding](Bind), `Borrowed` marks a reference as outliving a
/// SQLite [prepared statement](Statement), which SQLite does not need to copy
/// to use as a parameter value. Values are passed to SQLite with the
/// [`SQLITE_STATIC`] flag, which prevents SQLite from [cloning][] the data.
///
/// When used for [fetching](Fetch), `Borrowed` wraps references returned from
/// `sqlite3_column_*` functions, providing zero-copy access to column data that
/// is valid for the lifetime of the statement row.
///
/// [cloning]: https://sqlite.org/c3ref/c_static.html
#[derive(Copy, Clone, Debug)]
#[repr(transparent)]
#[doc(alias = "SQLITE_STATIC")]
pub struct Borrowed<'a, T: ?Sized>(&'a T);

impl<'a, T: ?Sized> Borrowed<'a, T> {
    /// Creates a new `Borrowed` wrapper around a reference.
    pub const fn new(value: &'a T) -> Self {
        Self(value)
    }

    /// Unwraps the `Borrowed` wrapper, returning the inner reference.
    pub const fn into_inner(self) -> &'a T {
        self.0
    }

    /// Returns a pointer to the wrapped value.
    #[inline]
    pub(crate) fn as_ptr(&self) -> *const T {
        self.0 as *const T
    }
}

impl<'a, T: ?Sized> Deref for Borrowed<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

// Bind implementations

#[cfg_attr(
    target_pointer_width = "32",
    doc = "[Binds](Bind) a [`&str`](str) via [`sqlite3_bind_text`]."
)]
#[cfg_attr(
    target_pointer_width = "64",
    doc = "[Binds](Bind) a [`&str`](str) via [`sqlite3_bind_text64`]."
)]
///
/// The [`SQLITE_STATIC`] flag is used; SQLite will read the string's bytes
/// without [cloning][] them.
///
/// [cloning]: https://sqlite.org/c3ref/c_static.html
impl<'b, 'a: 'b> Bind<'b> for Borrowed<'a, str> {
    unsafe fn bind<'c>(self, statement: &Statement<'c>, index: BindIndex) -> Result<()>
    where
        'c: 'b,
    {
        #[cfg(target_pointer_width = "32")]
        {
            use crate::error::{Error, ErrorMessage};
            let result = unsafe {
                sqlite3_bind_text(
                    statement.as_ptr(),
                    index.value(),
                    self.as_ptr() as *const i8,
                    self.0.len() as i32,
                    SQLITE_STATIC,
                )
            };
            match Error::<ErrorMessage>::from_connection(statement, result) {
                Some(err) => Err(err),
                None => Ok(()),
            }
        }

        #[cfg(target_pointer_width = "64")]
        {
            use crate::error::{Error, ErrorMessage};
            let result = unsafe {
                sqlite3_bind_text64(
                    statement.as_ptr(),
                    index.value(),
                    self.as_ptr() as *const i8,
                    self.0.len() as sqlite3_uint64,
                    SQLITE_STATIC,
                    ENCODING_UTF8,
                )
            };
            match Error::<ErrorMessage>::from_connection(statement, result) {
                Some(err) => Err(err),
                None => Ok(()),
            }
        }
    }
}

#[cfg_attr(
    target_pointer_width = "32",
    doc = "[Binds](Bind) a [`&[u8]`](primitive@slice) via [`sqlite3_bind_blob`]."
)]
#[cfg_attr(
    target_pointer_width = "64",
    doc = "[Binds](Bind) a [`&[u8]`](primitive@slice) via [`sqlite3_bind_blob64`]."
)]
///
/// The [`SQLITE_STATIC`] flag is used; SQLite will read the bytes without
/// [cloning][] them.
///
/// [cloning]: https://sqlite.org/c3ref/c_static.html
impl<'b, 'a: 'b> Bind<'b> for Borrowed<'a, [u8]> {
    unsafe fn bind<'c>(self, statement: &Statement<'c>, index: BindIndex) -> Result<()>
    where
        'c: 'b,
    {
        #[cfg(target_pointer_width = "32")]
        {
            use crate::error::{Error, ErrorMessage};
            use core::ffi::c_void;
            let result = unsafe {
                sqlite3_bind_blob(
                    statement.as_ptr(),
                    index.value(),
                    self.as_ptr() as *const c_void,
                    self.0.len() as i32,
                    SQLITE_STATIC,
                )
            };
            match Error::<ErrorMessage>::from_connection(statement, result) {
                Some(err) => Err(err),
                None => Ok(()),
            }
        }

        #[cfg(target_pointer_width = "64")]
        {
            use crate::error::{Error, ErrorMessage};
            use core::ffi::c_void;
            let result = unsafe {
                sqlite3_bind_blob64(
                    statement.as_ptr(),
                    index.value(),
                    self.as_ptr() as *const c_void,
                    self.0.len() as sqlite3_uint64,
                    SQLITE_STATIC,
                )
            };
            match Error::<ErrorMessage>::from_connection(statement, result) {
                Some(err) => Err(err),
                None => Ok(()),
            }
        }
    }
}

// Fetch implementations

impl<'r> Fetch<'r> for Borrowed<'r, str> {
    unsafe fn fetch<'c>(statement: &'r Statement<'c>, column: ColumnIndex) -> Self
    where
        'c: 'r,
    {
        let data = unsafe { sqlite3_column_text(statement.as_ptr(), column.value()) };
        let len = unsafe { sqlite3_column_bytes(statement.as_ptr(), column.value()) };

        let bytes = unsafe { slice::from_raw_parts::<'r, u8>(data, len as usize) };
        let text = unsafe { core::str::from_utf8_unchecked(bytes) };

        Self(text)
    }
}

impl<'r> Fetch<'r> for Borrowed<'r, [u8]> {
    unsafe fn fetch<'c>(statement: &'r Statement<'c>, column: ColumnIndex) -> Self
    where
        'c: 'r,
    {
        let data = unsafe { sqlite3_column_blob(statement.as_ptr(), column.value()) };
        let len = unsafe { sqlite3_column_bytes(statement.as_ptr(), column.value()) };

        let bytes = unsafe { slice::from_raw_parts::<'r, u8>(data as *const u8, len as usize) };

        Self(bytes)
    }
}
