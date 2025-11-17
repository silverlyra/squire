#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg), deny(rustdoc::broken_intra_doc_links))]

#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std as alloc;

pub use serde::{Deserialize, Deserializer, Serialize, Serializer, de, ser};

#[cfg(feature = "json")]
#[cfg_attr(docsrs, doc(cfg(feature = "json")))]
pub use serde_json as json;

#[cfg(feature = "jsonb")]
#[cfg_attr(docsrs, doc(cfg(feature = "jsonb")))]
pub use serde_sqlite_jsonb as jsonb;
