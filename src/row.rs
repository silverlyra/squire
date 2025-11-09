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
        let statement = self.execution.cursor();
        let more = unsafe { statement.internal_ref().row() }?;

        if more {
            Ok(Some(C::fetch(statement, self.indexes)?))
        } else {
            Ok(None)
        }
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
