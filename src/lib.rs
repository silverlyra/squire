#![cfg_attr(
    feature = "lang-array-assume-init",
    feature(maybe_uninit_array_assume_init)
)]
#![cfg_attr(feature = "lang-iat", feature(inherent_associated_types))]
#![cfg_attr(feature = "lang-rustc-scalar-valid-range", allow(internal_features))]
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
pub mod iter;
mod param;
mod query;
mod row;
mod statement;
mod types;
mod value;

pub use bind::Bind;
pub use blob::Reservation;
pub use column::{ColumnIndexes, Columns};
pub use connection::{Connection, ConnectionBuilder};
pub use database::{Database, IntoLocation};
pub use error::{
    AbortError, AuthorizationError, BusyError, CantOpenError, ConstraintError, CorruptError, Error,
    ErrorCategory, ErrorCode, ErrorContext, ErrorLocation, ErrorMessage, FetchError, GeneralError,
    IoError, LockedError, ParameterError, ReadOnlyError, Result,
};
pub use param::Parameters;
pub use query::Query;
pub use row::{Row, Rows};
pub use statement::{
    Binding, Execution, PrepareOptions, Statement, StatementColumns, StatementParameters,
};
pub use types::{BindIndex, BindIndexes, Borrowed, ColumnIndex, RowId, Type};
pub use value::Fetch;

#[cfg(all(feature = "json", feature = "serde"))]
pub use types::Json;
#[cfg(all(feature = "jsonb", feature = "serde"))]
pub use types::Jsonb;

#[cfg(feature = "derive")]
#[cfg_attr(docsrs, doc(cfg(feature = "derive")))]
pub use squire_derive::{Columns, Parameters};
