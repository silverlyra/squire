use core::{
    ffi::{c_char, c_uchar, c_void},
    ptr,
};

#[cfg(feature = "functions")]
use super::func::ContextRef;
use super::{pointer::Pointee, statement::Statement};
use crate::{
    blob::Reservation,
    error::{Error, Result},
    types::{BindIndex, Borrowed},
};

use sqlite::{
    SQLITE_STATIC, SQLITE_TRANSIENT, sqlite3_bind_double, sqlite3_bind_int, sqlite3_bind_int64,
    sqlite3_bind_null, sqlite3_bind_pointer, sqlite3_destructor_type,
};
#[cfg(target_pointer_width = "64")]
use sqlite::{
    SQLITE_UTF8, sqlite3_bind_blob64, sqlite3_bind_text64, sqlite3_bind_zeroblob64, sqlite3_uint64,
};
#[cfg(target_pointer_width = "32")]
use sqlite::{sqlite3_bind_blob, sqlite3_bind_text, sqlite3_bind_zeroblob};
#[cfg(all(feature = "functions", target_pointer_width = "32"))]
use sqlite::{sqlite3_result_blob, sqlite3_result_text, sqlite3_result_zeroblob};
#[cfg(all(feature = "functions", target_pointer_width = "64"))]
use sqlite::{sqlite3_result_blob64, sqlite3_result_text64, sqlite3_result_zeroblob64};
#[cfg(feature = "functions")]
use sqlite::{
    sqlite3_result_double, sqlite3_result_int, sqlite3_result_int64, sqlite3_result_null,
    sqlite3_result_pointer,
};

#[cfg(target_pointer_width = "64")]
const ENCODING_UTF8: c_uchar = SQLITE_UTF8 as c_uchar;

/// A type that can be [bound][bind] as a SQLite [prepared statement](Statement)
#[cfg_attr(not(feature = "functions"), doc = "parameter.")]
#[cfg_attr(
    feature = "functions",
    doc = "parameter, or [returned][result] from a SQL function."
)]
///
/// `squire::ffi::Bind` is the low-level `trait` whose implementations directly
/// call a [`sqlite3_bind_*`][bind] function in the C API. To make your own
/// user-defined types `Bind`able, implement [`Bind`](crate::Bind) instead.
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
/// - [`None`](core::option) (via [`sqlite3_bind_null`])
///
/// The lifetime parameter `'b` represents the lifetime for which SQLite may
/// access the bound data.
///
/// [bind]: https://sqlite.org/c3ref/bind_blob.html
#[cfg_attr(
    feature = "functions",
    doc = "[result]: https://sqlite.org/c3ref/result_blob.html"
)]
pub trait Bind<'b> {
    /// Bind `self` as a SQLite prepared statement [parameter][bind].
    ///
    /// [bind]: https://sqlite.org/c3ref/bind_blob.html
    ///
    /// # Safety
    ///
    /// Implementations access the `sqlite3_bind_*` API’s directly. If these
    /// API’s are used to bind a pointer non-`SQLITE_TRANSIENT`ly, the caller is
    /// responsible for ensuring the pointer remains valid for the duration of
    /// the binding; and if a [destructor](sqlite3_destructor_type) is used, for
    /// SQLite to call it at the end of the binding lifecycle.
    unsafe fn bind_parameter<'c>(self, statement: &Statement<'c>, index: BindIndex) -> Result<()>
    where
        'c: 'b;

    /// Bind `self` as the [return value][result] of a SQLite function.
    ///
    /// [result]: https://sqlite.org/c3ref/result_blob.html
    ///
    /// # Safety
    ///
    /// Implementations access the `sqlite3_result_*` API’s directly. If these
    /// API’s are used to return a pointer non-`SQLITE_TRANSIENT`ly, the caller
    /// is responsible for ensuring the pointer remains valid for the duration
    /// of the context; and if a [destructor](sqlite3_destructor_type) is used,
    /// for SQLite to call it at the end of the function evaluation.
    #[cfg(feature = "functions")]
    #[cfg_attr(docsrs, doc(cfg(feature = "functions")))]
    unsafe fn bind_return<'c>(self, context: &ContextRef<'c>)
    where
        'b: 'c;
}

