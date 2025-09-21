use sqlite::{SQLITE_PREPARE_DONT_LOG, SQLITE_PREPARE_NO_VTAB, SQLITE_PREPARE_PERSISTENT};

use crate::{
    connection::Connection,
    error::{Error, ErrorLocation, ErrorMessage, Result},
    ffi::{self, Execute},
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

impl<'c> ffi::Connected for Statement<'c> {
    fn as_connection_ptr(&self) -> *mut sqlite::sqlite3 {
        unsafe { self.inner.connection_ptr() }
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
        unsafe { self.inner.set(index, value.into_bind_value()?) }
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

/// Controls the behavior of [preparing](Statement::prepare()) a [`Statement`].
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct PrepareOptions(u32);

impl PrepareOptions {
    const DONT_LOG: u32 = SQLITE_PREPARE_DONT_LOG as u32;
    const PERSISTENT: u32 = SQLITE_PREPARE_PERSISTENT as u32;
    const NO_VTAB: u32 = SQLITE_PREPARE_NO_VTAB as u32;

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
