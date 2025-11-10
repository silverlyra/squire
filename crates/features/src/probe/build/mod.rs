//! Build-time SQLite feature detection.
//!
//! This module provides a way to probe SQLite's features at build time by
//! compiling and running a small C program that queries the SQLite library.
//!
//! This is intended to be used as a build-time dependency of the `squire-sys`
//! crate, called from its `build.rs` script.

mod compile;

use core::ffi::c_int;
use std::{collections::HashSet, error::Error, fmt};

use super::{Flag, Probe, Threading};
use crate::version::Version;

pub use compile::Build;

/// A SQLite library probed at build time.
///
/// This struct represents the results of compiling and running a C program
/// that queries SQLite's version, threading mode, and compile-time flags.
#[derive(Debug, Clone)]
pub struct Library {
    version: c_int,
    threading: c_int,
    flags: HashSet<Flag>,
}

impl Library {
    /// Probe a SQLite library by compiling and running a test program.
    ///
    /// This compiles the probe C program, links it against SQLite using
    /// the provided build configuration, runs it, and parses the output.
    ///
    /// # Panics
    ///
    /// Panics if compilation, linking, or execution fails.
    pub fn probe(build: Build) -> Self {
        compile::run_probe(build)
    }

    fn from_text(text: &str) -> Result<Self, ParseProbeError> {
        let mut lines = text.lines();

        // Parse version
        let version = lines
            .next()
            .ok_or(ParseProbeError::MissingVersion)?
            .trim()
            .parse()
            .map_err(|_| ParseProbeError::InvalidVersion)?;

        // Parse threading
        let threading = lines
            .next()
            .ok_or(ParseProbeError::MissingThreading)?
            .trim()
            .parse()
            .map_err(|_| ParseProbeError::InvalidThreading)?;

        // Skip blank line
        lines.next();

        // Parse flags - strip any `=value` suffix and parse into Flag enum
        let flags: HashSet<Flag> = lines
            .filter(|line| !line.trim().is_empty())
            .filter_map(|line| {
                let flag_name = line.trim().split('=').next()?;
                Flag::of(flag_name)
            })
            .collect();

        Ok(Self {
            version,
            threading,
            flags,
        })
    }
}

impl Probe for Library {
    fn version(&self) -> Version {
        Version::from_number(self.version)
    }

    fn is_set(&self, flag: Flag) -> bool {
        self.flags.contains(&flag)
    }

    fn threading(&self) -> Threading {
        match self.threading {
            0 => Threading::SingleThread,
            1 => Threading::Serialized,
            2 => Threading::MultiThread,
            _ => Threading::SingleThread,
        }
    }
}

/// Error parsing probe output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseProbeError {
    MissingVersion,
    InvalidVersion,
    MissingThreading,
    InvalidThreading,
}

impl Error for ParseProbeError {}

impl fmt::Display for ParseProbeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::MissingVersion => write!(f, "missing version line"),
            Self::InvalidVersion => write!(f, "invalid version number"),
            Self::MissingThreading => write!(f, "missing threading line"),
            Self::InvalidThreading => write!(f, "invalid threading mode"),
        }
    }
}
