use core::{
    ffi::{CStr, c_char, c_int, c_void},
    fmt,
    iter::Extend,
    mem,
    ops::Deref,
    ptr, slice,
};

#[cfg(target_pointer_width = "32")]
use sqlite::sqlite3_bind_text;
#[cfg(target_pointer_width = "64")]
use sqlite::{SQLITE_UTF8, sqlite3_bind_text64, sqlite3_uint64};
use sqlite::{
    sqlite3_destructor_type, sqlite3_free, sqlite3_malloc, sqlite3_str, sqlite3_str_append,
    sqlite3_str_appendall, sqlite3_str_appendchar, sqlite3_str_errcode, sqlite3_str_finish,
    sqlite3_str_length, sqlite3_str_new,
};

use super::{
    bind::{Bind, bind},
    connection::Connected,
    statement::Statement,
};
use crate::{
    error::{Error, ErrorCategory, ErrorReason, ParameterError, Result},
    types::BindIndex,
};

/// An owned null-terminated UTF-8 string allocated on the SQLite heap.
///
/// The string is allocated via [`sqlite3_malloc64`] and will be freed via
/// [`sqlite3_free`] when dropped. It dereferences to [`CStr`].
///
/// [`sqlite3_malloc64`]: https://sqlite.org/c3ref/free.html
/// [`sqlite3_free`]: https://sqlite.org/c3ref/free.html
pub struct String {
    text: *const c_char,
    len: usize,
}

impl String {
    /// Build a [`String`] from the [`Display`](fmt::Display) of a value.
    pub fn display<D: fmt::Display + ?Sized>(value: &D) -> Result<Self> {
        use fmt::Write as _;

        let mut builder = StringBuilder::detached();
        match write!(&mut builder, "{value}") {
            Ok(_) => builder.finish(),
            Err(_) => Err(ErrorReason::Parameter(ParameterError::Bind).into()),
        }
    }

    /// Create a [`String`] from a raw pointer and length.
    ///
    /// The `String` will take ownership of given pointer; dropping the `String`
    /// will [free][] the pointer.
    ///
    /// # Panics
    ///
    /// If `ptr` is null, `from_raw_parts` will panic.
    ///
    /// # Safety
    ///
    /// - `ptr` must have been allocated via [`sqlite3_malloc64`][free] or
    ///   returned from [`sqlite3_str_finish`].
    /// - `ptr` must point to `len` bytes of valid UTF-8 followed by a `'\0'`.
    ///
    /// [free]: https://sqlite.org/c3ref/free.html
    /// [`sqlite3_str_finish`]: https://sqlite.org/c3ref/str_finish.html
    #[inline]
    pub const unsafe fn from_raw_parts(ptr: *const c_char, len: usize) -> Self {
        match ptr {
            _ if ptr.is_null() => panic!("expected non-null String pointer"),
            text => Self { text, len },
        }
    }

    /// Return an empty [`String`], consisting of just the null terminator.
    ///
    /// Returns an error if [`sqlite3_malloc64`] cannot allocate a single byte.
    ///
    /// [`sqlite3_malloc64`]: https://sqlite.org/c3ref/free.html
    pub fn empty() -> Result<Self> {
        let ptr = unsafe { sqlite3_malloc(1) } as *mut c_char;

        if ptr.is_null() {
            return Err(ErrorCategory::OutOfMemory.into());
        }

        Ok(unsafe { Self::from_raw_parts(ptr, 0) })
    }

    /// Consume the [`String`], returning the raw pointer and length.
    ///
    /// After calling this method, the caller is responsible for freeing the
    /// memory via [`sqlite3_free`].
    ///
    /// [`sqlite3_free`]: https://sqlite.org/c3ref/free.html
    #[inline]
    pub const fn into_raw_parts(self) -> (*const c_char, usize) {
        let (ptr, len) = (self.as_ptr(), self.len);
        mem::forget(self);

        (ptr, len)
    }

    /// Returns a pointer to the string data.
    #[inline]
    pub const fn as_ptr(&self) -> *const c_char {
        self.text
    }

