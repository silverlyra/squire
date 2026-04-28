//! Build-time SQLite feature detection.
//!
//! This module provides a way to probe SQLite's features at build time by
//! compiling and running a small C program that queries the SQLite library.
//!
//! This is intended to be used as a build-time dependency of the `squire-sys`
//! crate, called from its `build.rs` script.

mod compile;

use core::{error::Error, ffi::c_int};
use std::{fmt, io, num::ParseIntError};

use crate::directive::{DirectiveMap, ParseDirectiveError};
use crate::info::Library;
use crate::probe::Probe;
use crate::version::{Override, Version};

pub use compile::Build;

impl Probe for Build {
    type Error = BuildProbeError;

    fn probe(&self) -> Result<Library, Self::Error> {
        let output = compile::run_probe(self)?;
        let (version, output) = output
            .split_once('\n')
            .ok_or(BuildProbeError::InvalidOutput)?;
        let (source_id, directives) = output
            .split_once('\n')
            .ok_or(BuildProbeError::InvalidOutput)?;

        let version: c_int = version.parse()?;
        let version = Version::from_number(version);

        let version = Override::check(version, source_id);

        let directives: DirectiveMap = directives.parse()?;

        Ok(Library::new(version, directives))
    }
}

/// Error parsing probe output.
#[derive(Debug)]
pub enum BuildProbeError {
    Directive(ParseDirectiveError),
    Execute(io::Error),
    Exit(std::process::ExitStatus),
    InvalidOutput,
    InvalidVersion,
    NoOutDir,
}

impl fmt::Display for BuildProbeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Directive(err) => write!(f, "{err}"),
            Self::Execute(err) => write!(f, "failed to execute probe.c: {err}"),
            Self::Exit(status) => write!(f, "probe exited with status {status}"),
            Self::InvalidOutput => write!(f, "invalid output from probe.c"),
            Self::InvalidVersion => write!(f, "invalid version number"),
            Self::NoOutDir => write!(f, "$OUT_DIR not set"),
        }
    }
}

impl Error for BuildProbeError {}

impl From<io::Error> for BuildProbeError {
    fn from(value: io::Error) -> Self {
        Self::Execute(value)
    }
}

impl From<ParseDirectiveError> for BuildProbeError {
    fn from(value: ParseDirectiveError) -> Self {
        Self::Directive(value)
    }
}

impl From<ParseIntError> for BuildProbeError {
    fn from(_value: ParseIntError) -> Self {
        Self::InvalidVersion
    }
}
