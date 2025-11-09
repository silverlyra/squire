use crate::{
    connection::Connection,
    error::Result,
    param::Parameters,
    statement::{Execution, Statement},
};

/// An SQL query which can [prepare](Statement::prepare) itself into a
/// [`Statement`], bind itself as [`Parameters`] to the statement (if any), and
/// read any desired output from the statement [execution](Execution).
pub trait Query<'s>: Parameters<'s> {
    type Output;

    /// [Prepare](Statement::prepare) a [`Statement`] to execute the [`Query`].
    fn prepare(connection: &Connection) -> Result<Statement<'_>>;

    /// Fetch the desired [`Query`] output from the [`Statement`].
    fn output<'c: 's>(execution: Execution<'c, 's>) -> Result<Self::Output>;
}
