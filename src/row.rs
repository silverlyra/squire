use crate::{
    column::{Column, Columns},
    error::Result,
    statement::{Binding, Execute, Execution},
    value::Fetch,
};

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

    pub fn fetch<'a, T: Fetch<'r>>(&'a mut self, column: Column) -> Result<T>
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
