//! # squire-sqlite3-src
//!
//! This crate bundles the [SQLite][] source code for [Squire][]. When Squire’s
//! `bundled` feature flag is enabled, SQLite is built from the `sqlite3.c` file
//! included in this crate, and linked into the [`squire-sqlite3-sys`][sys]
//! crate.
//!
//! Users of Squire don’t need to be aware of or interact with this crate. Cargo
//! will automatically download and build it when you enable Squire’s `bundled`
//! feature.
//!
//! [SQLite]: https://sqlite.org/
//! [Squire]: https://github.com/silverlyra/squire
//! [sys]: https://lib.rs/squire-sqlite3-sys

use std::{env, iter, ops::Deref, path::PathBuf};

use features::{Configuration, DirectiveMap, Library, Version};

const VERSION: Version = {
    let version = include_str!("../sqlite/VERSION").trim_ascii_end();
    match Version::parse(version) {
        Ok(version) => version,
        Err(_) => panic!("invalid SQLite version"),
    }
};

/// The [version](Version) of the bundled SQLite library.
pub const fn version() -> Version {
    VERSION
}

/// Build the bundled SQLite sources, using the given
/// [compile-time options](DirectiveMap).
pub fn build(location: Location, directives: DirectiveMap) -> Build {
    let mut compiler = cc::Build::new();
    compiler.file(location.input());

    apply(&directives, &mut compiler);
    compiler.warnings(false);

    compiler.out_dir(&location.dest);
    compiler.compile("sqlite3");

    Build::new(location, directives)
}

/// Set [compile-time options](DirectiveMap) for a SQLite [build][].
pub fn config(configuration: Option<&Configuration>) -> DirectiveMap {
    use features::directive::*;

    let mut directives = [
        Directive::DefaultSynchronous(Synchronous::Full),
        Directive::DefaultWalSynchronous(Synchronous::Normal),
        Directive::Threading(Threading::MultiThread),
        Directive::DefaultAutomaticIndex,
        Directive::DefaultForeignKeys,
        Directive::DefaultMemoryStatus(false),
        Directive::DoubleQuotedStrings(DoubleQuotedStrings::default()),
        Directive::EnableMemoryManagement,
        Directive::LikeOperatorDoesntMatchBlob,
        Directive::MaxExpressionDepth(0),
        Directive::MaxMmapSize(0),
        Directive::OmitAuthorization,
        Directive::OmitAutomaticReset,
        Directive::OmitBlobIo,
        Directive::OmitColumnDeclaredType,
        Directive::OmitDeprecated,
        Directive::OmitGetTable,
        Directive::OmitProgressCallback,
        Directive::OmitSharedCache,
        Directive::OmitTrace,
        Directive::OmitUtf16,
        Directive::UseAlloca,
        Directive::UseDatabaseUri,
        #[cfg(debug_assertions)]
        Directive::Debug,
        #[cfg(debug_assertions)]
        Directive::EnableApiArmor,
    ]
    .into_iter()
    .collect();

    if let Some(configuration) = configuration {
        configuration.apply(version(), &mut directives);
    }

    directives
}

fn apply(directives: &DirectiveMap, compiler: &mut cc::Build) {
    for directive in directives.values() {
        let directive = format!("{directive}");

        match directive.split_once('=') {
            Some((name, value)) => compiler.define(name, value),
            None => compiler.define(&directive, None),
        };
    }
}

/// The output of [`Build`], including the [`Location`] SQLite was built into.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Build {
    location: Location,
    library: features::Library,
}

impl Build {
    pub const fn new(location: Location, directives: DirectiveMap) -> Build {
        Self {
            location,
            library: Library::new(version(), directives),
        }
    }

    /// The `.c` source files that need to be built (`sqlite3.c`).
    pub fn sources(&self) -> impl Iterator<Item = PathBuf> {
        iter::once(self.input())
    }

    /// The build [`Location`].
    pub const fn location(&self) -> &Location {
        &self.location
    }

    /// The SQLite [version](features::Version) and
    /// [compile-time options](features::Directive).
    pub const fn library(&self) -> &features::Library {
        &self.library
    }
}

impl Deref for Build {
    type Target = Location;

    fn deref(&self) -> &Self::Target {
        self.location()
    }
}

/// Specifies the source and target directories for [`build`].
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Location {
    src: PathBuf,
    dest: PathBuf,
}

impl Location {
    /// Create a build [`Location`] from `$CARGO_MANIFEST_DIR`.
    pub fn new(dest: impl Into<PathBuf>) -> Self {
        Self {
            src: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("sqlite"),
            dest: dest.into(),
        }
    }

    /// The path to `sqlite3.c`.
    pub fn input(&self) -> PathBuf {
        self.src.join("sqlite3.c")
    }

    /// The path to `sqlite3.h`.
    pub fn header(&self) -> PathBuf {
        self.src.join("sqlite3.h")
    }

    /// The build's target directory.
    pub fn dest(&self) -> PathBuf {
        self.dest.clone()
    }

    /// Iterates source files (`sqlite3.c` and `sqlite3.h`).
    pub fn sources(&self) -> impl Iterator<Item = PathBuf> {
        iter::once(self.input()).chain(iter::once(self.header()))
    }
}

impl Default for Location {
    fn default() -> Self {
        Self::new(PathBuf::from(
            env::var_os("OUT_DIR").expect("$OUT_DIR not set"),
        ))
    }
}
