use core::{
    ffi::{CStr, c_char, c_int},
    marker::PhantomData,
    ptr,
};

#[cfg(target_pointer_width = "32")]
use sqlite::sqlite3_changes;
#[cfg(target_pointer_width = "64")]
use sqlite::sqlite3_changes64;
use sqlite::{
    SQLITE_DONE, SQLITE_OK, SQLITE_ROW, sqlite3, sqlite3_bind_parameter_count,
    sqlite3_bind_parameter_name, sqlite3_clear_bindings, sqlite3_column_count, sqlite3_data_count,
    sqlite3_db_handle, sqlite3_finalize, sqlite3_prepare_v3, sqlite3_reset, sqlite3_step,
    sqlite3_stmt,
};

use super::{
    bind::{Bind, Index},
    connection::{Connected, Connection},
    value::{Column, Fetch},
};
use crate::{
    call,
    error::{Error, ErrorLocation, ErrorMessage, Result},
};

/// A thin wrapper around a [`sqlite3_stmt`] prepared statement pointer.
#[derive(Debug)]
#[repr(transparent)]
pub struct Statement<'c> {
    handle: ptr::NonNull<sqlite3_stmt>,
    _connection: PhantomData<&'c Connection>,
}

#[cfg(any(feature = "multi-thread", feature = "serialized"))]
#[cfg_attr(
    docsrs,
    doc(cfg(any(feature = "multi-thread", feature = "serialized")))
)]
unsafe impl<'c> Send for Statement<'c> {}

#[cfg(feature = "serialized")]
#[cfg_attr(docsrs, doc(cfg(feature = "serialized")))]
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

    /// Prepare a [`Statement`] on a [`Connection`] from SQL `query` text.
    #[doc(alias = "sqlite3_prepare_v3")]
    #[must_use]
    pub fn prepare(
        connection: &'c Connection,
        query: &str,
        flags: u32,
    ) -> Result<(Self, usize), (ErrorMessage, Option<ErrorLocation>)> {
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
            _ => {
                let error = Error::from_connection(connection, result);
                Err(error.unwrap_or_default())
            }
        }
    }

    #[inline]
    #[doc(alias = "sqlite3_finalize")]
    pub unsafe fn finalize(&mut self) -> Result<(), ()> {
        call! { sqlite3_finalize(self.as_ptr()) }
    }

    pub fn close(mut self) -> Result<(), ()> {
        unsafe { self.finalize() }
    }

    #[doc(alias = "sqlite3_column_count")]
    pub fn column_count(&self) -> c_int {
        unsafe { sqlite3_column_count(self.as_ptr()) }
    }

    /// Return the highest (1-based) parameter index used by this [`Statement`].
    #[doc(alias = "sqlite3_bind_parameter_count")]
    pub fn parameter_count(&self) -> isize {
        unsafe { sqlite3_bind_parameter_count(self.as_ptr()) as isize }
    }

    #[doc(alias = "sqlite3_bind_parameter_name")]
    pub unsafe fn parameter_name(&self, index: Index) -> Option<&CStr> {
        let ptr = unsafe { sqlite3_bind_parameter_name(self.as_ptr(), index.value()) };

        if ptr.is_null() {
            None
        } else {
            Some(unsafe { CStr::from_ptr(ptr) })
        }
    }

    pub fn bind<'b, B>(&'b mut self, index: Index, value: B) -> Result<()>
    where
        B: Bind<'b>,
    {
        unsafe { value.bind(self, index) }
    }

    #[doc(alias = "sqlite3_clear_bindings")]
    pub fn clear(&mut self) -> Result<(), ()> {
        call! { sqlite3_clear_bindings(self.as_ptr()) }
    }

    /// [Step][step] the [statement](Statement) and read the next row.
    ///
    /// Returns:
    /// - `Ok(true)` if [`sqlite3_step`][step] returns `SQLITE_ROW`
    /// - `Ok(false)` if [`sqlite3_step`][step] returns `SQLITE_DONE`
    /// - an [`Error`] if [`sqlite3_step`][step] returns an error result code
    ///
    /// [step]: https://sqlite.org/c3ref/step.html
    #[doc(alias = "sqlite3_step")]
    pub fn row(&mut self) -> Result<bool> {
        let result = unsafe { sqlite3_step(self.as_ptr()) };

        if result == SQLITE_ROW {
            Ok(true)
        } else if result == SQLITE_DONE {
            Ok(false)
        } else {
            match Error::from_connection(self, result) {
                Some(err) => Err(err),
                None => Ok(false),
            }
        }
    }

    /// [Execute][step] the [statement](Statement), returning `()`, the
    /// last-inserted [`RowId`](crate::RowId), or the
    /// [number of changes](primitive@isize).
    ///
    /// Returns:
    /// - the [`Conclusion`] if [`sqlite3_step`][step] returns `SQLITE_DONE`
    /// - a [misuse error](crate::ErrorCategory::Misuse) if [`sqlite3_step`][step] returns `SQLITE_ROW`
    /// - an [`Error`] if [`sqlite3_step`][step] returns an error result code
    ///
    /// [step]: https://sqlite.org/c3ref/step.html
    pub fn execute<C: Conclusion>(&mut self) -> Result<C> {
        let result = unsafe { sqlite3_step(self.as_ptr()) };

        if result == SQLITE_DONE {
            let connection_ptr = unsafe { self.connection_ptr() };
            Ok(unsafe { C::from_connection_ptr(connection_ptr) })
        } else if result == SQLITE_ROW {
            Err(Error::misuse().into())
        } else {
            Err(Error::from_connection(self, result).unwrap_or_default())
        }
    }

    /// [Reset][reset] the [statement](Statement).
    ///
    /// [reset]: https://sqlite.org/c3ref/reset.html
    #[doc(alias = "sqlite3_reset")]
    pub unsafe fn reset(&mut self) -> Result<(), ()> {
        call! { sqlite3_reset(self.as_ptr()) }
    }

    pub unsafe fn fetch<'r, T: Fetch<'r>>(&'r self, column: Column) -> T {
        unsafe { T::fetch(self, column) }
    }

    #[doc(alias = "sqlite3_data_count")]
    pub fn data_count(&mut self) -> c_int {
        unsafe { sqlite3_data_count(self.as_ptr()) }
    }

    /// Access the raw [`sqlite3_stmt`] pointer.
    #[inline]
    pub const fn as_ptr(&self) -> *mut sqlite3_stmt {
        self.handle.as_ptr()
    }

    #[inline]
    pub(crate) unsafe fn connection_ptr(&self) -> *mut sqlite3 {
        unsafe { sqlite3_db_handle(self.as_ptr()) }
    }
}

