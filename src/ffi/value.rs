use core::{ffi::c_int, ops::Deref, slice};

use sqlite::{
    SQLITE_BLOB, SQLITE_FLOAT, SQLITE_INTEGER, SQLITE_NULL, SQLITE_TEXT, sqlite3_column_blob,
    sqlite3_column_bytes, sqlite3_column_double, sqlite3_column_int, sqlite3_column_int64,
    sqlite3_column_text, sqlite3_column_type,
};

use super::statement::Statement;

pub trait Fetch<'r> {
    unsafe fn fetch<'c>(statement: &'r Statement<'c>, column: Column) -> Self
    where
        'c: 'r;
}

impl<'r> Fetch<'r> for i32 {
    unsafe fn fetch<'c>(statement: &'r Statement<'c>, column: Column) -> Self
    where
        'c: 'r,
    {
        unsafe { sqlite3_column_int(statement.as_ptr(), column.value()) as i32 }
    }
}

impl<'r> Fetch<'r> for i64 {
    unsafe fn fetch<'c>(statement: &'r Statement<'c>, column: Column) -> Self
    where
        'c: 'r,
    {
        unsafe { sqlite3_column_int64(statement.as_ptr(), column.value()) as i64 }
    }
}

impl<'r> Fetch<'r> for f64 {
    unsafe fn fetch<'c>(statement: &'r Statement<'c>, column: Column) -> Self
    where
        'c: 'r,
    {
        unsafe { sqlite3_column_double(statement.as_ptr(), column.value()) }
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct Bytes<'r, T: ?Sized = [u8]>(&'r T);

impl<'r, T: ?Sized> Bytes<'r, T> {
    pub const fn into_inner(self) -> &'r T {
        self.0
    }
}

impl<'r, T: ?Sized> Deref for Bytes<'r, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'r> Fetch<'r> for Bytes<'r, str> {
    unsafe fn fetch<'c>(statement: &'r Statement<'c>, column: Column) -> Self
    where
        'c: 'r,
    {
        let data = unsafe { sqlite3_column_text(statement.as_ptr(), column.value()) };
        let len = unsafe { sqlite3_column_bytes(statement.as_ptr(), column.value()) };

        let bytes = unsafe { slice::from_raw_parts::<'r, u8>(data, len as usize) };
        let text = unsafe { str::from_utf8_unchecked(bytes) };

        Self(text)
    }
}

impl<'r> Fetch<'r> for Bytes<'r, [u8]> {
    unsafe fn fetch<'c>(statement: &'r Statement<'c>, column: Column) -> Self
    where
        'c: 'r,
    {
        let data = unsafe { sqlite3_column_blob(statement.as_ptr(), column.value()) };
        let len = unsafe { sqlite3_column_bytes(statement.as_ptr(), column.value()) };

        let bytes = unsafe { slice::from_raw_parts::<'r, u8>(data as *const u8, len as usize) };

        Self(bytes)
    }
}

impl<'r> Fetch<'r> for Type {
    unsafe fn fetch<'c>(statement: &'r Statement<'c>, column: Column) -> Self
    where
        'c: 'r,
    {
        let value = unsafe { sqlite3_column_type(statement.as_ptr(), column.value()) };

        match value {
            SQLITE_INTEGER => Type::Integer,
            SQLITE_FLOAT => Type::Float,
            SQLITE_TEXT => Type::Text,
            SQLITE_BLOB => Type::Blob,
            SQLITE_NULL => Type::Null,
            _ => panic!("unknown sqlite3_column_type {value}"),
        }
    }
}

/// A SQLite column index, used for [reading values][] out of queried rows.
///
/// [reading values]: https://sqlite.org/c3ref/column_blob.html
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
#[repr(transparent)]
pub struct Column(c_int);

impl Column {
    pub const fn new(value: c_int) -> Self {
        Self(value)
    }

    /// Access the underlying SQLite datatype constant as a C [`int`](c_int).
    #[inline]
    pub const fn value(&self) -> c_int {
        self.0
    }
}

impl From<i32> for Column {
    fn from(value: i32) -> Self {
        Self::new(value as c_int)
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[repr(i32)]
pub enum Type {
    #[doc(alias = "SQLITE_INTEGER")]
    Integer = SQLITE_INTEGER,
    #[doc(alias = "SQLITE_FLOAT")]
    Float = SQLITE_FLOAT,
    #[doc(alias = "SQLITE_TEXT")]
    Text = SQLITE_TEXT,
    #[doc(alias = "SQLITE_BLOB")]
    Blob = SQLITE_BLOB,
    #[doc(alias = "SQLITE_NULL")]
    Null = SQLITE_NULL,
}
