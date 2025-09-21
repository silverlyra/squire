use core::{
    ffi::{c_int, c_uchar, c_void},
    num::NonZero,
    ptr,
};

use super::statement::Statement;
use crate::{
    blob::Reservation,
    error::{Error, ErrorMessage, Result},
};

use sqlite::{
    SQLITE_STATIC, SQLITE_TRANSIENT, sqlite3_bind_double, sqlite3_bind_int, sqlite3_bind_int64,
    sqlite3_bind_null, sqlite3_destructor_type,
};
#[cfg(target_pointer_width = "64")]
use sqlite::{
    SQLITE_UTF8, sqlite3_bind_blob64, sqlite3_bind_text64, sqlite3_bind_zeroblob64, sqlite3_uint64,
};
#[cfg(target_pointer_width = "32")]
use sqlite::{sqlite3_bind_blob, sqlite3_bind_text, sqlite3_bind_zeroblob};

const ENCODING_UTF8: c_uchar = SQLITE_UTF8 as c_uchar;

/// A value which can be [bound as a parameter][bind] in SQLite [prepared
/// statements](Statement).
///
/// `squire::ffi::Bind` is the low-level `unsafe trait` whose implementations
/// directly call a [`sqlite3_bind_*`][bind] function in the C API. To make your
/// own user-defined types `Bind`able, implement [`squire::Bind`] instead.
///
/// Squire implements `ffi::Bind` only for types that the [SQLite C API][bind]
/// implements directly:
///
/// - [`f64`] (via [`sqlite3_bind_double`])
/// - [`i32`] (via [`sqlite3_bind_int`])
/// - [`i64`] (via [`sqlite3_bind_int64`])
#[cfg_attr(
    target_pointer_width = "32",
    doc = " - [`&str`](str) (via [`sqlite3_bind_text`])"
)]
#[cfg_attr(
    target_pointer_width = "64",
    doc = " - [`&str`](str) (via [`sqlite3_bind_text64`])"
)]
#[cfg_attr(
    target_pointer_width = "32",
    doc = " - [`&[u8]`](primitive@slice) (via [`sqlite3_bind_blob`])"
)]
#[cfg_attr(
    target_pointer_width = "64",
    doc = " - [`&[u8]`](primitive@slice) (via [`sqlite3_bind_blob64`])"
)]
#[cfg_attr(
    target_pointer_width = "32",
    doc = " - [`Reservation`] (via [`sqlite3_bind_zeroblob`])"
)]
#[cfg_attr(
    target_pointer_width = "64",
    doc = " - [`Reservation`] (via [`sqlite3_bind_zeroblob64`])"
)]
/// - [`None`] (via [`sqlite3_bind_null`])
///
/// The lifetime parameter `'b` represents the lifetime of the binding operation,
/// i.e., how long the statement reference passed to `bind` is valid for.
///
/// [bind]: https://sqlite.org/c3ref/bind_blob.html
pub unsafe trait Bind<'b> {
    /// Bind `self` as a SQLite prepared statement [parameter][bind].
    ///
    /// [bind]: https://sqlite.org/c3ref/bind_blob.html
    unsafe fn bind(self, statement: &'b Statement, index: Index) -> Result<()>;
}

/// Call a `sqlite3_bind_…` function and handle any possible [`Error`].
macro_rules! bind {
    { $fn:ident($stmt:expr, $index:expr, $($arg:expr),*) } => {
        {
            let result = unsafe { $fn($stmt.as_ptr(), $index.value(), $($arg),*) };

            match Error::<ErrorMessage>::from_connection($stmt, result) {
                Some(err) => Err(err),
                None => Ok(()),
            }
        }
    };
}

/// [Binds](Bind) an [`i32`] via [`sqlite3_bind_int`].
unsafe impl<'b> Bind<'b> for i32 {
    unsafe fn bind(self, statement: &'b Statement, index: Index) -> Result<()> {
        bind! { sqlite3_bind_int(statement, index, self) }
    }
}

/// [Binds](Bind) an [`i64`] via [`sqlite3_bind_int64`].
unsafe impl<'b> Bind<'b> for i64 {
    unsafe fn bind(self, statement: &'b Statement, index: Index) -> Result<()> {
        bind! { sqlite3_bind_int64(statement, index, self) }
    }
}

