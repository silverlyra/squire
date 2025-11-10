//! Internal implementation for compiling and running the SQLite probe program.

use std::{env, path::PathBuf, process::Command};

use super::Library;

/// The probe C source code, embedded in the binary.
const PROBE_C: &str = include_str!("probe.c");

/// Build instructions for compiling the SQLite probe program.
///
/// This struct encapsulates the information needed to compile and link
/// the probe program against a SQLite library.
#[derive(Debug, Clone)]
pub struct Build {
    /// The output directory for compiled artifacts (usually `$OUT_DIR`)
    pub(super) out_dir: Option<PathBuf>,
    /// The name of the SQLite library to link against
    pub(super) lib_name: String,
    /// Additional include paths for the C compiler
    pub(super) include_paths: Vec<PathBuf>,
    /// Additional library search paths for the linker
    pub(super) link_paths: Vec<PathBuf>,
}

impl Build {
    /// Create a new `Build` with the given library name.
    ///
    /// If no include or link paths are provided, the compiler will use
    /// default system paths.
    pub fn new(lib_name: impl Into<String>) -> Self {
        Self {
            out_dir: None,
            lib_name: lib_name.into(),
            include_paths: Vec::new(),
            link_paths: Vec::new(),
        }
    }

    /// Set the output directory for compiled artifacts.
    ///
    /// If not set, `$OUT_DIR` will be used.
    pub fn out_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.out_dir = Some(path.into());
        self
    }

    /// Add an include path for the C compiler.
    pub fn include_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.include_paths.push(path.into());
        self
    }

    /// Add a library search path for the linker.
    pub fn link_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.link_paths.push(path.into());
        self
    }

    /// Create a `Build` from pkg-config.
    ///
    /// This queries pkg-config for the `sqlite3` package and uses the
    /// returned include and link paths.
    #[cfg(feature = "pkg-config")]
    pub fn from_pkg_config() -> Result<Self, pkg_config::Error> {
        let library = pkg_config::Config::new().probe("sqlite3")?;

        let mut build = Self::new("sqlite3");

        for include in library.include_paths {
            build = build.include_path(include);
        }

        for link_path in library.link_paths {
            build = build.link_path(link_path);
        }

        Ok(build)
    }
}

impl Default for Build {
    /// Create a default `Build` configuration.
    ///
    /// This tries to use pkg-config to locate SQLite (if the `pkg-config`
    /// feature is enabled). If that fails, it returns a minimal configuration
    /// that links against `sqlite3` using default system paths.
    fn default() -> Self {
        #[cfg(feature = "pkg-config")]
        {
            Self::from_pkg_config().unwrap_or_else(|_| Self::new("sqlite3"))
        }

        #[cfg(not(feature = "pkg-config"))]
        {
            Self::new("sqlite3")
        }
    }
}

/// Compile and run the SQLite probe program.
///
/// This uses the `cc` crate to compile the probe program and link it against
/// SQLite using the provided build configuration.
pub(super) fn run_probe(build: Build) -> Library {
    let out_dir = build
        .out_dir
        .unwrap_or_else(|| PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set")));

    // Write probe.c to a temporary file in OUT_DIR
    let probe_c_path = out_dir.join("probe.c");
    std::fs::write(&probe_c_path, PROBE_C).expect("failed to write probe.c");

    // Get the compiler
    let compiler = cc::Build::new().get_compiler();

    // Build the compile command
    let mut cmd = compiler.to_command();
    cmd.arg(&probe_c_path);

    // Add include paths
    for include in &build.include_paths {
        cmd.arg(format!("-I{}", include.display()));
    }

    // Add library search paths
    for link_path in &build.link_paths {
        cmd.arg(format!("-L{}", link_path.display()));
    }

    // Link to SQLite
    cmd.arg(format!("-l{}", build.lib_name));

    // Set output
    let probe_exe = out_dir.join("sqlite_probe");
    cmd.arg("-o").arg(&probe_exe);

    // Compile and link
    let status = cmd
        .status()
        .expect("failed to compile and link probe program");

    if !status.success() {
        panic!("failed to compile and link probe program");
    }

    // Run the probe program and capture output
    let output = Command::new(&probe_exe)
        .output()
        .expect("failed to run probe executable");

    if !output.status.success() {
        panic!(
            "probe executable failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // Parse the output
    let text = String::from_utf8(output.stdout).expect("probe output is not valid UTF-8");
    Library::from_text(&text).expect("failed to parse probe output")
}
