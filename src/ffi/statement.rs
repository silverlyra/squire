use core::{
    ffi::{c_char, c_int},
    marker::PhantomData,
    ptr,
};
use std::ffi::CStr;

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
    connection::Connection,
    value::{Column, Fetch},
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

    #[doc(alias = "sqlite3_column_count")]
    pub fn column_count(&self) -> c_int {
        unsafe { sqlite3_column_count(self.as_ptr()) }
    }

    #[inline]
    #[doc(alias = "sqlite3_finalize")]
    pub fn close(self) -> Result<()> {
        call! { sqlite3_finalize(self.as_ptr()) }
    }

    /// Access the raw [`sqlite3_stmt`] pointer.
    #[inline]
    pub const fn as_ptr(&self) -> *mut sqlite3_stmt {
        self.handle.as_ptr()
    }

    #[inline]
    pub(super) unsafe fn connection_ptr(&self) -> *mut sqlite3 {
        unsafe { sqlite3_db_handle(self.as_ptr()) }
    }

    /// Return the highest (1-based) parameter index used by this [`Statement`].
    #[doc(alias = "sqlite3_bind_parameter_count")]
    pub fn parameter_count(&self) -> isize {
        unsafe { sqlite3_bind_parameter_count(self.as_ptr()) as isize }
    }

    #[doc(alias = "sqlite3_bind_parameter_name")]
    pub fn parameter_name(&self, index: Index) -> Option<&CStr> {
        let ptr = unsafe { sqlite3_bind_parameter_name(self.as_ptr(), index.value()) };

        if ptr.is_null() {
            None
        } else {
            Some(unsafe { CStr::from_ptr(ptr) })
        }
    }
}

impl<'c> Drop for Statement<'c> {
    fn drop(&mut self) {
        unsafe {
            sqlite3_finalize(self.as_ptr());
        }
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct Binding<'c, 's>
where
    'c: 's,
{
    statement: &'s mut Statement<'c>,
}

impl<'c, 's> Binding<'c, 's>
where
    'c: 's,
{
    pub unsafe fn set<'b, B>(&'b mut self, index: Index, value: B) -> Result<()>
    where
        B: Bind<'b>,
    {
        unsafe { value.bind(self.statement, index) }
    }

    pub fn ready<'b>(&'b mut self) -> Execution<'c, &'b mut Self>
    where
        's: 'b,
    {
        Execution::new(self)
    }

    pub fn done(self) -> Execution<'c, Self> {
        Execution::new(self)
    }

    #[doc(alias = "sqlite3_clear_bindings")]
    pub fn clear(self) -> Result<()> {
        call! { sqlite3_clear_bindings(self.statement.as_ptr()) }
    }
}

impl<'c, 's> Drop for Binding<'c, 's>
where
    'c: 's,
{
    fn drop(&mut self) {
        unsafe {
            sqlite3_clear_bindings(self.statement.as_ptr());
        }
    }
}

pub unsafe trait Conclusion: Sized {
    fn from_connection_ptr(connection: *mut sqlite3) -> Self;
}

unsafe impl Conclusion for () {
    #[inline(always)]
    fn from_connection_ptr(_connection: *mut sqlite3) -> Self {
        ()
    }
}

unsafe impl Conclusion for isize {
    #[inline(always)]
    fn from_connection_ptr(connection: *mut sqlite3) -> Self {
        #[cfg(target_pointer_width = "32")]
        let changes = unsafe { sqlite3_changes(connection) };

        #[cfg(target_pointer_width = "64")]
        let changes = unsafe { sqlite3_changes64(connection) };

        changes as Self
    }
}

pub unsafe trait Execute<'c> {
    type Cursor<'e>: Execute<'c> + Sized
    where
        'c: 'e,
        Self: 'e;

    unsafe fn connection_ptr(&self) -> *mut sqlite3;

    unsafe fn statement_ptr(&self) -> *mut sqlite3_stmt;

    fn cursor<'e>(&'e mut self) -> Self::Cursor<'e>
    where
        'c: 'e,
        Self: 'e;

    unsafe fn reset(&mut self) -> Result<()>;
}

unsafe impl<'c> Execute<'c> for Statement<'c> {
    type Cursor<'e>
        = &'e mut Statement<'c>
    where
        'c: 'e,
        Self: 'e;

    #[inline(always)]
    unsafe fn connection_ptr(&self) -> *mut sqlite3 {
        unsafe { Statement::connection_ptr(self) }
    }

    #[inline(always)]
    unsafe fn statement_ptr(&self) -> *mut sqlite3_stmt {
        self.as_ptr()
    }

    #[inline(always)]
    fn cursor<'e>(&'e mut self) -> Self::Cursor<'e>
    where
        'c: 'e,
        Self: 'e,
    {
        self
    }

    #[inline(always)]
    unsafe fn reset(&mut self) -> Result<()> {
        Ok(())
    }
}

unsafe impl<'c, 's> Execute<'c> for &'s mut Statement<'c>
where
    'c: 's,
{
    type Cursor<'e>
        = &'e mut Statement<'c>
    where
        'c: 'e,
        Self: 'e;

    #[inline(always)]
    unsafe fn connection_ptr(&self) -> *mut sqlite3 {
        unsafe { Statement::connection_ptr(self) }
    }

    #[inline(always)]
    unsafe fn statement_ptr(&self) -> *mut sqlite3_stmt {
        self.as_ptr()
    }

    #[inline(always)]
    fn cursor<'e>(&'e mut self) -> Self::Cursor<'e>
    where
        's: 'e,
        Self: 'e,
    {
        self
    }

    #[inline]
    unsafe fn reset(&mut self) -> Result<()> {
        call! { sqlite3_reset(self.statement_ptr()) }
    }
}

