use core::{
    ffi::{c_int, c_uchar, c_void},
    mem::forget,
    num::NonZero,
};

use super::statement::Statement;
use crate::{
    call,
    error::{Error, Result},
};

use sqlite::{SQLITE_STATIC, sqlite3_bind_int, sqlite3_bind_int64, sqlite3_destructor_type};
#[cfg(target_pointer_width = "64")]
use sqlite::{SQLITE_UTF8, sqlite3_bind_blob64, sqlite3_bind_text64, sqlite3_uint64};
#[cfg(target_pointer_width = "32")]
use sqlite::{sqlite3_bind_blob, sqlite3_bind_text};

const ENCODING_UTF8: c_uchar = SQLITE_UTF8 as c_uchar;

/// A value which can be [bound as a parameter][bind] in SQLite [prepared
/// statements](crate::Statement).
///
/// `squire::ffi::Bind` is the low-level `unsafe trait` whose implementations
/// directly call a [`sqlite3_bind_*`][bind] function in the C API. To make your
/// own user-defined types `Bind`able, implement [`squire::Bind`] instead.
///
/// [bind]: https://sqlite.org/c3ref/bind_blob.html
pub unsafe trait Bind<'s> {
    /// Bind `self` as a SQLite prepared statement [parameter][bind].
    ///
    /// [bind]: https://sqlite.org/c3ref/bind_blob.html
    unsafe fn bind(self, statement: &'s Statement, index: Index) -> Result<()>;
}

unsafe impl<'s> Bind<'s> for i32 {
    unsafe fn bind(self, statement: &'s Statement, index: Index) -> Result<()> {
        call! { sqlite3_bind_int(statement.as_ptr(), index.value(), self) }
    }
}

unsafe impl<'s> Bind<'s> for u32 {
    #[inline]
    unsafe fn bind(self, statement: &'s Statement, index: Index) -> Result<()> {
        unsafe { (self as i64).bind(statement, index) }
    }
}

unsafe impl<'s> Bind<'s> for i64 {
    unsafe fn bind(self, statement: &'s Statement, index: Index) -> Result<()> {
        call! { sqlite3_bind_int64(statement.as_ptr(), index.value(), self) }
    }
}

unsafe impl<'s> Bind<'s> for u64 {
    #[inline]
    unsafe fn bind(self, statement: &'s Statement, index: Index) -> Result<()> {
        match i64::try_from(self) {
            Ok(value) => unsafe { value.bind(statement, index) },
            Err(_) => Err(Error::range()),
        }
    }
}

/// A SQLite [prepared statement](Statement) parameter index, used when
/// [binding](Bind) values to a statement.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
#[repr(transparent)]
pub struct Index(NonZero<c_int>);

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

unsafe impl<'s, 'a: 's> Bind<'s> for Static<'a, str> {
    unsafe fn bind(self, statement: &'s Statement, index: Index) -> Result<()> {
        #[cfg(target_pointer_width = "32")]
        call! { sqlite3_bind_text(statement.as_ptr(), index.value(), self.as_ptr() as *const i8, self.0.len() as c_int, SQLITE_STATIC) }?;

        #[cfg(target_pointer_width = "64")]
        call! { sqlite3_bind_text64(statement.as_ptr(), index.value(), self.as_ptr() as *const i8, self.0.len() as sqlite3_uint64, SQLITE_STATIC, ENCODING_UTF8) }?;

        Ok(())
    }
}

unsafe impl<'s, 'a: 's> Bind<'s> for Static<'a, [u8]> {
    unsafe fn bind(self, statement: &'s Statement, index: Index) -> Result<()> {
        #[cfg(target_pointer_width = "32")]
        call! { sqlite3_bind_blob(statement.as_ptr(), index.value(), self.as_ptr() as *const c_void, self.0.len() as c_int, SQLITE_STATIC) }

        #[cfg(target_pointer_width = "64")]
        call! { sqlite3_bind_blob64(statement.as_ptr(), index.value(), self.as_ptr() as *const c_void, self.0.len() as sqlite3_uint64, SQLITE_STATIC) }
    }
}

impl Index {
    pub const fn new(value: c_int) -> Result<Self> {
        match NonZero::new(value) {
            Some(value) => Ok(Self(value)),
            None => Err(Error::range()),
        }
    }

    const fn value(&self) -> c_int {
        self.0.get()
    }

    pub const fn next(&self) -> Self {
        Self(unsafe { NonZero::new_unchecked(self.0.get() + 1) })
    }
}

impl TryFrom<i32> for Index {
    type Error = Error;

    fn try_from(value: i32) -> Result<Self> {
        Self::new(value as c_int)
    }
}

impl TryFrom<i64> for Index {
    type Error = Error;

    fn try_from(value: i64) -> Result<Self> {
        match c_int::try_from(value) {
            Ok(value) => Self::new(value),
            Err(_) => Err(Error::range()),
        }
    }
}

impl TryFrom<isize> for Index {
    type Error = Error;

    fn try_from(value: isize) -> Result<Self> {
        Self::new(value as c_int)
    }
}

impl TryFrom<u32> for Index {
    type Error = Error;

    fn try_from(value: u32) -> Result<Self> {
        match c_int::try_from(value) {
            Ok(value) => Self::new(value),
            Err(_) => Err(Error::range()),
        }
    }
}

impl TryFrom<u64> for Index {
    type Error = Error;

    fn try_from(value: u64) -> Result<Self> {
        match c_int::try_from(value) {
            Ok(value) => Self::new(value),
            Err(_) => Err(Error::range()),
        }
    }
}

impl TryFrom<usize> for Index {
    type Error = Error;

    fn try_from(value: usize) -> Result<Self> {
        match c_int::try_from(value) {
            Ok(value) => Self::new(value),
            Err(_) => Err(Error::range()),
        }
    }
}

#[cfg(feature = "nightly")]
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
