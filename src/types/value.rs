use sqlite::{
    SQLITE_BLOB, SQLITE_FLOAT, SQLITE_INTEGER, SQLITE_NULL, SQLITE_TEXT, sqlite3_column_type,
};

use super::ColumnIndex;
use crate::ffi::Statement;

/// The datatype of a SQLite column value.
///
/// SQLite uses a [dynamic type system](https://www.sqlite.org/datatype3.html).
/// Each value stored in a SQLite database has one of these five types.
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

impl Type {
    /// `true` unless this [`Type`] is [`NULL`](Self::Null); `false` for `NULL`.
    pub const fn has_value(&self) -> bool {
        !self.is_null()
    }

    /// `true` if this [`Type`] is [`NULL`](Self::Null); `false` otherwise.
    pub const fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    /// Fetches the column type from a SQLite statement.
    ///
    /// # Safety
    ///
    /// The column index must be valid for the statement.
    pub(crate) unsafe fn fetch<'r, 'c>(statement: &'r Statement<'c>, column: ColumnIndex) -> Self
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
