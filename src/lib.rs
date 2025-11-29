//! Squire provides an idiomatic and performant Rust interface for [SQLite][].
//!
#![cfg_attr(feature = "derive", doc = "```rust")]
#![cfg_attr(not(feature = "derive"), doc = "```ignore")]
//! # #![cfg_attr(
//! #     all(nightly, feature = "lang-array-assume-init"),
//! #     feature(maybe_uninit_array_assume_init)
//! # )]
//! use squire::{Columns, Connection, Database};
//!
//! #[derive(Columns, PartialEq, Eq, Clone, Debug)]
//! struct User {
//!     id: u64,
//!     username: String,
//!     email: Option<String>,
//! }
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let connection = Connection::open(Database::memory())?;
//!
//! connection.execute(
//!     "CREATE TABLE users (
//!         id INTEGER PRIMARY KEY,
//!         username TEXT NOT NULL,
//!         email TEXT
//!      ) STRICT",
//!      (), // no parameters
//! )?;
//!
//! let mut add_user = connection.prepare("INSERT INTO users (username, email) VALUES (?, ?)")?;
//!
//! add_user.execute(("alice", "alice@example.com"))?;
//! add_user.execute(("bob", None::<&str>))?;
//!
//! let mut select_users = connection.prepare("SELECT * FROM users")?;
//! let users: Vec<User> = select_users.query(())?.all()?;
//!
//! assert_eq!(
//!     users,
//!     vec![
//!         User { id: 1, username: "alice".to_owned(), email: Some("alice@example.com".to_owned()) },
//!         User { id: 2, username: "bob".to_owned(), email: None },
//!     ],
//! );
//! # Ok(())
//! # }
//! ```
//!
//! [SQLite]: https://sqlite.org/

#![cfg_attr(
    all(nightly, feature = "lang-array-assume-init"),
    feature(maybe_uninit_array_assume_init)
)]
#![cfg_attr(
    all(nightly, feature = "lang-rustc-scalar-valid-range"),
    allow(internal_features)
)]
#![cfg_attr(
    all(nightly, feature = "lang-rustc-scalar-valid-range"),
    feature(rustc_attrs)
)]
#![cfg_attr(all(nightly, feature = "lang-step-trait"), feature(step_trait))]
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
    ErrorCategory, ErrorCode, ErrorContainer, ErrorLocation, ErrorReason, FetchError, GeneralError,
    IntegrationError, IoError, LockedError, ParameterError, ReadOnlyError, Result,
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
