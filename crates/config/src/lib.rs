#![cfg_attr(not(feature = "std"), no_std)]
#![warn(clippy::pedantic)]
#![cfg_attr(docsrs, feature(doc_cfg), deny(rustdoc::broken_intra_doc_links))]
#![doc = concat!("# ", env!("CARGO_PKG_NAME"))]
//!
//! [Squire][] is a crate for embedding [SQLite][] in Rust. This crate
//! represents SQLite’s [compile-time options][], [library settings][], and
//! [connection settings][].
//!
//! Users of Squire don't need to interact with this crate directly, and can
//! treat it as an implementation detail.
//!
//! [Squire]: https://lib.rs/squire
//! [SQLite]: https://sqlite.org/
//! [C API]: https://sqlite.org/cintro.html
//!
//! [compile-time options]: crate::build
//! [connection settings]: crate::connection
//! [library settings]: crate::library

#[cfg(feature = "build")]
#[cfg_attr(docsrs, doc(cfg(feature = "build")))]
pub mod build;

pub mod types;