impl<'c> Connected for Statement<'c> {
    fn as_connection_ptr(&self) -> *mut sqlite3 {
        unsafe { self.connection_ptr() }
    }
}

impl<'c> Connected for &Statement<'c> {
    fn as_connection_ptr(&self) -> *mut sqlite3 {
        unsafe { self.connection_ptr() }
    }
}

impl<'c> Connected for &mut Statement<'c> {
    fn as_connection_ptr(&self) -> *mut sqlite3 {
        unsafe { self.connection_ptr() }
    }
}

pub trait Execute<'c>: Connected {
    unsafe fn as_statement_ptr(&self) -> *mut sqlite3_stmt;

    unsafe fn cursor<'e>(&'e mut self) -> &'e mut Statement<'c>
    where
        'c: 'e,
        Self: 'e;

    unsafe fn reset(&mut self) -> Result<(), ()>;
}

impl<'c> Execute<'c> for Statement<'c> {
    unsafe fn as_statement_ptr(&self) -> *mut sqlite3_stmt {
        self.as_ptr()
    }

    unsafe fn cursor<'e>(&'e mut self) -> &'e mut Statement<'c>
    where
        'c: 'e,
        Self: 'e,
    {
        self
    }

    #[inline(always)]
    unsafe fn reset(&mut self) -> Result<(), ()> {
        Ok(())
    }
}

impl<'c, 's> Execute<'c> for &'s mut Statement<'c>
where
    'c: 's,
{
    unsafe fn as_statement_ptr(&self) -> *mut sqlite3_stmt {
        self.as_ptr()
    }

    unsafe fn cursor<'e>(&'e mut self) -> &'e mut Statement<'c>
    where
        'c: 'e,
        Self: 'e,
    {
        self
    }

    #[inline]
    unsafe fn reset(&mut self) -> Result<(), ()> {
        call! { sqlite3_reset(self.as_statement_ptr()) }
    }
}

pub trait Conclusion: Sized {
    unsafe fn from_connection_ptr(connection: *mut sqlite3) -> Self;
}

impl Conclusion for () {
    #[inline(always)]
    unsafe fn from_connection_ptr(_connection: *mut sqlite3) -> Self {
        ()
    }
}

impl Conclusion for isize {
    #[inline(always)]
    unsafe fn from_connection_ptr(connection: *mut sqlite3) -> Self {
        #[cfg(target_pointer_width = "32")]
        let changes = unsafe { sqlite3_changes(connection) };

        #[cfg(target_pointer_width = "64")]
        let changes = unsafe { sqlite3_changes64(connection) };

        changes as Self
    }
}