unsafe impl<'c, 's> Execute<'c> for Binding<'c, 's>
where
    'c: 's,
{
    type Cursor<'e>
        = &'e mut Statement<'c>
    where
        'c: 'e,
        Self: 'e;

    #[inline(always)]
    unsafe fn connection_ptr(&self) -> *mut sqlite3 {
        unsafe { self.statement.connection_ptr() }
    }

    #[inline(always)]
    unsafe fn statement_ptr(&self) -> *mut sqlite3_stmt {
        self.statement.as_ptr()
    }

    #[inline(always)]
    fn cursor<'e>(&'e mut self) -> Self::Cursor<'e>
    where
        's: 'e,
        Self: 'e,
    {
        self.statement
    }

    #[inline]
    unsafe fn reset(&mut self) -> Result<()> {
        let statement = unsafe { self.statement_ptr() };
        call! { sqlite3_reset(statement) }?;
        call! { sqlite3_clear_bindings(statement) }
    }
}

unsafe impl<'c, 's, 'b> Execute<'c> for &'b mut Binding<'c, 's>
where
    'c: 's,
    's: 'b,
{
    type Cursor<'e>
        = &'e mut Statement<'c>
    where
        's: 'e,
        Self: 'e;

    #[inline(always)]
    unsafe fn connection_ptr(&self) -> *mut sqlite3 {
        unsafe { self.statement.connection_ptr() }
    }

    #[inline(always)]
    unsafe fn statement_ptr(&self) -> *mut sqlite3_stmt {
        self.statement.as_ptr()
    }

    #[inline(always)]
    fn cursor<'e>(&'e mut self) -> Self::Cursor<'e>
    where
        's: 'e,
        Self: 'e,
    {
        self.statement
    }

    #[inline]
    unsafe fn reset(&mut self) -> Result<()> {
        call! { sqlite3_reset(self.statement_ptr()) }
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct Execution<'c, S>
where
    S: Execute<'c>,
{
    inner: S,
    _connection: PhantomData<&'c Connection>,
}

impl<'c, S> Execution<'c, S>
where
    S: Execute<'c>,
{
    #[inline]
    const fn new(executor: S) -> Self {
        Self {
            inner: executor,
            _connection: PhantomData,
        }
    }

    /// [Step][step] the [statement](Statement) and read the next row.
    ///
    /// Returns:
    /// - a [`Row`] if [`sqlite3_step`][step] returns `SQLITE_ROW`
    /// - `None` if [`sqlite3_step`][step] returns `SQLITE_DONE`
    /// - an [`Error`] if [`sqlite3_step`][step] returns an error result code
    ///
    /// [step]: https://sqlite.org/c3ref/step.html
    #[doc(alias = "sqlite3_step")]
    pub fn row(&mut self) -> Result<Option<Row<'c, '_, S>>> {
        let result = unsafe { sqlite3_step(self.statement_ptr()) };

        if result == SQLITE_ROW {
            Ok(Some(Row {
                execution: self.inner.cursor(),
            }))
        } else if result == SQLITE_DONE {
            Ok(None)
        } else {
            match Error::new(result) {
                Some(err) => Err(err),
                None => Ok(None),
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
    pub fn execute<C: Conclusion>(mut self) -> Result<C> {
        let result = unsafe { sqlite3_step(self.statement_ptr()) };

        if result == SQLITE_DONE {
            let connection_ptr = unsafe { self.inner.connection_ptr() };
            Ok(C::from_connection_ptr(connection_ptr))
        } else if result == SQLITE_ROW {
            Err(Error::misuse())
        } else {
            Err(Error::from(result))
        }
    }

    /// [Reset][reset] the [statement](Statement).
    ///
    /// [reset]: https://sqlite.org/c3ref/reset.html
    #[doc(alias = "sqlite3_reset")]
    pub fn reset(mut self) -> Result<()> {
        unsafe { self.inner.reset() }
    }

    #[inline]
    unsafe fn statement_ptr(&mut self) -> *mut sqlite3_stmt {
        unsafe { self.inner.statement_ptr() }
    }
}

impl<'c, S> Drop for Execution<'c, S>
where
    S: Execute<'c>,
{
    fn drop(&mut self) {
        unsafe {
            let _ = self.inner.reset();
        }
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct Row<'c, 'e, E>
where
    E: Execute<'c> + 'e,
    'c: 'e,
{
    execution: E::Cursor<'e>,
}

impl<'c, 'e, E> Row<'c, 'e, E>
where
    E: Execute<'c> + 'e,
    'c: 'e,
{
    pub unsafe fn fetch<'r, T: Fetch<'r>>(&'r self, column: Column) -> T {
        unsafe { T::fetch(self, column) }
    }

    #[doc(alias = "sqlite3_data_count")]
    pub fn column_count(&mut self) -> c_int {
        unsafe { sqlite3_data_count(self.statement_ptr()) }
    }

    #[inline]
    pub(super) fn statement_ptr(&self) -> *mut sqlite3_stmt {
        unsafe { self.execution.statement_ptr() }
    }
}
