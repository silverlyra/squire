use core::num::NonZero;

use sqlite::{sqlite3, sqlite3_last_insert_rowid};

use crate::ffi::Conclusion;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
#[repr(transparent)]
pub struct RowId(NonZero<i64>);

impl RowId {
    pub const fn new(value: i64) -> Option<Self> {
        match NonZero::new(value) {
            Some(id) => Some(RowId(id)),
            None => None,
        }
    }

    pub const fn into_inner(self) -> i64 {
        self.0.get()
    }
}

unsafe impl Conclusion for Option<RowId> {
    #[inline]
    fn from_connection_ptr(connection: *mut sqlite3) -> Self {
        RowId::new(unsafe { sqlite3_last_insert_rowid(connection) })
    }
}

mod mutex {}
