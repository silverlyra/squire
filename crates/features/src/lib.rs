//! # squire-sqlite3-features
//!
//! [Squire][] is a crate for embedding [SQLite][] in Rust. This crate performs
//! feature and version detection of SQLite.
//!
//! Users of Squire don't need to interact with this crate directly, and can
//! treat it as an implementation detail.
//!
//! [Squire]: https://lib.rs/squire
//! [SQLite]: https://sqlite.org/
//! [C API]: https://sqlite.org/cintro.html

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg), deny(rustdoc::broken_intra_doc_links))]

#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std as alloc;

pub mod directive;
pub mod feature;
#[cfg(feature = "alloc")]
mod info;
#[cfg(feature = "metadata")]
mod metadata;
mod probe;
mod version;

#[cfg(feature = "alloc")]
pub use directive::DirectiveMap;
pub use directive::{Directive, DirectiveKey};
pub use feature::{Configuration, Feature, FeatureKey};
#[cfg(feature = "alloc")]
pub use info::Library;
#[cfg(feature = "metadata")]
pub use metadata::Metadata;
pub use probe::Probe;
#[cfg(feature = "build")]
#[cfg_attr(docsrs, doc(cfg(feature = "build")))]
pub use probe::build::{Build, BuildProbeError};
#[cfg(feature = "dynamic")]
#[cfg_attr(docsrs, doc(cfg(feature = "dynamic")))]
pub use probe::dynamic;
pub use version::Version;