/// Call a `sqlite3_bind_…` function and handle any possible [`Error`].
macro_rules! bind {
    { $fn:ident($stmt:expr, $index:expr, $($arg:expr),*) } => {
        {
            let result = unsafe { $fn($stmt.as_ptr(), $index.value(), $($arg),*) };

            match Error::from_connection($stmt, result) {
                None => Ok(()),
                Some(err) => Err(err),
            }
        }
    };
}

/// Call a `sqlite3_result_…` function.
#[cfg(feature = "functions")]
macro_rules! result {
    { $fn:ident($ctx:expr, $($arg:expr),*) } => {
        unsafe { $fn($ctx.as_ptr(), $($arg),*) }
    };
}

pub(crate) use bind;
#[cfg(feature = "functions")]
pub(crate) use result;

/// [Binds](Bind) an [`i32`] via [`sqlite3_bind_int`].
impl<'b> Bind<'b> for i32 {
    unsafe fn bind_parameter<'c>(self, statement: &Statement<'c>, index: BindIndex) -> Result<()>
    where
        'c: 'b,
    {
        bind! { sqlite3_bind_int(statement, index, self) }
    }

    #[cfg(feature = "functions")]
    unsafe fn bind_return<'c>(self, context: &ContextRef<'c>)
    where
        'b: 'c,
    {
        result! { sqlite3_result_int(context, self) }
    }
}

/// [Binds](Bind) an [`i64`] via [`sqlite3_bind_int64`].
impl<'b> Bind<'b> for i64 {
    unsafe fn bind_parameter<'c>(self, statement: &Statement<'c>, index: BindIndex) -> Result<()>
    where
        'c: 'b,
    {
        bind! { sqlite3_bind_int64(statement, index, self) }
    }

    #[cfg(feature = "functions")]
    unsafe fn bind_return<'c>(self, context: &ContextRef<'c>)
    where
        'b: 'c,
    {
        result! { sqlite3_result_int64(context, self) }
    }
}