/// [Binds](Bind) an [`f64`] via [`sqlite3_bind_double`].
unsafe impl<'b> Bind<'b> for f64 {
    unsafe fn bind(self, statement: &'b Statement, index: Index) -> Result<()> {
        bind! { sqlite3_bind_double(statement, index, self) }
    }
}

#[cfg_attr(
    target_pointer_width = "32",
    doc = "[Binds](Bind) a [`&str`](str) via [`sqlite3_bind_text`]."
)]
#[cfg_attr(
    target_pointer_width = "64",
    doc = "[Binds](Bind) a [`&str`](str) via [`sqlite3_bind_text64`]."
)]
///
/// The [`SQLITE_TRANSIENT`] flag is used; SQLite will [clone][] the string's
/// bytes before `bind` returns. If you know the `&str` will outlive the
/// prepared statement, wrap the `&str` in [`Static`].
///
/// [clone]: https://sqlite.org/c3ref/c_static.html
unsafe impl<'b> Bind<'b> for &str {
    unsafe fn bind(self, statement: &'b Statement, index: Index) -> Result<()> {
        #[cfg(target_pointer_width = "32")]
        bind! { sqlite3_bind_text(statement, index, self.as_ptr() as *const i8, self.len() as c_int, SQLITE_TRANSIENT) }?;

        #[cfg(target_pointer_width = "64")]
        bind! { sqlite3_bind_text64(statement, index, self.as_ptr() as *const i8, self.len() as sqlite3_uint64, SQLITE_TRANSIENT, ENCODING_UTF8) }?;

        Ok(())
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
/// The [`SQLITE_TRANSIENT`] flag is used; SQLite will [clone][] the bytes
/// before `bind` returns. If you know the `&[u8]` will outlive the prepared
/// statement, wrap the `&[u8]` in [`Static`].
///
/// [clone]: https://sqlite.org/c3ref/c_static.html
unsafe impl<'b> Bind<'b> for &[u8] {
    unsafe fn bind(self, statement: &'b Statement, index: Index) -> Result<()> {
        #[cfg(target_pointer_width = "32")]
        bind! { sqlite3_bind_blob(statement, index, self.as_ptr() as *const c_void, self.len() as c_int, SQLITE_TRANSIENT) }

        #[cfg(target_pointer_width = "64")]
        bind! { sqlite3_bind_blob64(statement, index, self.as_ptr() as *const c_void, self.len() as sqlite3_uint64, SQLITE_TRANSIENT) }
    }
}

/// Marks a reference as outliving a SQLite [prepared
/// statement](crate::ffi::Statement), which SQLite does not need to copy to use
/// as a [`Bind`] value.
///
/// `Static` values are passed to SQLite with the [`SQLITE_STATIC`] flag, which
/// prevents SQLite from [cloning][] the data.
///
/// [cloning]: https://sqlite.org/c3ref/c_static.html
#[derive(Copy, Clone, Debug)]
#[repr(transparent)]
#[doc(alias = "SQLITE_STATIC")]
pub struct Static<'a, T: ?Sized>(&'a T);

impl<'a, T: ?Sized> Static<'a, T> {
    pub const fn new(value: &'a T) -> Self {
        Self(value)
    }

