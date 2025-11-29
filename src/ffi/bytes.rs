use core::{
    ffi::{c_uchar, c_void},
    fmt, mem,
    ops::Deref,
    ptr, slice,
};

#[cfg(target_pointer_width = "32")]
use sqlite::sqlite3_bind_blob;
#[cfg(target_pointer_width = "64")]
use sqlite::{sqlite3_bind_blob64, sqlite3_uint64};
use sqlite::{sqlite3_destructor_type, sqlite3_free, sqlite3_malloc64};

use super::{
    bind::{Bind, bind},
    statement::Statement,
};
use crate::{
    error::{Error, ErrorCategory, Result},
    types::BindIndex,
};

const EMPTY: &[u8] = &[];

/// An owned `[u8]` array allocated on the SQLite heap.
///
/// The array is allocated via [`sqlite3_malloc64`] and will be freed via
/// [`sqlite3_free`] when dropped. It dereferences to `&[u8]`.
///
/// [`sqlite3_malloc64`]: https://sqlite.org/c3ref/free.html
/// [`sqlite3_free`]: https://sqlite.org/c3ref/free.html
pub struct Bytes {
    data: *const c_uchar,
    len: usize,
}

impl Bytes {
    /// Copy `data` onto the [SQLite heap][free].
    ///
    /// [free]: https://sqlite.org/c3ref/free.html
    pub fn new(data: impl AsRef<[u8]>) -> Result<Self> {
        let data = data.as_ref();

        Self::allocate(data.len(), |dest| {
            dest.copy_from_slice(data);
            Ok(())
        })
    }

    /// Return an empty [`Bytes`].
    #[inline]
    pub const fn empty() -> Self {
        Self {
            data: ptr::null(),
            len: 0,
        }
    }

    /// Allocate `len` bytes on the [SQLite heap][free], and call `populate` to
    /// fill in the bytes.
    ///
    /// If `len` is zero, returns an empty [`Bytes`] without calling `populate`.
    ///
    /// Note that the provided slice may contain arbitrary bytes; they will not
    /// be zeroed before being passed to `populate`.
    ///
    /// [free]: https://sqlite.org/c3ref/free.html
    pub fn allocate<F: FnOnce(&mut [u8]) -> Result<()>>(len: usize, populate: F) -> Result<Self> {
        if len == 0 {
            return Ok(Self::empty());
        }

        let ptr = unsafe { sqlite3_malloc64(len as u64) as *mut c_uchar };

        if ptr.is_null() {
            return Err(ErrorCategory::OutOfMemory.into());
        }

        let data = unsafe { slice::from_raw_parts_mut(ptr, len) };
        populate(data)?;

        Ok(unsafe { Self::from_raw_parts(ptr, len) })
    }

    /// Create [`Bytes`] from a raw pointer and length.
    ///
    /// The `Bytes` will take ownership of given pointer; dropping the `Bytes`
    /// will [free][] the pointer.
    ///
    /// # Panics
    ///
    /// If `ptr` is null and `len` is non-zero, `from_raw_parts` will panic.
    ///
    /// # Safety
    ///
    /// - `ptr` must have been allocated via [`sqlite3_malloc64`][free]
    /// - `ptr` must point to `len` bytes
    ///
    /// [free]: https://sqlite.org/c3ref/free.html
    #[inline]
    pub const unsafe fn from_raw_parts(ptr: *const c_uchar, len: usize) -> Self {
        match ptr {
            _ if ptr.is_null() && len == 0 => panic!("expected non-null Bytes pointer"),
            data => Self { data, len },
        }
    }

    /// Consume [`Bytes`], returning the raw pointer and length.
    ///
    /// After calling this method, the caller is responsible for freeing the
    /// memory via [`sqlite3_free`].
    ///
    /// [`sqlite3_free`]: https://sqlite.org/c3ref/free.html
    #[inline]
    pub const fn into_raw_parts(self) -> (*const c_uchar, usize) {
        let (ptr, len) = (self.as_ptr(), self.len);
        mem::forget(self);

        (ptr, len)
    }

    /// Returns a pointer to the [`Bytes`]’ data.
    #[inline]
    pub const fn as_ptr(&self) -> *const c_uchar {
        self.data
    }

    /// Returns the number of allocated bytes.
    #[inline]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the [`Bytes`] is zero-[length](Self::len).
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    const fn data(&self) -> &[u8] {
        if self.is_empty() {
            EMPTY
        } else {
            unsafe { slice::from_raw_parts(self.data, self.len) }
        }
    }
}

impl AsRef<[u8]> for Bytes {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.data()
    }
}

impl Deref for Bytes {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.data()
    }
}

impl Drop for Bytes {
    fn drop(&mut self) {
        // Don't free zero-length Bytes; they point to a static empty slice.
        if self.len > 0 {
            // SAFETY: `self.data` was allocated via `sqlite3_malloc64`.
            unsafe { sqlite3_free(self.as_ptr() as *mut c_void) }
        }
    }
}

impl<const N: usize> From<[u8; N]> for Bytes {
    fn from(value: [u8; N]) -> Self {
        Self::allocate(N, move |dest| {
            dest.copy_from_slice(&value[..]);
            Ok(())
        })
        .expect("malloc")
    }
}