    /// Returns the length of the string in bytes, not including the `'\0'`
    /// terminator.
    #[inline]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the string is empty.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the string as `&[u8]`, excluding the `'\0'` terminator.
    #[inline]
    pub const fn bytes_until_nul(&self) -> &[u8] {
        unsafe { slice::from_raw_parts::<'static, u8>(self.text as *const u8, self.len) }
    }

    /// Returns the string as `&[u8]`, including the `'\0'` terminator.
    #[inline]
    pub const fn bytes_with_nul(&self) -> &[u8] {
        unsafe { slice::from_raw_parts::<'static, u8>(self.text as *const u8, self.len + 1) }
    }

    /// Returns the string as a [`CStr`].
    #[inline]
    pub const fn as_c_str(&self) -> &CStr {
        unsafe { CStr::from_bytes_with_nul_unchecked(self.bytes_with_nul()) }
    }

    /// Returns the string as a `&str`.
    #[inline]
    pub fn as_str(&self) -> &str {
        // SAFETY: A String can only be constructed from valid UTF-8 slices, or
        // via unsafe functions which stipulate the content must be UTF-8.
        unsafe { str::from_utf8_unchecked(self.bytes_until_nul()) }
    }
}

impl fmt::Debug for String {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("String").field(&self.as_c_str()).finish()
    }
}

impl fmt::Display for String {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Deref for String {
    type Target = CStr;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_c_str()
    }
}

impl Drop for String {
    fn drop(&mut self) {
        // SAFETY: `self.ptr` was allocated via `sqlite3_malloc64` or
        // `sqlite3_str_finish`, both of which return memory that must be freed
        // via `sqlite3_free`.
        unsafe { sqlite3_free(self.as_ptr() as *mut c_void) }
    }
}

#[cfg(target_pointer_width = "64")]
const ENCODING_UTF8: core::ffi::c_uchar = SQLITE_UTF8 as core::ffi::c_uchar;

#[cfg_attr(
    target_pointer_width = "32",
    doc = "[Binds](Bind) a [`ffi::String`](String) via [`sqlite3_bind_text`]."
)]
#[cfg_attr(
    target_pointer_width = "64",
    doc = "[Binds](Bind) a [`ffi::String`](String) via [`sqlite3_bind_text64`]."
)]
///
/// The string's memory is transferred to SQLite, which will free it via
/// [`sqlite3_free`] when the binding is no longer needed.
impl<'b> Bind<'b> for String {
    unsafe fn bind<'s>(self, statement: &Statement<'s>, index: BindIndex) -> Result<()>
    where
        's: 'b,
    {
        let (ptr, len) = self.into_raw_parts();

        let destructor = sqlite3_destructor_type::new(sqlite3_free);

        #[cfg(target_pointer_width = "32")]
        bind! { sqlite3_bind_text(statement, index, ptr, len as c_int, destructor) }?;

        #[cfg(target_pointer_width = "64")]
        bind! { sqlite3_bind_text64(statement, index, ptr, len as sqlite3_uint64, destructor, ENCODING_UTF8) }?;

        Ok(())
    }
}

/// Dynamically creates a [`String`] via the [`sqlite3_str`] interface.
/// Strings built with `StringBuilder` are allocated via [`sqlite3_malloc64`].
///
/// # Example
///
/// ```
/// # use std::fmt::Write;
/// # use squire::ffi::StringBuilder;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut builder = StringBuilder::detached();
/// write!(builder, "Hello, {}!", "world")?;
/// assert_eq!("Hello, world!", builder.finish()?.as_str());
/// # Ok(())
/// # }
/// ```
///
/// [`sqlite3_str`]: https://sqlite.org/c3ref/str.html
/// [`sqlite3_malloc64`]: https://sqlite.org/c3ref/free.html
#[derive(Debug)]
pub struct StringBuilder {
    ptr: ptr::NonNull<sqlite3_str>,
}

