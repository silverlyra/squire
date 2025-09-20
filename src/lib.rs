#![cfg_attr(feature = "nightly", feature(inherent_associated_types))]
#![cfg_attr(feature = "nightly", feature(step_trait))]

mod connection;
mod database;
mod error;
pub mod ffi;
mod param;
mod statement;
pub mod types;

pub use connection::{Connection, ConnectionBuilder};
pub use database::{Database, IntoLocation};
pub use error::{
    AbortError, AuthorizationError, BusyError, CantOpenError, ConstraintError, CorruptError, Error,
    ErrorCategory, ErrorCode, GeneralError, IoError, LockedError, ReadOnlyError, Result,
};
pub use statement::Statement;
pub use types::RowId;