impl fmt::Debug for Bytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Bytes").field(&self.data()).finish()
    }
}

#[cfg_attr(
    target_pointer_width = "32",
    doc = "[Binds](Bind) [`ffi::Bytes`](Bytes) via [`sqlite3_bind_blob`]."
)]
#[cfg_attr(
    target_pointer_width = "64",
    doc = "[Binds](Bind) [`ffi::Bytes`](Bytes) via [`sqlite3_bind_blob64`]."
)]
///
/// The memory is transferred to SQLite, which will free it via [`sqlite3_free`]
/// when the binding is no longer needed.
impl<'b> Bind<'b> for Bytes {
    unsafe fn bind<'s>(self, statement: &Statement<'s>, index: BindIndex) -> Result<()>
    where
        's: 'b,
    {
        let (ptr, len) = self.into_raw_parts();
        let ptr = ptr as *const c_void;
        let destructor = sqlite3_destructor_type::new(sqlite3_free);

        #[cfg(target_pointer_width = "32")]
        bind! { sqlite3_bind_blob(statement, index, ptr, len as c_int, destructor) }?;

        #[cfg(target_pointer_width = "64")]
        bind! { sqlite3_bind_blob64(statement, index, ptr, len as sqlite3_uint64, destructor) }?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use sqlite::{SQLITE_OPEN_CREATE, SQLITE_OPEN_READWRITE};

    use super::*;
    use crate::ffi::Connection;

    #[test]
    fn test_new() {
        let data = b"hello, world!";
        let bytes = Bytes::new(data).expect("new");

        assert_eq!(bytes.len(), 13);
        assert!(!bytes.is_empty());
        assert_eq!(&*bytes, data);
    }

    #[test]
    fn test_new_empty() {
        let bytes = Bytes::new([]).expect("new");

        assert_eq!(bytes.len(), 0);
        assert!(bytes.is_empty());
        assert!(bytes.as_ptr().is_null());
        assert_eq!(&*bytes, &[] as &[u8]);
    }

    #[test]
    fn test_empty() {
        let bytes = Bytes::empty();

        assert_eq!(bytes.len(), 0);
        assert!(bytes.is_empty());
        assert_eq!(&*bytes, &[] as &[u8]);
    }

    #[test]
    fn test_allocate() {
        let bytes = Bytes::allocate(5, |dest| {
            dest.copy_from_slice(&[1, 2, 3, 4, 5]);
            Ok(())
        })
        .expect("allocate");

        assert_eq!(bytes.len(), 5);
        assert_eq!(&*bytes, &[1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_from_array() {
        let bytes: Bytes = [0xDE, 0xAD, 0xBE, 0xEF].into();

        assert_eq!(bytes.len(), 4);
        assert_eq!(&*bytes, &[0xDE, 0xAD, 0xBE, 0xEF]);
    }

    #[test]
    fn test_as_ref() {
        let bytes = Bytes::new([1, 2, 3]).expect("new");
        let slice: &[u8] = bytes.as_ref();

        assert_eq!(slice, &[1, 2, 3]);
    }

    #[test]
    fn test_deref() {
        let bytes = Bytes::new([4, 5, 6]).expect("new");

        assert_eq!(&*bytes, &[4, 5, 6]);
        assert_eq!(bytes[0], 4);
        assert_eq!(bytes[1], 5);
        assert_eq!(bytes[2], 6);
    }

    #[test]
    fn test_into_raw_parts() {
        let bytes = Bytes::new([7, 8, 9]).expect("new");
        let (ptr, len) = bytes.into_raw_parts();

        assert!(!ptr.is_null());
        assert_eq!(len, 3);

        // Reconstruct and drop to free memory
        let bytes = unsafe { Bytes::from_raw_parts(ptr, len) };
        assert_eq!(&*bytes, &[7, 8, 9]);
    }

    #[test]
    fn test_debug() {
        let bytes = Bytes::new([0x01, 0x02, 0x03]).expect("new");
        assert_eq!(format!("{bytes:?}"), "Bytes([1, 2, 3])");
    }

    #[test]
    fn test_bind_bytes() {
        use crate::Borrowed;
        use crate::ffi::Statement;

        let conn = Connection::open(
            c":memory:",
            SQLITE_OPEN_READWRITE | SQLITE_OPEN_CREATE,
            None,
        )
        .expect("open");

        let (stmt, _) = Statement::prepare(&conn, "SELECT ?", 0).expect("prepare");

        {
            let data = b"binary\x00data\xFF";
            let bytes = Bytes::new(data).expect("new");

            let index = BindIndex::new(1).expect("valid index");
            unsafe { stmt.bind(index, bytes) }.expect("bind");

            let has_row = unsafe { stmt.row() }.expect("row");
            assert!(has_row);

            let col = crate::types::ColumnIndex::new(0);
            let value: Borrowed<'_, [u8]> = unsafe { stmt.fetch(col) };
            assert_eq!(&*value, data);
        }

        stmt.close().expect("close stmt");
        conn.close().expect("close conn");
    }
}