impl StringBuilder {
    /// [Create][] a [`StringBuilder`].
    ///
    /// Strings built by this `StringBuilder` will have a maximum length of the
    /// [`SQLITE_LIMIT_LENGTH`] limit of the [`Connection`](super::Connection).
    ///
    /// [create]: https://sqlite.org/c3ref/str_new.html
    /// [`SQLITE_LIMIT_LENGTH`]: https://sqlite.org/c3ref/c_limit_attached.html#sqlitelimitlength
    #[doc(alias = "sqlite3_str_new")]
    pub fn new(conn: impl Connected) -> Self {
        // SAFETY: `sqlite3_str_new` always returns a valid pointer, even on OOM
        // (in which case it returns a special singleton that silently rejects
        // appends and returns NULL from `sqlite3_str_finish`).
        let ptr = unsafe { sqlite3_str_new(conn.as_connection_ptr()) };
        Self {
            ptr: unsafe { ptr::NonNull::new_unchecked(ptr) },
        }
    }

    /// [Create][] a new [`StringBuilder`] whose limits aren't defined
    /// by a [`Connection`](super::Connection).
    ///
    /// [create]: https://sqlite.org/c3ref/str_new.html
    pub fn detached() -> Self {
        // SAFETY: `sqlite3_str_new(NULL)` is explicitly supported.
        let ptr = unsafe { sqlite3_str_new(ptr::null_mut()) };

        Self {
            ptr: unsafe { ptr::NonNull::new_unchecked(ptr) },
        }
    }

    /// Returns the current error for this builder.
    #[doc(alias = "sqlite3_str_errcode")]
    pub fn error(&self) -> Option<Error> {
        let code = unsafe { sqlite3_str_errcode(self.as_ptr()) };
        Error::from_code(code)
    }

    /// Returns the current length of the string being built.
    #[doc(alias = "sqlite3_str_length")]
    pub fn len(&self) -> usize {
        unsafe { sqlite3_str_length(self.as_ptr()) as usize }
    }

    /// Returns `true` if the builder is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Extend the [`String`] by [appending](Append) text to it.
    #[doc(alias = "sqlite3_str_append")]
    pub fn append<T: Append>(&mut self, text: &T) {
        unsafe { text.append(self.ptr) };
    }

    /// Access the underlying [`*mut sqlite3_str`][string].
    ///
    /// [string]: https://sqlite.org/c3ref/str.html
    pub const fn as_ptr(&self) -> *mut sqlite3_str {
        self.ptr.as_ptr()
    }

    /// Consume the builder and return the [finished][] string.
    ///
    /// Returns an error if an allocation error occurred during building.
    ///
    /// [finished]: https://sqlite.org/c3ref/str_finish.html
    #[doc(alias = "sqlite3_str_finish")]
    pub fn finish(self) -> Result<String> {
        let len = self.len();
        let error = self.error();

        let ptr = unsafe { sqlite3_str_finish(self.as_ptr()) };
        mem::forget(self);

        if let Some(error) = error {
            unsafe { sqlite3_free(ptr as *mut c_void) };
            return Err(error);
        }

        if ptr.is_null() {
            // NULL with SQLITE_OK means empty string
            return String::empty();
        }

        Ok(unsafe { String::from_raw_parts(ptr, len) })
    }
}

/// A type which can be appended to a [`StringBuilder`].
pub trait Append {
    /// Append `self` to the [string builder](sqlite3_str).
    ///
    /// # Safety
    ///
    /// Callers must ensure the `string` pointer remains valid.
    unsafe fn append(&self, string: ptr::NonNull<sqlite3_str>);
}

impl Append for str {
    unsafe fn append(&self, string: ptr::NonNull<sqlite3_str>) {
        debug_assert!(self.len() <= c_int::MAX as usize);

        unsafe {
            sqlite3_str_append(
                string.as_ptr(),
                self.as_ptr() as *const c_char,
                self.len() as c_int,
            );
        }
    }
}

impl Append for CStr {
    unsafe fn append(&self, string: ptr::NonNull<sqlite3_str>) {
        unsafe {
            sqlite3_str_appendall(string.as_ptr(), self.as_ptr() as *const c_char);
        }
    }
}

