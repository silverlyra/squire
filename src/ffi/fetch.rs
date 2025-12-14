use sqlite::{sqlite3_column_double, sqlite3_column_int, sqlite3_column_int64};

use super::statement::Statement;
use crate::types::{ColumnIndex, Type};

/// A type that can be read via an [`sqlite3_column_*`][column] function.
///
/// [column]: https://sqlite.org/c3ref/column_blob.html
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
    unsafe fn fetch<'c>(statement: &'r Statement<'c>, column: ColumnIndex) -> Self
    where
        'c: 'r;
}

impl<'r> Fetch<'r> for i32 {
    unsafe fn fetch<'c>(statement: &'r Statement<'c>, column: ColumnIndex) -> Self
    where
        'c: 'r,
    {
        unsafe { sqlite3_column_int(statement.as_ptr(), column.value()) as i32 }
    }
}

impl<'r> Fetch<'r> for i64 {
    unsafe fn fetch<'c>(statement: &'r Statement<'c>, column: ColumnIndex) -> Self
    where
        'c: 'r,
    {
        unsafe { sqlite3_column_int64(statement.as_ptr(), column.value()) as i64 }
    }
}

impl<'r> Fetch<'r> for f64 {
    unsafe fn fetch<'c>(statement: &'r Statement<'c>, column: ColumnIndex) -> Self
    where
        'c: 'r,
    {
        unsafe { sqlite3_column_double(statement.as_ptr(), column.value()) }
    }
}

impl<'r> Fetch<'r> for Type {
    unsafe fn fetch<'c>(statement: &'r Statement<'c>, column: ColumnIndex) -> Self
    where
        'c: 'r,
    {
        unsafe { Type::fetch(statement, column) }
    }
}

impl<'r, T> Fetch<'r> for Option<T>
where
    T: Fetch<'r>,
{
    unsafe fn fetch<'c>(statement: &'r Statement<'c>, column: ColumnIndex) -> Self
    where
        'c: 'r,
    {
        let column_type = unsafe { Type::fetch(statement, column) };

        if column_type.has_value() {
            Some(unsafe { T::fetch(statement, column) })
        } else {
            None
        }
    }
}
