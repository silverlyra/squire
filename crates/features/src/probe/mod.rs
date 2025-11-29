use core::{error::Error, fmt, str::FromStr};

use crate::version::Version;

#[cfg(feature = "build")]
pub mod build;
#[cfg(feature = "dynamic")]
pub mod dynamic;
mod flag;

pub use flag::Flag;

/// A SQLite library whose version and included features are being probed.
pub trait Probe {
    /// Get the [`Version`] of the SQLite library.
    fn version(&self) -> Version;

    /// Check if a SQLite [compile-time option][] ([`Flag`]) was set.
    ///
    /// [compile-time option]: https://sqlite.org/compile.html
    fn is_set(&self, flag: Flag) -> bool;

    /// Check the [thread safety][] ([`Threading`]) of the library.
    ///
    /// [thread safety]: https://sqlite.org/threadsafe.html
    fn threading(&self) -> Threading;
}

/// The threading mode that SQLite was built with.
///
/// If [single-threaded](Threading::SingleThread), none of the SQLite functions
/// are re-entrant, and they cannot be called from multiple threads.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[repr(usize)]
pub enum Threading {
    SingleThread = 0,
    MultiThread = 1,
    Serialized = 2,
}

impl Threading {
    /// `true` if the library was built without thread safety.
    pub const fn is_single_threaded(&self) -> bool {
        matches!(*self, Self::SingleThread)
    }

    /// `true` if the library was built with thread safety.
    pub const fn is_thread_safe(&self) -> bool {
        matches!(*self, Self::MultiThread | Self::Serialized)
    }

    /// Returns the string representation matching Cargo feature names.
    ///
    /// ```rust
    /// # use squire_sqlite3_features::Threading;
    /// assert_eq!(Threading::MultiThread.as_str(), "multi-thread");
    /// ```
    pub const fn as_str(&self) -> &'static str {
        match *self {
            Threading::SingleThread => "single-thread",
            Threading::MultiThread => "multi-thread",
            Threading::Serialized => "serialized",
        }
    }
}

impl fmt::Display for Threading {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for Threading {
    type Err = UnknownThreadingMode;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "single-thread" => Ok(Threading::SingleThread),
            "multi-thread" => Ok(Threading::MultiThread),
            "serialized" => Ok(Threading::Serialized),
            _ => Err(UnknownThreadingMode),
        }
    }
}

/// Error returned when parsing an unknown threading mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnknownThreadingMode;

impl fmt::Display for UnknownThreadingMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown threading mode")
    }
}

impl Error for UnknownThreadingMode {}
