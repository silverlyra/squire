//! The `ffi` module gives lower-level [`unsafe`][] access to the SQLite API.
//!
//! Like the rest of Squire, `ffi` provides Rust interfaces to SQLite. For
//! example, there is an [`ffi::Statement`](Statement) shadowing the main
//! [`Statement`](crate::Statement) struct. While `Statement` ensures the SQLite
//! API is used correctly, in ways that won't crash your program or corrupt its
//! memory, with `ffi::Statement` it's your responsibility to use the API
//! correctly.
//!
//! [`unsafe`]: https://doc.rust-lang.org/book/ch20-01-unsafe-rust.html

mod bind;
mod bytes;
mod call;
mod connection;
#[cfg(feature = "mutex")]
mod mutex;
mod statement;
mod string;
mod value;

pub use crate::types::ColumnIndex;
pub use bind::{Bind, destructor};
pub use bytes::Bytes;
pub use connection::{Connected, Connection};
#[cfg(feature = "mutex")]
#[cfg_attr(docsrs, doc(cfg(any(feature = "mutex", feature = "serialized"))))]
pub use mutex::{Mutex, MutexGuard, MutexRef, StaticMutex};
pub use statement::{Conclusion, Execute, Statement};
pub use string::{Append, String, StringBuilder};
pub use value::Fetch;
