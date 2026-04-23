use core::{ffi::c_void, ops::Deref, slice};

/// A reference that can be used for zero-copy parameter [binding](crate::Bind)
/// and column value [access](crate::Fetch).
///
/// When used for [binding](crate::Bind), `Borrowed` marks a reference as
/// outliving a SQLite [prepared statement](crate::Statement), which SQLite does
/// not need to copy to use as a parameter value. Values are passed to SQLite
/// as `SQLITE_STATIC`, which prevents SQLite from [cloning][] the data.
///
/// When used for [fetching](crate::Fetch), `Borrowed` wraps references returned
/// from `sqlite3_column_*` functions, providing zero-copy access to column data
/// that is valid for the lifetime of the statement row.
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

impl<'a> Borrowed<'a, str> {
    #[inline]
    pub(crate) unsafe fn from_raw_str(data: *const u8, len: i32) -> Self {
        let bytes = unsafe { slice::from_raw_parts::<'a, u8>(data, len as usize) };
        let text = unsafe { core::str::from_utf8_unchecked(bytes) };

        Self(text)
    }
}

impl<'a> Borrowed<'a, [u8]> {
    #[inline]
    pub(crate) unsafe fn from_raw_bytes(data: *const c_void, len: i32) -> Self {
        let bytes = unsafe { slice::from_raw_parts::<'a, u8>(data as *const u8, len as usize) };

        Self(bytes)
    }
}

impl<'a, T: ?Sized> Deref for Borrowed<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}
