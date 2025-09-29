use core::{ffi::c_int, marker::PhantomData};
use sqlite::{SQLITE_PREPARE_DONT_LOG, SQLITE_PREPARE_NO_VTAB, SQLITE_PREPARE_PERSISTENT, sqlite3};

use crate::{
    bind::{Bind, Index},
    connection::Connection,
    error::{Error, ErrorLocation, ErrorMessage, Result},
    ffi,
    param::Parameters,
    row::Row,
    types::RowId,
    value::Column,
};

/// A [prepared statement][]
///
/// [prepared statement]: https://sqlite.org/c3ref/stmt.html
#[derive(Debug)]
#[repr(transparent)]
pub struct Statement<'c> {
    inner: ffi::Statement<'c>,
}

impl<'c> Statement<'c> {
    #[inline]
    #[must_use]
    pub(crate) const fn new(inner: ffi::Statement<'c>) -> Self {
        Self { inner }
    }

    #[must_use]
    pub fn prepare(
        connection: &'c Connection,
        query: impl AsRef<str>,
        options: PrepareOptions,
    ) -> Result<Self, (ErrorMessage, Option<ErrorLocation>)> {
        ffi::Statement::prepare(
            connection.internal_ref(),
            query.as_ref(),
            options.into_inner(),
        )
        .map(|(statement, _)| Self::new(statement))
    }

    pub fn binding(&mut self) -> Binding<'c, '_> {
        Binding { statement: self }
    }

    pub fn bind<'s, P>(&'s mut self, parameters: P) -> Result<Binding<'c, 's>>
    where
        P: Parameters<'s>,
    {
        let indexes =
            P::resolve(self).ok_or(Error::resolve("cannot resolve bind parameter indexes"))?;

        let mut binding = self.binding();
        parameters.bind(&mut binding, indexes)?;
        Ok(binding)
    }

    pub fn query<'s, P>(&'s mut self, parameters: P) -> Result<Execution<'c, Binding<'c, 's>>>
    where
        P: Parameters<'s>,
    {
        self.bind(parameters).map(Binding::done)
    }

    // Inspect the [columns](StatementColumns) returned by this statement.
    pub fn columns<'s>(&'s self) -> StatementColumns<'c, 's> {
        StatementColumns::new(self)
    }

    // Inspect the [parameters](StatementParameters) declared by this statement.
    pub fn parameters<'s>(&'s self) -> StatementParameters<'c, 's> {
        StatementParameters::new(self)
    }

    /// Access the [`ffi::Statement`] underlying this [`Statement`].
    #[inline]
    pub(crate) fn internal_ref(&self) -> &ffi::Statement<'c> {
        &self.inner
    }

    /// Mutate the [`ffi::Statement`] underlying this [`Statement`].
    #[inline]
    pub(crate) fn internal_mut(&mut self) -> &mut ffi::Statement<'c> {
        &mut self.inner
    }
}

impl<'c> ffi::Connected for Statement<'c> {
    fn as_connection_ptr(&self) -> *mut sqlite3 {
        unsafe { self.internal_ref().connection_ptr() }
    }
}

impl<'c> Execute<'c> for Statement<'c> {
    fn cursor<'e>(&'e mut self) -> &'e mut Statement<'c>
    where
        'c: 'e,
        Self: 'e,
    {
        self
    }

    #[inline(always)]
    fn reset(&mut self) -> Result<(), ()> {
        Ok(())
    }
}

impl<'c, 's> ffi::Connected for &'s mut Statement<'c> {
    fn as_connection_ptr(&self) -> *mut sqlite3 {
        unsafe { self.internal_ref().connection_ptr() }
    }
}

impl<'c, 's> Execute<'c> for &'s mut Statement<'c> {
    fn cursor<'e>(&'e mut self) -> &'e mut Statement<'c>
    where
        'c: 'e,
        Self: 'e,
    {
        self
    }

    #[inline(always)]
    fn reset(&mut self) -> Result<(), ()> {
        unsafe { self.internal_mut().reset() }
    }
}

/// Controls the behavior of [preparing](Statement::prepare()) a [`Statement`].
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct PrepareOptions(u32);

