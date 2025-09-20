use crate::{
    error::Result,
    ffi,
    statement::{Binding, Statement},
};

pub use ffi::Index;

pub trait Bind<'b> {
    type Value: ffi::Bind<'b>;

    fn into_bind_value(self) -> Self::Value;
}

pub trait Parameters {
    type Indexes: Copy + Sized;

    fn resolve<'c>(statement: &Statement<'c>) -> Option<Self::Indexes>;

    fn bind<'c, 's>(
        self,
        statement: &'s mut Statement<'c>,
        indexes: Self::Indexes,
    ) -> Result<Binding<'c, 's>>;
}
