//! # squire-sqlite3-sys
//!
//! [Squire][] is a crate for embedding [SQLite][] in Rust. This crate links
//! SQLite into the application, exposing the [C API][] of SQLite to Rust.
//!
//! Users of Squire don't need to interact with this crate directly, and can
//! treat it as an implementation detail.
//!
//! [Squire]: https://lib.rs/squire
//! [SQLite]: https://sqlite.org/
//! [C API]: https://sqlite.org/cintro.html

#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

/// Generated `unsafe extern "C"` bindings from [bindgen][].
///
/// [bindgen]: https://rust-lang.github.io/rust-bindgen/
mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub use bindings::*;

/// Use `SQLITE_STATIC` as a SQLite [destructor](sqlite_destructor_type)
/// argument to signal to SQLite that the memory allocation will outlive the
/// (e.g.) prepared statement.
///
/// SQLite will not clone the provided data, nor will it run a destructor. To
/// instead have SQLite clone a borrowed buffer, use [`SQLITE_TRANSIENT`].
///
/// (See the [SQLite reference][] for details.)
///
/// [SQLite reference]: https://sqlite.org/c3ref/c_static.html
pub const SQLITE_STATIC: sqlite3_destructor_type = sqlite3_destructor_type::from_sentinel(0);

/// Use `SQLITE_TRANSIENT` as a SQLite [destructor](sqlite_destructor_type)
/// argument to instruct SQLite to clone the provided value before returning,
/// and for SQLite to take responsibility to free the memory it allocated for
/// the clone when it is no longer needed.
///
/// (See the [SQLite reference][] for details.)
///
/// [SQLite reference]: https://sqlite.org/c3ref/c_static.html
pub const SQLITE_TRANSIENT: sqlite3_destructor_type = sqlite3_destructor_type::from_sentinel(-1);
