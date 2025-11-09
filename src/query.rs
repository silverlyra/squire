use crate::{
    column::Columns,
    connection::Connection,
    error::Result,
    param::Parameters,
    statement::{Binding, Execution, Statement},
};

pub trait Query<'s>: Parameters<'s> {
    type Output;

    fn prepare(connection: &Connection) -> Result<Statement<'_>>;

    fn resolve<'c: 's>(execution: Execution<'c, 's>) -> Result<Self::Output>;
}