    #[inline]
    pub(super) fn as_ptr(&self) -> *const T {
        self.0 as *const T
    }
}

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
unsafe impl<'b, 'a: 'b> Bind<'b> for Static<'a, str> {
    unsafe fn bind(self, statement: &'b Statement, index: Index) -> Result<()> {
        #[cfg(target_pointer_width = "32")]
        bind! { sqlite3_bind_text(statement, index, self.as_ptr() as *const i8, self.0.len() as c_int, SQLITE_STATIC) }?;

        #[cfg(target_pointer_width = "64")]
        bind! { sqlite3_bind_text64(statement, index, self.as_ptr() as *const i8, self.0.len() as sqlite3_uint64, SQLITE_STATIC, ENCODING_UTF8) }?;

        Ok(())
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
unsafe impl<'b, 'a: 'b> Bind<'b> for Static<'a, [u8]> {
    unsafe fn bind(self, statement: &'b Statement, index: Index) -> Result<()> {
        #[cfg(target_pointer_width = "32")]
        bind! { sqlite3_bind_blob(statement, index, self.as_ptr() as *const c_void, self.0.len() as c_int, SQLITE_STATIC) }

        #[cfg(target_pointer_width = "64")]
        bind! { sqlite3_bind_blob64(statement, index, self.as_ptr() as *const c_void, self.0.len() as sqlite3_uint64, SQLITE_STATIC) }
    }
}

#[cfg_attr(
    target_pointer_width = "32",
    doc = "[Binds](Bind) a [`String`] via [`sqlite3_bind_text`]."
)]
#[cfg_attr(
    target_pointer_width = "64",
    doc = "[Binds](Bind) a [`String`] via [`sqlite3_bind_text64`]."
)]
///
/// The [`SQLITE_TRANSIENT`] flag is used; SQLite will [clone][] the string's
/// bytes before `bind` returns.
///
/// [clone]: https://sqlite.org/c3ref/c_static.html
unsafe impl<'b> Bind<'b> for String {
    unsafe fn bind(self, statement: &'b Statement, index: Index) -> Result<()> {
        unsafe { self.as_str().bind(statement, index) }
    }
}

#[cfg_attr(
    target_pointer_width = "32",
    doc = "[Binds](Bind) a `Vec<u8>` via [`sqlite3_bind_blob`]."
)]
#[cfg_attr(
    target_pointer_width = "64",
    doc = "[Binds](Bind) a `Vec<u8>` via [`sqlite3_bind_blob64`]."
)]
///
/// The [`SQLITE_TRANSIENT`] flag is used; SQLite will [clone][] the bytes
/// before `bind` returns.
///
/// [clone]: https://sqlite.org/c3ref/c_static.html
unsafe impl<'b> Bind<'b> for Vec<u8> {
    unsafe fn bind(self, statement: &'b Statement, index: Index) -> Result<()> {
        unsafe { self.as_slice().bind(statement, index) }
    }
}

#[cfg_attr(
    target_pointer_width = "32",
    doc = "[Binds](Bind) a [blob reservation](Reservation) via [`sqlite3_bind_zeroblob`]."
)]
#[cfg_attr(
    target_pointer_width = "64",
    doc = "[Binds](Bind) a [blob reservation](Reservation) via [`sqlite3_bind_zeroblob64`]."
)]
///
/// When a `Reservation` is [used](Bind) as a prepared [statement](Statement)
/// parameter, SQLite will create a `BLOB` of the [requested length](Reservation::len())
/// and set every byte in the blob to `\0`.
///
/// [clone]: https://sqlite.org/c3ref/c_static.html
unsafe impl<'b> Bind<'b> for Reservation {
    unsafe fn bind(self, statement: &'b Statement, index: Index) -> Result<()> {
        #[cfg(target_pointer_width = "32")]
        bind! { sqlite3_bind_zeroblob(statement, index, self.0 as c_int) }

        #[cfg(target_pointer_width = "64")]
        bind! { sqlite3_bind_zeroblob64(statement, index, self.len() as sqlite3_uint64) }
    }
}

/// [Binds](Bind) an [`Option`]. Bind the `Some` value if the option is present,
/// or bind `NULL` via [`sqlite3_bind_null`] if `None`.
unsafe impl<'b, T> Bind<'b> for Option<T>
where
    T: Bind<'b>,
{
    unsafe fn bind(self, statement: &'b Statement, index: Index) -> Result<()> {
        if let Some(value) = self {
            unsafe { value.bind(statement, index) }
        } else {
            bind! { sqlite3_bind_null(statement, index,) }
        }
    }
}

/// Create a SQLite [destructor](sqlite_destructor_type) for [bindable](Bind)
/// type `T`.
///
/// When SQLite invokes the destructor, Squire will call
/// [`drop_in_place`](ptr::drop_in_place) to [`Drop`] it.
pub const fn destructor<T>() -> sqlite3_destructor_type {
    sqlite3_destructor_type::new(destroy::<T>)
}

unsafe extern "C" fn destroy<T>(p: *mut c_void) {
    unsafe { ptr::drop_in_place(p) };
}

