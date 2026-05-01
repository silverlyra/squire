use core::{
    ffi::{CStr, c_char, c_int, c_void},
    fmt,
    iter::Extend,
    mem,
    ops::Deref,
    ptr, slice,
};
use std::io;

#[cfg(target_pointer_width = "32")]
use sqlite::sqlite3_bind_text;
#[cfg(all(feature = "functions", target_pointer_width = "32"))]
use sqlite::sqlite3_result_text;
#[cfg(all(feature = "functions", target_pointer_width = "64"))]
use sqlite::sqlite3_result_text64;
#[cfg(target_pointer_width = "64")]
use sqlite::{SQLITE_UTF8, sqlite3_bind_text64, sqlite3_uint64};
use sqlite::{
    sqlite3_destructor_type, sqlite3_free, sqlite3_str, sqlite3_str_append, sqlite3_str_appendall,
    sqlite3_str_appendchar, sqlite3_str_errcode, sqlite3_str_finish, sqlite3_str_length,
    sqlite3_str_new,
};

#[cfg(feature = "functions")]
use super::{bind::result, func::ContextRef};
use super::{
    bind::{Bind, bind},
    bytes::Bytes,
    connection::Connected,
    statement::Statement,
};
use crate::{
    error::{Error, ParameterError, Result},
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
    /// Allocate a null-terminated string on the [SQLite heap][free],
    /// and copy `value` into it.
    ///
    /// Returns an `Err` if `value` contains any `'\0'` bytes.
    ///
    /// [free]: https://sqlite.org/c3ref/free.html
    pub fn new(value: impl AsRef<str>) -> Result<Self> {
        let value = value.as_ref();
        debug_assert!(value.len() <= c_int::MAX as usize);

        let bytes = Bytes::allocate(value.len() + 1, |buf| {
            buf[..value.len()].copy_from_slice(value.as_bytes());
            buf[value.len()] = 0;

            let _ = CStr::from_bytes_with_nul(buf)?;
            Ok(())
        })?;

        // SAFETY: `value` was a `&str` already.
        Ok(unsafe { Self::from_bytes_unchecked(bytes) })
    }

    /// [Build](StringBuilder) a [`String`] from the [`Display`](fmt::Display)
    /// of a value.
    pub fn display<D: fmt::Display + ?Sized>(value: &D) -> Result<Self> {
        use fmt::Write as _;

        let mut builder = StringBuilder::new();
        write!(&mut builder, "{value}").map_err(|_| ParameterError::Bind)?;
        builder.finish()
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

    /// Interpret [`Bytes`] as a [`String`] containing valid UTF-8.
    pub fn from_bytes(bytes: Bytes) -> Result<Self> {
        let s = CStr::from_bytes_with_nul(bytes.data())?;
        let s = s.to_str()?;

        Ok(unsafe { Self::from_raw_parts(s.as_ptr() as *const c_char, s.len()) })
    }

    /// Interpret [`Bytes`] as a [`String`].
    ///
    /// # Safety
    ///
    /// `bytes` must not be empty, and its last byte must be `'\0'`.
    pub const unsafe fn from_bytes_unchecked(bytes: Bytes) -> Self {
        let (ptr, len) = bytes.into_raw_parts();
        unsafe { Self::from_raw_parts(ptr as *const c_char, len.unchecked_sub(1)) }
    }

    /// Return an empty [`String`], consisting of just the null terminator.
    ///
    /// Returns an error if [`sqlite3_malloc64`] cannot allocate a single byte.
    ///
    /// [`sqlite3_malloc64`]: https://sqlite.org/c3ref/free.html
    pub fn empty() -> Result<Self> {
        let bytes = Bytes::zeroed(1)?;
        Ok(unsafe { Self::from_bytes_unchecked(bytes) })
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
    unsafe fn bind_parameter<'s>(self, statement: &Statement<'s>, index: BindIndex) -> Result<()>
    where
        's: 'b,
    {
        let (ptr, len) = self.into_raw_parts();

        let destructor = sqlite3_destructor_type::free();

        #[cfg(target_pointer_width = "32")]
        bind! { sqlite3_bind_text(statement, index, ptr, len as c_int, destructor) }?;

        #[cfg(target_pointer_width = "64")]
        bind! { sqlite3_bind_text64(statement, index, ptr, len as sqlite3_uint64, destructor, ENCODING_UTF8) }?;

        Ok(())
    }

    #[cfg(feature = "functions")]
    unsafe fn bind_return<'c>(self, context: &ContextRef<'c>)
    where
        'b: 'c,
    {
        let (ptr, len) = self.into_raw_parts();

        let destructor = sqlite3_destructor_type::free();

        #[cfg(target_pointer_width = "32")]
        result! { sqlite3_result_text(context, ptr, len as c_int, destructor) }

        #[cfg(target_pointer_width = "64")]
        result! { sqlite3_result_text64(context, ptr, len as sqlite3_uint64, destructor, ENCODING_UTF8) }
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
/// let mut builder = StringBuilder::new();
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

#[allow(clippy::new_without_default)]
impl StringBuilder {
    /// [Create][] a [`StringBuilder`].
    ///
    /// [create]: https://sqlite.org/c3ref/str_new.html
    #[doc(alias = "sqlite3_str_new")]
    pub fn new() -> Self {
        // SAFETY: `sqlite3_str_new(NULL)` is explicitly supported.
        let ptr = unsafe { sqlite3_str_new(ptr::null_mut()) };

        Self {
            ptr: unsafe { ptr::NonNull::new_unchecked(ptr) },
        }
    }

    /// [Create][] a [`StringBuilder`].
    ///
    /// Strings built by this `StringBuilder` will have a maximum length of the
    /// [`SQLITE_LIMIT_LENGTH`] limit of the [`Connection`](super::Connection).
    ///
    /// [create]: https://sqlite.org/c3ref/str_new.html
    /// [`SQLITE_LIMIT_LENGTH`]: https://sqlite.org/c3ref/c_limit_attached.html#sqlitelimitlength
    pub fn with_limit(conn: impl Connected) -> Self {
        // SAFETY: `sqlite3_str_new` always returns a valid pointer, even on OOM
        // (in which case it returns a special singleton that silently rejects
        // appends and returns NULL from `sqlite3_str_finish`).
        let ptr = unsafe { sqlite3_str_new(conn.as_connection_ptr()) };
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

    /// Discards content in the [`StringBuilder`] beyond first `n` bytes.
    #[doc(alias = "sqlite3_str_truncate")]
    #[cfg(sqlite_has_truncate_string)]
    pub fn truncate(&mut self, n: usize) {
        debug_assert!(n <= c_int::MAX as usize);
        unsafe { sqlite::sqlite3_str_truncate(self.as_ptr(), n as c_int) }
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
        debug_assert!(self.len() + s.len() <= c_int::MAX as usize);
        unsafe {
            sqlite3_str_append(self.as_ptr(), s.as_ptr() as *const c_char, s.len() as c_int);
        }
        // Errors are deferred to finish(); fmt::Write::write_str succeeds
        Ok(())
    }
}

impl io::Write for StringBuilder {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // SAFETY: `sqlite3_str_append` is safe to call with any valid
        // `sqlite3_str` pointer. The length must be non-negative.
        debug_assert!(self.len() + buf.len() <= c_int::MAX as usize);
        unsafe {
            sqlite3_str_append(
                self.as_ptr(),
                buf.as_ptr() as *const c_char,
                buf.len() as c_int,
            );
        }
        // Errors are deferred to finish(); io::Write::write succeeds
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
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

/// A style of in-memory representation for text that SQLite recognizes, and in
/// some cases can [optimize].
///
/// [optimize]: https://sqlite.org/c3ref/c_any.html#sqliteutf8zt
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum StringRepresentation {
    /// A C-style string, where the content is followed by a `'\0'` byte.
    ///
    /// > The [`SQLITE_UTF8_ZT`][] encoding means that the input string (call it `z`)
    /// > is UTF-8 encoded and that it is zero-terminated. If the length parameter
    /// > (call it `n`) is non-negative, this encoding option means that the
    /// > caller guarantees that `z` array contains at least `n + 1` bytes and
    /// > that the `z[n]` byte has a value of zero.
    ///
    /// [`SQLITE_UTF8_ZT`]: https://sqlite.org/c3ref/c_any.html#sqliteutf8zt
    #[cfg(sqlite_has_utf8_zt)]
    NullTerminated = 0x43,
}

#[cfg(test)]
mod tests {
    use core::fmt::Write;

    use sqlite::{SQLITE_OPEN_CREATE, SQLITE_OPEN_READWRITE};

    use super::*;
    use crate::ffi::Connection;

    #[test]
    fn test_detached_builder() {
        let mut builder = StringBuilder::new();
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
            let mut builder = StringBuilder::with_limit(&conn);
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
        let mut builder = StringBuilder::new();
        let value = 42u32;
        write!(builder, "value = {}", value).unwrap();

        let string = builder.finish().expect("finish");
        assert_eq!(string.len(), 10);
        assert_eq!(string.as_str(), "value = 42");
    }

    #[test]
    fn test_empty_string() {
        let builder = StringBuilder::new();
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
            let mut builder = StringBuilder::with_limit(&conn);
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