impl PrepareOptions {
    const DONT_LOG: u32 = SQLITE_PREPARE_DONT_LOG as u32;
    const NO_VTAB: u32 = SQLITE_PREPARE_NO_VTAB as u32;
    const PERSISTENT: u32 = SQLITE_PREPARE_PERSISTENT as u32;

    pub const fn transient() -> Self {
        Self(0)
    }

    pub const fn persistent() -> Self {
        Self(Self::PERSISTENT)
    }

    pub const fn allow_virtual_tables(&self, allowed: bool) -> Self {
        if allowed {
            Self(self.0 & !Self::NO_VTAB)
        } else {
            Self(self.0 | Self::NO_VTAB)
        }
    }

    pub const fn log(&self, allowed: bool) -> Self {
        if allowed {
            Self(self.0 & !Self::DONT_LOG)
        } else {
            Self(self.0 | Self::DONT_LOG)
        }
    }

    pub const fn into_inner(self) -> u32 {
        self.0
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
    pub fn set<B>(&mut self, index: Index, value: B) -> Result<()>
    where
        B: Bind<'s>,
    {
        self.statement
            .internal_mut()
            .bind(index, value.into_bind_value()?)
    }

    pub fn ready<'b>(&'b mut self) -> Execution<'c, &'b mut Self> {
        Execution::new(self)
    }

    pub fn done(self) -> Execution<'c, Self> {
        Execution::new(self)
    }
}

impl<'c, 's> ffi::Connected for Binding<'c, 's>
where
    'c: 's,
{
    fn as_connection_ptr(&self) -> *mut sqlite3 {
        self.statement.as_connection_ptr()
    }
}

impl<'c, 's> Execute<'c> for Binding<'c, 's>
where
    'c: 's,
{
    fn cursor<'e>(&'e mut self) -> &'e mut Statement<'c>
    where
        'c: 'e,
        Self: 'e,
    {
        &mut self.statement
    }

    fn reset(&mut self) -> Result<(), ()> {
        let inner = self.statement.internal_mut();

        inner.clear()?;
        unsafe { inner.reset() }
    }
}

impl<'c, 's, 'b> ffi::Connected for &'b mut Binding<'c, 's> {
    fn as_connection_ptr(&self) -> *mut sqlite3 {
        self.statement.as_connection_ptr()
    }
}

impl<'c, 's, 'b> Execute<'c> for &'b mut Binding<'c, 's>
where
    'c: 's,
    's: 'b,
{
    fn cursor<'e>(&'e mut self) -> &'e mut Statement<'c>
    where
        'c: 'e,
        Self: 'e,
    {
        &mut self.statement
    }

    fn reset(&mut self) -> Result<(), ()> {
        let inner = self.statement.internal_mut();
        unsafe { inner.reset() }
    }
}

pub trait Execute<'c>: ffi::Connected {
    fn cursor<'e>(&'e mut self) -> &'e mut Statement<'c>
    where
        'c: 'e,
        Self: 'e;

    fn reset(&mut self) -> Result<(), ()>;
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
    const fn new(inner: S) -> Self {
        Self {
            inner,
            _connection: PhantomData,
        }
    }

    pub fn row(&mut self) -> Result<Option<Row<'c, '_, S>>> {
        let more = self.cursor().internal_mut().row()?;
        Ok(if more { Some(Row::new(self)) } else { None })
    }

    pub fn run(mut self) -> Result<isize> {
        self.cursor().internal_mut().execute()
    }

    pub fn insert(mut self) -> Result<Option<RowId>> {
        self.cursor().internal_mut().execute()
    }
}

impl<'c, S> ffi::Connected for Execution<'c, S>
where
    S: Execute<'c>,
{
    #[inline]
    fn as_connection_ptr(&self) -> *mut sqlite3 {
        self.inner.as_connection_ptr()
    }
}

impl<'c, S> Execute<'c> for Execution<'c, S>
where
    S: Execute<'c>,
{
    #[inline]
    fn cursor<'e>(&'e mut self) -> &'e mut Statement<'c>
    where
        'c: 'e,
        Self: 'e,
    {
        self.inner.cursor()
    }

    #[inline]
    fn reset(&mut self) -> Result<(), ()> {
        self.inner.reset()
    }
}

impl<'c, S> Drop for Execution<'c, S>
where
    S: Execute<'c>,
{
    fn drop(&mut self) {
        let _ = self.inner.reset();
    }
}

