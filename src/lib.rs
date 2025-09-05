#![cfg_attr(feature = "nightly", feature(inherent_associated_types))]
#![cfg_attr(feature = "nightly", feature(step_trait))]

mod connection;
mod database;
mod error;
pub mod ffi;
pub mod types;

pub use error::{
    AbortError, AuthorizationError, BusyError, CantOpenError, ConstraintError, CorruptError, Error,
    ErrorCategory, ErrorCode, GeneralError, IoError, LockedError, ReadOnlyError, Result,
};