/// [Binds](Bind) an [`f64`] via [`sqlite3_bind_double`].
impl<'b> Bind<'b> for f64 {
    unsafe fn bind_parameter<'c>(self, statement: &Statement<'c>, index: BindIndex) -> Result<()>
    where
        'c: 'b,
    {
        bind! { sqlite3_bind_double(statement, index, self) }
    }

    #[cfg(feature = "functions")]
    unsafe fn bind_return<'c>(self, context: &ContextRef<'c>)
    where
        'b: 'c,
    {
        result! { sqlite3_result_double(context, self) }
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
/// prepared statement, wrap the `&str` in [`Borrowed`](crate::Borrowed).
///
/// [clone]: https://sqlite.org/c3ref/c_static.html
impl<'b> Bind<'b> for &str {
    unsafe fn bind_parameter<'c>(self, statement: &Statement<'c>, index: BindIndex) -> Result<()>
    where
        'c: 'b,
    {
        #[cfg(target_pointer_width = "32")]
        bind! { sqlite3_bind_text(statement, index, self.as_ptr() as *const c_char, self.len() as c_int, SQLITE_TRANSIENT) }?;

        #[cfg(target_pointer_width = "64")]
        bind! { sqlite3_bind_text64(statement, index, self.as_ptr() as *const c_char, self.len() as sqlite3_uint64, SQLITE_TRANSIENT, ENCODING_UTF8) }?;

        Ok(())
    }

    #[cfg(feature = "functions")]
    unsafe fn bind_return<'c>(self, context: &ContextRef<'c>)
    where
        'b: 'c,
    {
        #[cfg(target_pointer_width = "32")]
        result! { sqlite3_result_text(context, self.as_ptr() as *const c_char, self.len() as c_int, SQLITE_TRANSIENT) }

        #[cfg(target_pointer_width = "64")]
        result! { sqlite3_result_text64(context, self.as_ptr() as *const c_char, self.len() as sqlite3_uint64, SQLITE_TRANSIENT, ENCODING_UTF8) }
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
/// statement, wrap the `&[u8]` in [`Borrowed`](crate::Borrowed).
///
/// [clone]: https://sqlite.org/c3ref/c_static.html
impl<'b> Bind<'b> for &[u8] {
    unsafe fn bind_parameter<'c>(self, statement: &Statement<'c>, index: BindIndex) -> Result<()>
    where
        'c: 'b,
    {
        #[cfg(target_pointer_width = "32")]
        bind! { sqlite3_bind_blob(statement, index, self.as_ptr() as *const c_void, self.len() as c_int, SQLITE_TRANSIENT) }

        #[cfg(target_pointer_width = "64")]
        bind! { sqlite3_bind_blob64(statement, index, self.as_ptr() as *const c_void, self.len() as sqlite3_uint64, SQLITE_TRANSIENT) }
    }

    #[cfg(feature = "functions")]
    unsafe fn bind_return<'c>(self, context: &ContextRef<'c>)
    where
        'b: 'c,
    {
        #[cfg(target_pointer_width = "32")]
        result! { sqlite3_result_blob(context, self.as_ptr() as *const c_void, self.len() as c_int, SQLITE_TRANSIENT) }

        #[cfg(target_pointer_width = "64")]
        result! { sqlite3_result_blob64(context, self.as_ptr() as *const c_void, self.len() as sqlite3_uint64, SQLITE_TRANSIENT) }
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
impl<'b> Bind<'b> for String {
    unsafe fn bind_parameter<'c>(self, statement: &Statement<'c>, index: BindIndex) -> Result<()>
    where
        'c: 'b,
    {
        unsafe { self.as_str().bind_parameter(statement, index) }
    }

    #[cfg(feature = "functions")]
    unsafe fn bind_return<'c>(self, context: &ContextRef<'c>)
    where
        'b: 'c,
    {
        unsafe { self.as_str().bind_return(context) }
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
impl<'b> Bind<'b> for Vec<u8> {
    unsafe fn bind_parameter<'c>(self, statement: &Statement<'c>, index: BindIndex) -> Result<()>
    where
        'c: 'b,
    {
        unsafe { self.as_slice().bind_parameter(statement, index) }
    }

    #[cfg(feature = "functions")]
    unsafe fn bind_return<'c>(self, context: &ContextRef<'c>)
    where
        'b: 'c,
    {
        unsafe { self.as_slice().bind_return(context) }
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
impl<'b> Bind<'b> for Reservation {
    unsafe fn bind_parameter<'c>(self, statement: &Statement<'c>, index: BindIndex) -> Result<()>
    where
        'c: 'b,
    {
        #[cfg(target_pointer_width = "32")]
        bind! { sqlite3_bind_zeroblob(statement, index, self.0 as c_int) }

        #[cfg(target_pointer_width = "64")]
        bind! { sqlite3_bind_zeroblob64(statement, index, self.len() as sqlite3_uint64) }
    }

    #[cfg(feature = "functions")]
    unsafe fn bind_return<'c>(self, context: &ContextRef<'c>)
    where
        'b: 'c,
    {
        #[cfg(target_pointer_width = "32")]
        result! { sqlite3_result_zeroblob(context, self.0 as c_int) }

        #[cfg(target_pointer_width = "64")]
        result! { sqlite3_result_zeroblob64(context, self.len() as sqlite3_uint64) };
    }
}

/// [Binds](Bind) an [`Option`]. Bind the `Some` value if the option is present,
/// or bind `NULL` via [`sqlite3_bind_null`] if `None`.
impl<'b, T> Bind<'b> for Option<T>
where
    T: Bind<'b>,
{
    unsafe fn bind_parameter<'c>(self, statement: &Statement<'c>, index: BindIndex) -> Result<()>
    where
        'c: 'b,
    {
        if let Some(value) = self {
            unsafe { value.bind_parameter(statement, index) }
        } else {
            unsafe { Null.bind_parameter(statement, index) }
        }
    }

    #[cfg(feature = "functions")]
    unsafe fn bind_return<'c>(self, context: &ContextRef<'c>)
    where
        'b: 'c,
    {
        if let Some(value) = self {
            unsafe { value.bind_return(context) }
        } else {
            unsafe { Null.bind_return(context) }
        }
    }
}

