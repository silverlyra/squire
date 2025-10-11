#![cfg_attr(feature = "lang-iat", feature(inherent_associated_types))]
#![cfg_attr(feature = "lang-rustc-scalar-valid-range", feature(rustc_attrs))]
#![cfg_attr(feature = "lang-step-trait", feature(step_trait))]
#![cfg_attr(docsrs, feature(doc_cfg), deny(rustdoc::broken_intra_doc_links))]

mod bind;
mod blob;
mod column;
mod connection;
mod database;
mod error;
pub mod ffi;
mod param;
mod row;
mod statement;
mod types;
mod value;

pub use bind::{Bind, Index};
pub use blob::Reservation;
pub use column::{Column, Columns};
pub use connection::{Connection, ConnectionBuilder};
pub use database::{Database, IntoLocation};
pub use error::{
    AbortError, AuthorizationError, BusyError, CantOpenError, ConstraintError, CorruptError, Error,
    ErrorCategory, ErrorCode, GeneralError, IoError, LockedError, ParameterError, ReadOnlyError,
    Result,
};
pub use param::Parameters;
pub use statement::{Binding, Execution, PrepareOptions, Statement, StatementParameters};
pub use types::RowId;
pub use value::Fetch;
