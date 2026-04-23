use sqlite::{
    sqlite3_column_blob, sqlite3_column_bytes, sqlite3_column_double, sqlite3_column_int,
    sqlite3_column_int64, sqlite3_column_text,
};
#[cfg(feature = "value")]
use sqlite::{
    sqlite3_column_value, sqlite3_value_blob, sqlite3_value_bytes, sqlite3_value_double,
    sqlite3_value_int, sqlite3_value_int64, sqlite3_value_pointer, sqlite3_value_text,
};

#[cfg(feature = "value")]
use super::pointer::{Pointee, Pointer, PointerMut};
use super::statement::Statement;
#[cfg(feature = "value")]
use super::value::{OpaqueValueRef, ValueRef};
use crate::types::{Borrowed, ColumnIndex, Type};

#[cfg_attr(
    not(feature = "value"),
    doc = "A type that can be read via a [`sqlite3_column_*`][column] function."
)]
#[cfg_attr(
    feature = "value",
    doc = "A type that can be read via a [`sqlite3_column_*`][column] or"
)]
#[cfg_attr(feature = "value", doc = "[`sqlite3_value_*`][value] function.")]
///
/// [column]: https://sqlite.org/c3ref/column_blob.html
#[cfg_attr(
    feature = "value",
    doc = "[value]: https://sqlite.org/c3ref/value_blob.html"
)]
pub trait Fetch<'r> {
    /// [Fetch][fetch] a column value from the [statement](Statement).
    ///
    /// [fetch]: https://sqlite.org/c3ref/column_blob.html
    ///
    /// # Safety
    ///
    /// Callers are responsible for managing the `ffi::Statement` lifecycle, and
    /// ensuring the [`ColumnIndex`] is in bounds.
    ///
    /// From the SQLite [reference][fetch]:
    ///
    /// > If the SQL statement does not currently point to a valid row, or if the
    /// > column index is out of range, the result is undefined. These routines
    /// > may only be called when the most recent call to `sqlite3_step` has
    /// > returned `SQLITE_ROW` and neither `sqlite3_reset` nor `sqlite3_finalize`
    /// > have been called subsequently.
    /// >
    /// > If any of these routines are called after `sqlite3_reset` or
    /// > `sqlite3_finalize` or after `sqlite3_step` has returned something other
    /// > than `SQLITE_ROW`, results are undefined. If `sqlite3_step` or
    /// > `sqlite3_reset` or `sqlite3_finalize` are called from a different thread
    /// > while any of these routines are pending, then the results are undefined.
    unsafe fn fetch_column<'c>(statement: &'r Statement<'c>, column: ColumnIndex) -> Self
    where
        'c: 'r;

    /// [Unpack][fetch] a dynamic [value](ValueRef) into this type.
    ///
    /// [fetch]: https://sqlite.org/c3ref/value_blob.html
    ///
    /// # Safety
    ///
    /// Callers are responsible for managing the `ffi::ValueRef` lifecycle.
    #[cfg(feature = "value")]
    #[cfg_attr(docsrs, doc(cfg(any(feature = "functions", feature = "value"))))]
    unsafe fn fetch_value<'c>(value: &'r ValueRef<'c>) -> Self
    where
        'c: 'r;
}

impl<'r> Fetch<'r> for i32 {
    unsafe fn fetch_column<'c>(statement: &'r Statement<'c>, column: ColumnIndex) -> Self
    where
        'c: 'r,
    {
        unsafe { sqlite3_column_int(statement.as_ptr(), column.value()) as i32 }
    }

    #[cfg(feature = "value")]
    unsafe fn fetch_value<'c>(value: &'r ValueRef<'c>) -> Self
    where
        'c: 'r,
    {
        unsafe { sqlite3_value_int(value.as_ptr()) as i32 }
    }
}

impl<'r> Fetch<'r> for i64 {
    unsafe fn fetch_column<'c>(statement: &'r Statement<'c>, column: ColumnIndex) -> Self
    where
        'c: 'r,
    {
        unsafe { sqlite3_column_int64(statement.as_ptr(), column.value()) as i64 }
    }

    #[cfg(feature = "value")]
    unsafe fn fetch_value<'c>(value: &'r ValueRef<'c>) -> Self
    where
        'c: 'r,
    {
        unsafe { sqlite3_value_int64(value.as_ptr()) as i64 }
    }
}

impl<'r> Fetch<'r> for f64 {
    unsafe fn fetch_column<'c>(statement: &'r Statement<'c>, column: ColumnIndex) -> Self
    where
        'c: 'r,
    {
        unsafe { sqlite3_column_double(statement.as_ptr(), column.value()) }
    }

    #[cfg(feature = "value")]
    unsafe fn fetch_value<'c>(value: &'r ValueRef<'c>) -> Self
    where
        'c: 'r,
    {
        unsafe { sqlite3_value_double(value.as_ptr()) }
    }
}

impl<'r> Fetch<'r> for Type {
    unsafe fn fetch_column<'c>(statement: &'r Statement<'c>, column: ColumnIndex) -> Self
    where
        'c: 'r,
    {
        unsafe { Type::fetch_column(statement, column) }
    }

