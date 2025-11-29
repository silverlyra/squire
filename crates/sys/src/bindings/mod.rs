#[cfg(feature = "bindgen")]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(not(feature = "bindgen"))]
mod default;
#[cfg(not(feature = "bindgen"))]
pub(crate) mod destructor;

#[cfg(not(feature = "bindgen"))]
pub use default::*;