impl Append for u8 {
    unsafe fn append(&self, string: ptr::NonNull<sqlite3_str>) {
        unsafe {
            sqlite3_str_appendchar(string.as_ptr(), 1, *self as c_char);
        }
    }
}

impl<A: Append> Extend<A> for StringBuilder {
    fn extend<T: IntoIterator<Item = A>>(&mut self, iter: T) {
        for item in iter {
            self.append(&item);
        }
    }
}

impl fmt::Write for StringBuilder {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        // SAFETY: `sqlite3_str_append` is safe to call with any valid
        // `sqlite3_str` pointer. The length must be non-negative.
        debug_assert!(s.len() <= c_int::MAX as usize);
        unsafe {
            sqlite3_str_append(self.as_ptr(), s.as_ptr() as *const c_char, s.len() as c_int);
        }
        // Errors are deferred to finish(); fmt::Write::write_str succeeds
        Ok(())
    }
}

impl Drop for StringBuilder {
    fn drop(&mut self) {
        // `finish` calls `mem::forget`; if we're here, the String is being
        // dropped without being `finish`ed. Free its memory.
        let ptr = unsafe { sqlite3_str_finish(self.as_ptr()) };
        unsafe { sqlite3_free(ptr as *mut c_void) };
    }
}

#[cfg(test)]
mod tests {
    use core::fmt::Write;

    use sqlite::{SQLITE_OPEN_CREATE, SQLITE_OPEN_READWRITE};

    use super::*;
    use crate::ffi::Connection;

    #[test]
    fn test_detached_builder() {
        let mut builder = StringBuilder::detached();
        write!(builder, "hello").unwrap();
        write!(builder, ", world!").unwrap();

        let string = builder.finish().expect("finish");
        assert_eq!(string.len(), 13);
        assert_eq!(string.as_str(), "hello, world!");
    }

    #[test]
    fn test_connected_builder() {
        let conn = Connection::open(
            c":memory:",
            SQLITE_OPEN_READWRITE | SQLITE_OPEN_CREATE,
            None,
        )
        .expect("open");

        {
            let mut builder = StringBuilder::new(&conn);
            write!(builder, "test string").unwrap();

            let string = builder.finish().expect("finish");
            assert_eq!(string.len(), 11);
            assert_eq!(string.as_str(), "test string");
        }

        conn.close().expect("close");
    }

    #[test]
    fn test_display() {
        let string = String::display("hello, world!").expect("string");
        assert_eq!(format!("{string}"), string.as_str());
    }

    #[test]
    fn test_debug() {
        let string = String::display("hello, world!").expect("string");
        assert_eq!(format!("{string:?}"), r#"String("hello, world!")"#);
    }

    #[test]
    fn test_display_formatting() {
        let mut builder = StringBuilder::detached();
        let value = 42u32;
        write!(builder, "value = {}", value).unwrap();

        let string = builder.finish().expect("finish");
        assert_eq!(string.len(), 10);
        assert_eq!(string.as_str(), "value = 42");
    }

    #[test]
    fn test_empty_string() {
        let builder = StringBuilder::detached();
        let string = builder.finish().expect("finish");
        assert!(string.is_empty());
        assert_eq!(string.as_str(), "");
    }

    #[test]
    fn test_bind_string() {
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
            let mut builder = StringBuilder::new(&conn);
            write!(builder, "hello from sqlite3_str").unwrap();
            let string = builder.finish().expect("finish");

            let index = BindIndex::new(1).expect("valid index");
            unsafe { stmt.bind(index, string) }.expect("bind");

            // Step and verify the value
            let has_row = unsafe { stmt.row() }.expect("row");
            assert!(has_row);

            let col = crate::types::ColumnIndex::new(0);
            let value: Borrowed<'_, str> = unsafe { stmt.fetch(col) };
            assert_eq!(&*value, "hello from sqlite3_str");
        }

        stmt.close().expect("close stmt");
        conn.close().expect("close conn");
    }
}
