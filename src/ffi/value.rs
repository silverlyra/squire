use sqlite::{sqlite3_column_double, sqlite3_column_int, sqlite3_column_int64};

use super::statement::Statement;
use crate::types::{ColumnIndex, Type};

pub trait Fetch<'r> {
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
