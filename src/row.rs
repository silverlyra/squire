use crate::{
    column::{ColumnIndexes, Columns},
    error::{Error, Result},
    statement::{Binding, Execute, Execution},
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

impl<'c, 's, 'r, C, S> Rows<'c, 's, C, S>
where
    C: Columns<'r>,
    S: Execute<'c, 's>,
    'c: 's,
    's: 'r,
{
    pub fn next(&'r mut self) -> Result<Option<C>> {
        let more = unsafe { self.advance()? };

        if more {
            let statement = self.execution.cursor();
            Ok(Some(C::fetch(statement, self.indexes)?))
        } else {
            Ok(None)
        }
    }

    pub fn map<F, T: 's>(self, f: F) -> Map<'c, 's, C, F, S>
    where
        F: FnMut(C) -> T,
    {
        Map { rows: self, f }
    }

    pub fn filter<F>(self, predicate: F) -> Filter<'c, 's, C, F, S>
    where
        F: FnMut(&C) -> bool,
    {
        Filter {
            rows: self,
            predicate,
        }
    }

    pub fn filter_map<F, T: 's>(self, f: F) -> FilterMap<'c, 's, C, F, S>
    where
        F: FnMut(C) -> Option<T>,
    {
        FilterMap { rows: self, f }
    }
}

impl<'c, 's, C, S> Rows<'c, 's, C, S>
where
    C: ColumnIndexes,
    S: Execute<'c, 's>,
    'c: 's,
{
    /// # Safety
    ///
    /// This function must not be called while any data borrowed from a previous
    /// row is still in use. The public `next()` method enforces this via `&mut self`,
    /// but internal code may bypass this for performance in controlled scenarios.
    unsafe fn advance(&self) -> Result<bool> {
        let statement = self.execution.cursor();
        unsafe { statement.internal_ref().row() }
    }
}

// IntoIterator implementation for owned (non-borrowing) Columns types
impl<'c, 's, C, S> IntoIterator for Rows<'c, 's, C, S>
where
    C: ColumnIndexes + for<'r> Columns<'r> + 'static,
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

// Map combinator
#[derive(Debug)]
pub struct Map<'c, 's, C, F, S = Binding<'c, 's>>
where
    C: ColumnIndexes,
    S: Execute<'c, 's>,
    'c: 's,
{
    rows: Rows<'c, 's, C, S>,
    f: F,
}

impl<'c, 's, 'r, C, F, T, S> Map<'c, 's, C, F, S>
where
    C: Columns<'r>,
    F: FnMut(C) -> T,
    S: Execute<'c, 's>,
    'c: 's,
    's: 'r,
{
    pub fn next(&'r mut self) -> Result<Option<T>> {
        Ok(self.rows.next()?.map(|item| (self.f)(item)))
    }
}

impl<'c, 's, C, F, T, S> Iterator for Map<'c, 's, C, F, S>
where
    C: for<'r> Columns<'r>,
    F: for<'r> FnMut(C) -> T,
    T: 's,
    S: Execute<'c, 's>,
    'c: 's,
{
    type Item = Result<T>;

    fn next(&mut self) -> Option<Self::Item> {
        // SAFETY: We never hold onto row data across loop iterations
        let more = match unsafe { self.rows.advance() } {
            Ok(m) => m,
            Err(e) => return Some(Err(e)),
        };

        if !more {
            return None;
        }

        let statement = self.rows.execution.cursor();
        let item = match C::fetch(statement, self.rows.indexes) {
            Ok(i) => i,
            Err(e) => return Some(Err(e)),
        };

        Some(Ok((self.f)(item)))
    }
}

// Filter combinator
#[derive(Debug)]
pub struct Filter<'c, 's, C, F, S = Binding<'c, 's>>
where
    C: ColumnIndexes,
    S: Execute<'c, 's>,
    'c: 's,
{
    rows: Rows<'c, 's, C, S>,
    predicate: F,
}

impl<'c, 's, 'r, C, F, S> Filter<'c, 's, C, F, S>
where
    C: Columns<'r>,
    F: FnMut(&C) -> bool,
    S: Execute<'c, 's>,
    'c: 's,
    's: 'r,
{
    pub fn next(&'r mut self) -> Result<Option<C>> {
        loop {
            // SAFETY: We never hold onto row data across loop iterations,
            // so it's safe to call advance() multiple times
            let more = unsafe { self.rows.advance()? };

            if !more {
                return Ok(None);
            }

            let statement = self.rows.execution.cursor();
            let item = C::fetch(statement, self.rows.indexes)?;

            if (self.predicate)(&item) {
                return Ok(Some(item));
            }
            // Otherwise continue looping
        }
    }
}

impl<'c, 's, C, F, S> Iterator for Filter<'c, 's, C, F, S>
where
    C: ColumnIndexes + for<'r> Columns<'r>,
    F: for<'r> FnMut(&C) -> bool,
    S: Execute<'c, 's>,
    'c: 's,
{
    type Item = Result<C>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // SAFETY: We never hold onto row data across loop iterations
            let more = match unsafe { self.rows.advance() } {
                Ok(m) => m,
                Err(e) => return Some(Err(e)),
            };

            if !more {
                return None;
            }

            let statement = self.rows.execution.cursor();
            let item = match C::fetch(statement, self.rows.indexes) {
                Ok(i) => i,
                Err(e) => return Some(Err(e)),
            };

            if (self.predicate)(&item) {
                return Some(Ok(item));
            }
            // Otherwise continue looping
        }
    }
}

// FilterMap combinator
#[derive(Debug)]
pub struct FilterMap<'c, 's, C, F, S = Binding<'c, 's>>
where
    C: ColumnIndexes,
    S: Execute<'c, 's>,
    'c: 's,
{
    rows: Rows<'c, 's, C, S>,
    f: F,
}

impl<'c, 's, 'r, C, F, T, S> FilterMap<'c, 's, C, F, S>
where
    C: Columns<'r>,
    F: FnMut(C) -> Option<T>,
    S: Execute<'c, 's>,
    'c: 's,
    's: 'r,
{
    pub fn next(&'r mut self) -> Result<Option<T>> {
        loop {
            // SAFETY: We never hold onto row data across loop iterations,
            // so it's safe to call advance() multiple times
            let more = unsafe { self.rows.advance()? };

            if !more {
                return Ok(None);
            }

            let statement = self.rows.execution.cursor();
            let item = C::fetch(statement, self.rows.indexes)?;

            if let Some(mapped) = (self.f)(item) {
                return Ok(Some(mapped));
            }
            // Otherwise continue looping
        }
    }
}

impl<'c, 's, C, F, T, S> Iterator for FilterMap<'c, 's, C, F, S>
where
    C: ColumnIndexes + for<'r> Columns<'r>,
    F: for<'r> FnMut(C) -> Option<T>,
    T: 's,
    S: Execute<'c, 's>,
    'c: 's,
{
    type Item = Result<T>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // SAFETY: We never hold onto row data across loop iterations
            let more = match unsafe { self.rows.advance() } {
                Ok(m) => m,
                Err(e) => return Some(Err(e)),
            };

            if !more {
                return None;
            }

            let statement = self.rows.execution.cursor();
            let item = match C::fetch(statement, self.rows.indexes) {
                Ok(i) => i,
                Err(e) => return Some(Err(e)),
            };

            if let Some(mapped) = (self.f)(item) {
                return Some(Ok(mapped));
            }
            // Otherwise continue looping
        }
    }
}
