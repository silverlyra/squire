use core::error::Error;

use crate::info::Library;

#[cfg(feature = "build")]
pub mod build;
#[cfg(feature = "dynamic")]
pub mod dynamic;

/// A SQLite [`Library`] whose [version](crate::Version) and
/// [compile-time options](crate::Directive) are being probed.
pub trait Probe {
    type Error: Error + 'static;

    /// Probe the [`Library`].
    fn probe(&self) -> Result<Library, Self::Error>;
}
