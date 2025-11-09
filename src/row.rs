use crate::{
    column::{ColumnIndexes, Columns},
    error::{Error, Result},
    iter,
    statement::{Binding, Execute, Execution, Statement},
    types::ColumnIndex,
    value::Fetch,
};

#[derive(Debug)]
pub struct Rows<'c, 's, C: ColumnIndexes, S = Binding<'c, 's>>
where
    S: Execute<'c, 's>,
    'c: 's,
{
    execution: Execution<'c, 's, S>,
    indexes: C::Indexes,
}

impl<'c, 's, C, S> Rows<'c, 's, C, S>
where
    C: ColumnIndexes,
    S: Execute<'c, 's>,
    'c: 's,
{
    pub(crate) fn new(execution: Execution<'c, 's, S>) -> Result<Self> {
        if let Some(indexes) = C::resolve(execution.cursor()) {
            Ok(Self { execution, indexes })
        } else {
            Err(Error::resolve("failed to resolve column indexes"))
        }
    }
}

#[allow(clippy::should_implement_trait)]
impl<'c, 's, 'r, C, S> Rows<'c, 's, C, S>
where
    C: Columns<'r>,
    S: Execute<'c, 's>,
    'c: 's,
    's: 'r,
{
    /// [Fetch](Columns::fetch) the next row.
    ///
    /// If fetching fails, returns an error. If no more rows are available,
    /// returns `Ok(None)`. Otherwise, returns `Ok(Some(C))`.
    pub fn next(&'r mut self) -> Result<Option<C>> {
        unsafe { self.advance() }
    }

    /// Call a closure to transform each row in the result set.
    ///
    /// Like [`Iterator::map`], but can map over [`Columns`] which borrow data
    /// from the SQLite row. The returned `T` must not borrow from the row.
    pub fn map<F, T: 's>(self, f: F) -> iter::Map<'c, 's, C, F, S>
    where
        F: FnMut(C) -> T,
    {
        iter::Map { rows: self, f }
    }

    /// Call a closure to transform each row in the result set into an
    /// [`Option`], and filter out any rows where the closure returned `None`.
    ///
    /// Like [`Iterator::filter_map`], but can map over [`Columns`] which borrow
    /// data from the SQLite row. The returned `T` must not borrow from the row.
    pub fn filter_map<F, T: 's>(self, f: F) -> iter::FilterMap<'c, 's, C, F, S>
    where
        F: FnMut(C) -> Option<T>,
    {
        iter::FilterMap { rows: self, f }
    }

    /// # Safety
    ///
    /// This function must not be called while any data borrowed from a previous
    /// row is still in use. The public `next()` method enforces this via `&mut self`,
    /// but internal code may bypass this for performance in controlled scenarios.
    pub(crate) unsafe fn advance(&self) -> Result<Option<C>> {
        let statement = self.execution.cursor();

        // SAFETY: This always is an &'r Statement
        let statement =
            unsafe { core::mem::transmute::<&Statement<'_>, &'r Statement<'s>>(statement) };

        let more = unsafe { statement.internal_ref().row()? };

        if more {
            Ok(Some(C::fetch(statement, self.indexes)?))
        } else {
            Ok(None)
        }
    }
}

// IntoIterator implementation for owned (non-borrowing) Columns types
impl<'c, 's, C, S> IntoIterator for Rows<'c, 's, C, S>
where
    C: for<'r> Columns<'r> + 'static,
    S: Execute<'c, 's>,
    'c: 's,
{
    type Item = Result<C>;
    type IntoIter = RowsIterator<'c, 's, C, S>;

    fn into_iter(self) -> Self::IntoIter {
        RowsIterator { rows: self }
    }
}

#[derive(Debug)]
pub struct RowsIterator<'c, 's, C: ColumnIndexes, S = Binding<'c, 's>>
where
    S: Execute<'c, 's>,
    'c: 's,
{
    rows: Rows<'c, 's, C, S>,
}

impl<'c, 's, C, S> Iterator for RowsIterator<'c, 's, C, S>
where
    C: for<'r> Columns<'r> + 'static,
    S: Execute<'c, 's>,
    'c: 's,
{
    type Item = Result<C>;

    fn next(&mut self) -> Option<Self::Item> {
        Rows::next(&mut self.rows).transpose()
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct Row<'c, 's, 'r, S = Binding<'c, 's>>
where
    S: Execute<'c, 's>,
    'c: 's,
    's: 'r,
{
    execution: &'r mut Execution<'c, 's, S>,
}

impl<'c, 's, 'r, S> Row<'c, 's, 'r, S>
where
    S: Execute<'c, 's>,
    'c: 's,
    's: 'r,
{
    #[inline]
    pub(crate) const fn new(execution: &'r mut Execution<'c, 's, S>) -> Self {
        Self { execution }
    }

    pub fn fetch<'a, T: Fetch<'r>>(&'a mut self, column: ColumnIndex) -> Result<T>
    where
        'a: 'r,
    {
        let statement = self.execution.cursor();
        T::fetch(statement, column)
    }

    pub fn unpack<'a, T: Columns<'r>>(&'a mut self, indexes: T::Indexes) -> Result<T>
    where
        'a: 'r,
    {
        let statement = self.execution.cursor();
        T::fetch(statement, indexes)
    }
}
