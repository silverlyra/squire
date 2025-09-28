use crate::{
    error::Result,
    statement::{Execute, Execution},
    value::{Column, Fetch},
};

#[derive(Debug)]
#[repr(transparent)]
pub struct Row<'c, 'r, S>
where
    S: Execute<'c>,
    'c: 'r,
{
    execution: &'r mut Execution<'c, S>,
}

impl<'c, 'r, S> Row<'c, 'r, S>
where
    S: Execute<'c>,
    'c: 'r,
{
    #[inline]
    pub(crate) const fn new(execution: &'r mut Execution<'c, S>) -> Self {
        Self { execution }
    }

    pub fn fetch<'a, T: Fetch<'r>>(&'a mut self, column: Column) -> Result<T>
    where
        'a: 'r,
    {
        let statement = self.execution.cursor();
        T::fetch(statement, column)
    }
}