#[cfg_attr(
    target_pointer_width = "32",
    doc = "[Binds](Bind) a [borrowed](Borrowed) [`str`](str) via [`sqlite3_bind_text`]."
)]
#[cfg_attr(
    target_pointer_width = "64",
    doc = "[Binds](Bind) a [borrowed](Borrowed) [`str`](str) via [`sqlite3_bind_text64`]."
)]
///
/// The [`SQLITE_STATIC`] flag is used; SQLite will read the string's bytes
/// without [cloning][] them.
///
/// [cloning]: https://sqlite.org/c3ref/c_static.html
impl<'b, 'a: 'b> Bind<'b> for Borrowed<'a, str> {
    unsafe fn bind_parameter<'c>(self, statement: &Statement<'c>, index: BindIndex) -> Result<()>
    where
        'c: 'b,
    {
        #[cfg(target_pointer_width = "32")]
        bind! { sqlite3_bind_text(statement, index, self.as_ptr() as *const c_char, self.len() as c_int, SQLITE_STATIC) }?;

        #[cfg(target_pointer_width = "64")]
        bind! { sqlite3_bind_text64(statement, index, self.as_ptr() as *const c_char, self.len() as sqlite3_uint64, SQLITE_STATIC, ENCODING_UTF8) }?;

        Ok(())
    }

    #[cfg(feature = "functions")]
    unsafe fn bind_return<'c>(self, context: &ContextRef<'c>)
    where
        'b: 'c,
    {
        #[cfg(target_pointer_width = "32")]
        result! { sqlite3_result_text(context, self.as_ptr() as *const c_char, self.len() as c_int, SQLITE_STATIC) }

        #[cfg(target_pointer_width = "64")]
        result! { sqlite3_result_text64(context, self.as_ptr() as *const c_char, self.len() as sqlite3_uint64, SQLITE_STATIC, ENCODING_UTF8) }
    }
}

#[cfg_attr(
    target_pointer_width = "32",
    doc = "[Binds](Bind) a [borrowed](Borrowed) [`[u8]`](primitive@slice) via [`sqlite3_bind_blob`]."
)]
#[cfg_attr(
    target_pointer_width = "64",
    doc = "[Binds](Bind) a [borrowed](Borrowed) [`[u8]`](primitive@slice) via [`sqlite3_bind_blob64`]."
)]
///
/// The [`SQLITE_STATIC`] flag is used; SQLite will read the bytes without
/// [cloning][] them.
///
/// [cloning]: https://sqlite.org/c3ref/c_static.html
impl<'b, 'a: 'b> Bind<'b> for Borrowed<'a, [u8]> {
    unsafe fn bind_parameter<'c>(self, statement: &Statement<'c>, index: BindIndex) -> Result<()>
    where
        'c: 'b,
    {
        #[cfg(target_pointer_width = "32")]
        bind! { sqlite3_bind_blob(statement, index, self.as_ptr() as *const c_void, self.len() as c_int, SQLITE_STATIC) }

        #[cfg(target_pointer_width = "64")]
        bind! { sqlite3_bind_blob64(statement, index, self.as_ptr() as *const c_void, self.len() as sqlite3_uint64, SQLITE_STATIC) }
    }

    #[cfg(feature = "functions")]
    unsafe fn bind_return<'c>(self, context: &ContextRef<'c>)
    where
        'b: 'c,
    {
        #[cfg(target_pointer_width = "32")]
        result! { sqlite3_result_blob(context, self.as_ptr() as *const c_void, self.len() as c_int, SQLITE_STATIC) }

        #[cfg(target_pointer_width = "64")]
        result! { sqlite3_result_blob64(context, self.as_ptr() as *const c_void, self.len() as sqlite3_uint64, SQLITE_STATIC) }
    }
}