/// A SQLite [prepared statement](Statement) parameter index, used when
/// [binding](Bind) values to a statement.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
#[repr(transparent)]
pub struct Index(NonZero<c_int>);

impl Index {
    /// The first numbered binding parameter index (`1`).
    ///
    /// ```rust
    /// # use squire::ffi::Index;
    /// let first = Index::INITIAL;
    /// assert_eq!(usize::from(first), 1);
    /// ```
    pub const INITIAL: Self = unsafe { Self(NonZero::new_unchecked(1)) };

    pub const fn new(value: c_int) -> Result<Self, ()> {
        match NonZero::new(value) {
            Some(value) => Ok(Self(value)),
            None => Err(Error::range()),
        }
    }

    /// Access the underlying column index value as a C [`int`](c_int).
    #[inline]
    pub const fn value(&self) -> c_int {
        self.0.get()
    }

    /// Add `1` to this [`Index`].
    ///
    /// ```rust
    /// # use squire::ffi::Index;
    /// let first = Index::INITIAL;
    /// assert_eq!(usize::from(first), 1);
    ///
    /// let second = first.next();
    /// assert_eq!(usize::from(second), 2);
    /// ```
    pub const fn next(&self) -> Self {
        Self(unsafe { NonZero::new_unchecked(self.0.get() + 1) })
    }
}

impl From<Index> for i32 {
    fn from(index: Index) -> Self {
        index.value() as Self
    }
}

impl From<Index> for u32 {
    fn from(index: Index) -> Self {
        index.value() as Self
    }
}

impl From<Index> for i64 {
    fn from(index: Index) -> Self {
        index.value() as Self
    }
}

impl From<Index> for u64 {
    fn from(index: Index) -> Self {
        index.value() as Self
    }
}

impl From<Index> for isize {
    fn from(index: Index) -> Self {
        index.value() as Self
    }
}

impl From<Index> for usize {
    fn from(index: Index) -> Self {
        index.value() as Self
    }
}

impl TryFrom<i32> for Index {
    type Error = Error<()>;

    fn try_from(value: i32) -> Result<Self, ()> {
        Self::new(value as c_int)
    }
}

impl TryFrom<i64> for Index {
    type Error = Error<()>;

    fn try_from(value: i64) -> Result<Self, ()> {
        match c_int::try_from(value) {
            Ok(value) => Self::new(value),
            Err(_) => Err(Error::range()),
        }
    }
}

impl TryFrom<isize> for Index {
    type Error = Error<()>;

    fn try_from(value: isize) -> Result<Self, ()> {
        Self::new(value as c_int)
    }
}

impl TryFrom<u32> for Index {
    type Error = Error<()>;

    fn try_from(value: u32) -> Result<Self, ()> {
        match c_int::try_from(value) {
            Ok(value) => Self::new(value),
            Err(_) => Err(Error::range()),
        }
    }
}

impl TryFrom<u64> for Index {
    type Error = Error<()>;

    fn try_from(value: u64) -> Result<Self, ()> {
        match c_int::try_from(value) {
            Ok(value) => Self::new(value),
            Err(_) => Err(Error::range()),
        }
    }
}

impl TryFrom<usize> for Index {
    type Error = Error<()>;

    fn try_from(value: usize) -> Result<Self, ()> {
        match c_int::try_from(value) {
            Ok(value) => Self::new(value),
            Err(_) => Err(Error::range()),
        }
    }
}

#[cfg(feature = "lang-step-trait")]
impl core::iter::Step for Index {
    fn steps_between(start: &Self, end: &Self) -> (usize, Option<usize>) {
        if start.0.get() <= end.0.get() {
            let steps = (end.0.get() - start.0.get()) as usize;
            (steps, Some(steps))
        } else {
            (0, None)
        }
    }

    fn forward_checked(start: Self, count: usize) -> Option<Self> {
        let count = c_int::try_from(count).ok()?;
        let new_value = start.0.get().checked_add(count)?;
        NonZero::new(new_value).map(Self)
    }

    fn backward_checked(start: Self, count: usize) -> Option<Self> {
        let count = c_int::try_from(count).ok()?;
        let new_value = start.0.get().checked_sub(count)?;
        NonZero::new(new_value).map(Self)
    }
}
