use crate::{
    connection::Connection,
    error::{Error, Result},
    ffi,
    param::{Bind, Index, Parameters},
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

    pub fn binding(&mut self) -> Binding<'c, '_> {
        Binding {
            inner: self.inner.binding(),
        }
    }

    pub fn bind<'s, P>(&'s mut self, parameters: P) -> Result<Binding<'c, 's>>
    where
        P: Parameters,
    {
        let indexes = P::resolve(self).ok_or(Error::misuse())?;
        parameters.bind(self, indexes)
    }

    pub fn query<'s, P>(&'s mut self, parameters: P) -> Result<Execution<'c, ffi::Binding<'c, 's>>>
    where
        P: Parameters,
    {
        self.bind(parameters).map(Binding::done)
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct Binding<'c, 's>
where
    'c: 's,
{
    inner: ffi::Binding<'c, 's>,
}

impl<'c, 's> Binding<'c, 's>
where
    'c: 's,
{
    pub fn set<'b, B>(&'b mut self, index: Index, value: B) -> Result<()>
    where
        B: Bind<'b>,
    {
        unsafe { self.inner.set(index, value.into_bind_value()) }
    }

    pub fn ready<'b>(&'b mut self) -> Execution<'c, &'b mut ffi::Binding<'c, 's>> {
        Execution::new(self.inner.ready())
    }

    pub fn done(self) -> Execution<'c, ffi::Binding<'c, 's>> {
        Execution::new(self.inner.done())
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct Execution<'c, S>
where
    S: ffi::Execute<'c>,
{
    inner: ffi::Execution<'c, S>,
}

impl<'c, S> Execution<'c, S>
where
    S: ffi::Execute<'c>,
{
    #[inline]
    const fn new(inner: ffi::Execution<'c, S>) -> Self {
        Self { inner }
    }
}
