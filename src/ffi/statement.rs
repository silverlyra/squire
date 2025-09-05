use core::{
    ffi::{c_char, c_int},
    marker::PhantomData,
    ops::Deref,
    ptr,
};

use sqlite::{
    SQLITE_DONE, SQLITE_MISUSE, SQLITE_OK, SQLITE_ROW, sqlite3, sqlite3_clear_bindings,
    sqlite3_data_count, sqlite3_db_handle, sqlite3_finalize, sqlite3_prepare_v3, sqlite3_reset,
    sqlite3_step, sqlite3_stmt,
};

use super::{
    bind::{Bind, Index},
    connection::Connection,
};
use crate::{
    call,
    error::{Error, Result},
};

/// A thin wrapper around a [`sqlite3_stmt`] prepared statement pointer.
#[derive(Debug)]
#[repr(transparent)]
pub struct Statement<'c> {
    handle: ptr::NonNull<sqlite3_stmt>,
    _connection: PhantomData<&'c Connection>,
}

#[cfg(any(feature = "multi-thread", feature = "serialized"))]
unsafe impl<'c> Send for Statement<'c> {}
#[cfg(feature = "serialized")]
unsafe impl<'c> Sync for Statement<'c> {}

impl<'c> Statement<'c> {
    #[inline]
    #[must_use]
    pub const fn new(handle: *mut sqlite3_stmt) -> Option<Self> {
        match ptr::NonNull::new(handle) {
            Some(handle) => Some(Self {
                handle,
                _connection: PhantomData,
            }),
            None => None,
        }
    }

    #[must_use]
    pub fn prepare(connection: &'c Connection, query: &str, flags: u32) -> Result<(Self, usize)> {
        let length = i32::try_from(query.len()).map_err(|_| Error::too_big())?;
        let query_p = query.as_bytes().as_ptr().cast::<c_char>();
        let mut handle: *mut sqlite3_stmt = ptr::null_mut();
        let mut tail: *const c_char = ptr::null();

        let result = unsafe {
            sqlite3_prepare_v3(
                connection.as_ptr(),
                query_p,
                length,
                flags,
                &mut handle,
                &mut tail,
            )
        };

        let sql_length = if tail.is_null() {
            0
        } else {
            unsafe { tail.byte_offset_from_unsigned(query_p) }
        };

        match Self::new(handle) {
            Some(statement) if result == SQLITE_OK => Ok((statement, sql_length)),
            _ => Err(Error::from(result)),
        }
    }

    pub fn binding(&mut self) -> Binding<'c, '_> {
        Binding { statement: self }
    }

    #[inline]
    pub fn close(self) -> Result<()> {
        call! { sqlite3_finalize(self.as_ptr()) }
    }

    #[inline]
    pub(super) fn as_ptr(&self) -> *mut sqlite3_stmt {
        self.handle.as_ptr()
    }

    #[inline]
    pub(super) unsafe fn connection_ptr(&self) -> *mut sqlite3 {
        unsafe { sqlite3_db_handle(self.as_ptr()) }
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct Binding<'c, 's: 'c> {
    statement: &'s mut Statement<'c>,
}

impl<'c, 's: 'c> Binding<'c, 's> {
    pub unsafe fn set<B>(&'s mut self, index: Index, value: B) -> Result<()>
    where
        B: Bind<'s>,
    {
        unsafe { value.bind(self.statement, index) }
    }

    pub fn ready(&mut self) -> Execution<'c, 's, '_> {
        Execution { binding: self }
    }

    #[doc(alias = "sqlite3_clear_bindings")]
    pub fn clear(self) -> Result<()> {
        call! { sqlite3_clear_bindings(self.statement.as_ptr()) }
    }
}

impl<'c, 's: 'c> Drop for Binding<'c, 's> {
    fn drop(&mut self) {
        unsafe {
            sqlite3_clear_bindings(self.statement.as_ptr());
        }
    }
}

pub unsafe trait Conclusion: Sized {
    #[cfg(feature = "nightly")]
    type Output: From<Self> = Self;
    #[cfg(not(feature = "nightly"))]
    type Output: From<Self>;

    fn from_connection_ptr(connection: *mut sqlite3) -> Self::Output;
}

unsafe impl Conclusion for () {
    #[cfg(not(feature = "nightly"))]
    type Output = Self;

    #[inline(always)]
    fn from_connection_ptr(_connection: *mut sqlite3) -> Self::Output {
        ()
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct Execution<'c, 's: 'c, 'b: 's> {
    binding: &'b mut Binding<'c, 's>,
}

impl<'c, 's: 'c, 'b: 's> Execution<'c, 's, 'b> {
    #[doc(alias = "sqlite3_step")]
    pub fn row(&mut self) -> Result<Option<Row<'c, 's, 'b, '_>>> {
        let result = unsafe { sqlite3_step(self.statement.as_ptr()) };

        if result == SQLITE_ROW {
            Ok(Some(Row { execution: self }))
        } else if result == SQLITE_DONE {
            Ok(None)
        } else {
            match Error::new(result) {
                Some(err) => Err(err),
                None => Ok(None),
            }
        }
    }

    pub fn execute<C: Conclusion>(self) -> Result<C::Output> {
        let result = unsafe { sqlite3_step(self.statement.as_ptr()) };

        if result == SQLITE_DONE {
            Ok(C::from_connection_ptr(unsafe {
                self.statement.connection_ptr()
            }))
        } else if result == SQLITE_ROW {
            Err(Error::misuse())
        } else {
            Err(Error::from(result))
        }
    }

    #[doc(alias = "sqlite3_reset")]
    pub fn reset(self) -> Result<()> {
        call! { sqlite3_reset(self.statement.as_ptr()) }
    }
}

impl<'c, 's: 'c, 'b: 's> Deref for Execution<'c, 's, 'b> {
    type Target = Binding<'c, 's>;

    fn deref(&self) -> &Self::Target {
        self.binding
    }
}

impl<'c, 's: 'c, 'b: 's> Drop for Execution<'c, 's, 'b> {
    fn drop(&mut self) {
        unsafe {
            sqlite3_reset(self.statement.as_ptr());
        }
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct Row<'c, 's: 'c, 'b: 's, 'e: 'b> {
    execution: &'e mut Execution<'c, 's, 'b>,
}

impl<'c, 's: 'c, 'b: 's, 'e: 'b> Row<'c, 's, 'b, 'e> {
    #[doc(alias = "sqlite3_data_count")]
    pub fn column_count(&mut self) -> c_int {
        unsafe { sqlite3_data_count(self.statement.as_ptr()) }
    }
}

impl<'c, 's: 'c, 'b: 's, 'e: 'b> Deref for Row<'c, 's, 'b, 'e> {
    type Target = Execution<'c, 's, 'b>;

    fn deref(&self) -> &Self::Target {
        self.execution
    }
}