#[derive(Debug)]
pub struct StatementColumns<'c, 's>
where
    'c: 's,
{
    statement: &'s Statement<'c>,
}

impl<'c, 's> StatementColumns<'c, 's>
where
    'c: 's,
{
    const fn new(statement: &'s Statement<'c>) -> Self {
        Self { statement }
    }

    pub fn name(&self, column: Column) -> Option<&str> {
        self.statement
            .internal_ref()
            .column_name(column)
            .map(|name| unsafe { str::from_utf8_unchecked(name.to_bytes()) })
    }

    pub fn index(&self, name: impl AsRef<str>) -> Option<Column> {
        let name = name.as_ref();

        for index in self.iter() {
            if let Some(n) = self.name(index)
                && name == n
            {
                return Some(index);
            }
        }

        None
    }

    pub fn iter(&self) -> impl Iterator<Item = Column> {
        StatementColumnIter::new(self.count())
    }

    pub fn len(&self) -> usize {
        self.count() as usize
    }

    fn count(&self) -> c_int {
        self.statement.internal_ref().column_count()
    }
}

impl<'c, 's> IntoIterator for StatementColumns<'c, 's>
where
    'c: 's,
{
    type Item = Column;
    type IntoIter = StatementColumnIter;

    fn into_iter(self) -> Self::IntoIter {
        StatementColumnIter::new(self.count())
    }
}

#[derive(Debug)]
pub struct StatementColumnIter {
    current: c_int,
    count: c_int,
}

impl StatementColumnIter {
    const fn new(count: c_int) -> Self {
        Self { current: 0, count }
    }
}

impl Iterator for StatementColumnIter {
    type Item = Column;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current;

        if current < self.count {
            self.current = self.current + 1;
            Some(Column::new(current))
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct StatementParameters<'c, 's>
where
    'c: 's,
{
    statement: &'s Statement<'c>,
}

impl<'c, 's> StatementParameters<'c, 's>
where
    'c: 's,
{
    const fn new(statement: &'s Statement<'c>) -> Self {
        Self { statement }
    }

    pub fn name(&self, index: Index) -> Option<&str> {
        self.statement
            .internal_ref()
            .parameter_name(index)
            .map(|name| unsafe { str::from_utf8_unchecked(name.to_bytes()) })
    }

    pub fn index(&self, name: impl AsRef<str>) -> Option<Index> {
        let name = name.as_ref();

        for index in self.iter() {
            if let Some(n) = self.name(index)
                && name == n
            {
                return Some(index);
            }
        }

        None
    }

    pub fn iter(&self) -> impl Iterator<Item = Index> {
        StatementParameterIter::new(self)
    }

    pub fn len(&self) -> usize {
        self.count() as usize
    }

    fn count(&self) -> c_int {
        self.statement.internal_ref().parameter_count()
    }

    fn max(&self) -> Option<Index> {
        Index::new(self.count()).ok()
    }
}

impl<'c, 's> IntoIterator for StatementParameters<'c, 's>
where
    'c: 's,
{
    type Item = Index;
    type IntoIter = StatementParameterIter;

    fn into_iter(self) -> Self::IntoIter {
        StatementParameterIter::new(&self)
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(transparent)]
pub struct StatementParameterIter {
    state: StatementParameterIterState,
}

impl StatementParameterIter {
    fn new<'c, 's>(parameters: &StatementParameters<'c, 's>) -> Self
    where
        'c: 's,
    {
        let state = match parameters.max() {
            Some(max) => StatementParameterIterState::Next {
                current: Index::INITIAL,
                max,
            },
            None => StatementParameterIterState::Done,
        };

        Self { state }
    }
}

impl Iterator for StatementParameterIter {
    type Item = Index;

    fn next(&mut self) -> Option<Self::Item> {
        match self.state {
            StatementParameterIterState::Next { current, max } => {
                self.state = if current < max {
                    StatementParameterIterState::Next {
                        current: current.next(),
                        max,
                    }
                } else {
                    StatementParameterIterState::Done
                };

                Some(current)
            }
            StatementParameterIterState::Done => None,
        }
    }
}

#[derive(Copy, Clone, Debug)]
enum StatementParameterIterState {
    Next { current: Index, max: Index },
    Done,
}