    #[cfg(feature = "value")]
    unsafe fn fetch_value<'c>(value: &'r ValueRef<'c>) -> Self
    where
        'c: 'r,
    {
        unsafe { Type::fetch_value(value) }
    }
}

impl<'r, T> Fetch<'r> for Option<T>
where
    T: Fetch<'r>,
{
    unsafe fn fetch_column<'c>(statement: &'r Statement<'c>, column: ColumnIndex) -> Self
    where
        'c: 'r,
    {
        let column_type = unsafe { Type::fetch_column(statement, column) };

        if column_type.has_value() {
            Some(unsafe { T::fetch_column(statement, column) })
        } else {
            None
        }
    }

    #[cfg(feature = "value")]
    unsafe fn fetch_value<'c>(value: &'r ValueRef<'c>) -> Self
    where
        'c: 'r,
    {
        let value_type = unsafe { Type::fetch_value(value) };

        if value_type.has_value() {
            Some(unsafe { T::fetch_value(value) })
        } else {
            None
        }
    }
}

impl<'r> Fetch<'r> for Borrowed<'r, str> {
    unsafe fn fetch_column<'c>(statement: &'r Statement<'c>, column: ColumnIndex) -> Self
    where
        'c: 'r,
    {
        let data = unsafe { sqlite3_column_text(statement.as_ptr(), column.value()) };
        let len = unsafe { sqlite3_column_bytes(statement.as_ptr(), column.value()) };

        unsafe { Self::from_raw_str(data, len) }
    }

    #[cfg(feature = "value")]
    unsafe fn fetch_value<'c>(value: &'r ValueRef<'c>) -> Self
    where
        'c: 'r,
    {
        let data = unsafe { sqlite3_value_text(value.as_ptr()) };
        let len = unsafe { sqlite3_value_bytes(value.as_ptr()) };

        unsafe { Self::from_raw_str(data, len) }
    }
}

impl<'r> Fetch<'r> for Borrowed<'r, [u8]> {
    unsafe fn fetch_column<'c>(statement: &'r Statement<'c>, column: ColumnIndex) -> Self
    where
        'c: 'r,
    {
        let data = unsafe { sqlite3_column_blob(statement.as_ptr(), column.value()) };
        let len = unsafe { sqlite3_column_bytes(statement.as_ptr(), column.value()) };

        unsafe { Self::from_raw_bytes(data, len) }
    }

    #[cfg(feature = "value")]
    unsafe fn fetch_value<'c>(value: &'r ValueRef<'c>) -> Self
    where
        'c: 'r,
    {
        let data = unsafe { sqlite3_value_blob(value.as_ptr()) };
        let len = unsafe { sqlite3_value_bytes(value.as_ptr()) };

        unsafe { Self::from_raw_bytes(data, len) }
    }
}

#[cfg(feature = "value")]
impl<'r, T: Pointee> Fetch<'r> for Pointer<'r, T> {
    unsafe fn fetch_column<'c>(statement: &'r Statement<'c>, column: ColumnIndex) -> Self
    where
        'c: 'r,
    {
        let value = unsafe { sqlite3_column_value(statement.as_ptr(), column.value()) };
        let value = ValueRef::new(value).expect("sqlite3_value");

        unsafe {
            let value = core::mem::transmute::<&'_ ValueRef<'_>, &'r ValueRef<'c>>(&value);
            Self::fetch_value(value)
        }
    }

    unsafe fn fetch_value<'c>(value: &'r ValueRef<'c>) -> Self
    where
        'c: 'r,
    {
        let ptr = unsafe { sqlite3_value_pointer(value.as_ptr(), T::TYPE.as_ptr()) as *mut T };
        unsafe { Pointer::new(ptr as *const T).expect("non-null pointer") }
    }
}

#[cfg(feature = "value")]
impl<'r, T: Pointee> Fetch<'r> for PointerMut<'r, T> {
    unsafe fn fetch_column<'c>(statement: &'r Statement<'c>, column: ColumnIndex) -> Self
    where
        'c: 'r,
    {
        let value = unsafe { sqlite3_column_value(statement.as_ptr(), column.value()) };
        let value = ValueRef::new(value).expect("sqlite3_value");

        unsafe {
            let value = core::mem::transmute::<&'_ ValueRef<'_>, &'r ValueRef<'c>>(&value);
            Self::fetch_value(value)
        }
    }

    unsafe fn fetch_value<'c>(value: &'r ValueRef<'c>) -> Self
    where
        'c: 'r,
    {
        let ptr = unsafe { sqlite3_value_pointer(value.as_ptr(), T::TYPE.as_ptr()) as *mut T };
        unsafe { PointerMut::new(ptr).expect("non-null pointer") }
    }
}

#[cfg(feature = "value")]
impl<'r> Fetch<'r> for OpaqueValueRef<'r> {
    unsafe fn fetch_column<'c>(statement: &'r Statement<'c>, column: ColumnIndex) -> Self
    where
        'c: 'r,
    {
        let value = unsafe { sqlite3_column_value(statement.as_ptr(), column.value()) };
        OpaqueValueRef::new(value).expect("sqlite3_value")
    }

    unsafe fn fetch_value<'c>(value: &'r ValueRef<'c>) -> Self
    where
        'c: 'r,
    {
        value.as_opaque()
    }
}