/// [Binds](Bind) a reference using the [pointer passing interface].
///
/// [pointer passing interface]: https://sqlite.org/bindptr.html
impl<'b, 'a: 'b, T: Pointee + ?Sized> Bind<'b> for &'a T {
    unsafe fn bind_parameter<'c>(self, statement: &Statement<'c>, index: BindIndex) -> Result<()>
    where
        'c: 'b,
    {
        let pointer = self as *const T;
        bind! { sqlite3_bind_pointer(statement, index, pointer as *mut c_void, T::TYPE.as_ptr(), SQLITE_STATIC) }
    }

    #[cfg(feature = "functions")]
    unsafe fn bind_return<'c>(self, context: &ContextRef<'c>)
    where
        'b: 'c,
    {
        let pointer = self as *const T;
        result! { sqlite3_result_pointer(context, pointer as *mut c_void, T::TYPE.as_ptr(), SQLITE_STATIC) }
    }
}

/// [Binds](Bind) a mutable reference using the [pointer passing interface].
///
/// [pointer passing interface]: https://sqlite.org/bindptr.html
impl<'b, 'a: 'b, T: Pointee + ?Sized> Bind<'b> for &'a mut T {
    unsafe fn bind_parameter<'c>(self, statement: &Statement<'c>, index: BindIndex) -> Result<()>
    where
        'c: 'b,
    {
        let pointer = self as *const T;
        bind! { sqlite3_bind_pointer(statement, index, pointer as *mut c_void, T::TYPE.as_ptr(), SQLITE_STATIC) }
    }

    #[cfg(feature = "functions")]
    unsafe fn bind_return<'c>(self, context: &ContextRef<'c>)
    where
        'b: 'c,
    {
        let pointer = self as *const T;
        result! { sqlite3_result_pointer(context, pointer as *mut c_void, T::TYPE.as_ptr(), SQLITE_STATIC) }
    }
}

/// [Binds](Bind) an owned value using the [pointer passing interface].
///
/// [pointer passing interface]: https://sqlite.org/bindptr.html
impl<'b, T: Pointee> Bind<'b> for Box<T> {
    unsafe fn bind_parameter<'c>(self, statement: &Statement<'c>, index: BindIndex) -> Result<()>
    where
        'c: 'b,
    {
        let pointer = Box::into_raw(self);
        let destructor = sqlite3_destructor_type::new(destroy_box::<T>);
        bind! { sqlite3_bind_pointer(statement, index, pointer as *mut c_void, T::TYPE.as_ptr(), destructor) }
    }

    #[cfg(feature = "functions")]
    unsafe fn bind_return<'c>(self, context: &ContextRef<'c>)
    where
        'b: 'c,
    {
        let pointer = Box::into_raw(self);
        let destructor = sqlite3_destructor_type::new(destroy_box::<T>);
        result! { sqlite3_result_pointer(context, pointer as *mut c_void, T::TYPE.as_ptr(), destructor) }
    }
}

pub(crate) struct Null;

impl<'b> Bind<'b> for Null {
    unsafe fn bind_parameter<'c>(self, statement: &Statement<'c>, index: BindIndex) -> Result<()>
    where
        'c: 'b,
    {
        bind! { sqlite3_bind_null(statement, index,) }
    }

    #[cfg(feature = "functions")]
    unsafe fn bind_return<'c>(self, context: &ContextRef<'c>)
    where
        'b: 'c,
    {
        result! { sqlite3_result_null(context,) }
    }
}

/// Create a SQLite [destructor](sqlite3_destructor_type) for [bindable](Bind)
/// type `T`.
///
/// When SQLite invokes the destructor, Squire will call
/// [`drop_in_place`](ptr::drop_in_place) to [`Drop`] it.
pub const fn destructor<T>() -> sqlite3_destructor_type {
    sqlite3_destructor_type::new(destroy::<T>)
}

unsafe extern "C" fn destroy<T>(p: *mut c_void) {
    unsafe { ptr::drop_in_place(p as *mut T) };
}

pub(super) unsafe extern "C" fn destroy_box<T>(p: *mut c_void) {
    let _ = unsafe { Box::from_raw(p as *mut T) };
}
