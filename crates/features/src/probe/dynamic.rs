use core::ffi::{c_char, c_int};

use super::{Flag, Probe, Threading};
use crate::version::Version;

use libloading::AsFilename;
pub use libloading::Error;
#[cfg(feature = "std")]
use libloading::library_filename;
#[cfg(feature = "std")]
use std::ffi::OsStr;

type Sqlite3CompileOptionUsed = unsafe extern "C" fn(*const c_char) -> c_int;
type Sqlite3VersionNumber = unsafe extern "C" fn() -> c_int;
type Sqlite3ThreadSafe = unsafe extern "C" fn() -> c_int;

/// A dynamically loaded SQLite library.
///
/// This struct uses `libloading` to load a SQLite shared library at runtime
/// and probe its version, compile-time options, and threading mode.
pub struct Library {
    #[allow(dead_code)]
    lib: libloading::Library,
    option_used: Sqlite3CompileOptionUsed,
    version_number: Sqlite3VersionNumber,
    thread_safe: Sqlite3ThreadSafe,
}

#[allow(clippy::should_implement_trait)]
impl Library {
    /// Load a SQLite library from the given path.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the library at the given path is a valid
    /// SQLite shared library with the expected function signatures.
    pub fn open<P: AsFilename>(path: P) -> Result<Self, Error> {
        let lib = unsafe { libloading::Library::new(path)? };

        let compileoption_used =
            unsafe { *lib.get::<Sqlite3CompileOptionUsed>(b"sqlite3_compileoption_used\0")? };

        let libversion_number =
            unsafe { *lib.get::<Sqlite3VersionNumber>(b"sqlite3_libversion_number\0")? };

        let threadsafe = unsafe { *lib.get::<Sqlite3ThreadSafe>(b"sqlite3_threadsafe\0")? };

        Ok(Self {
            lib,
            option_used: compileoption_used,
            version_number: libversion_number,
            thread_safe: threadsafe,
        })
    }

    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    pub fn resolve<P: AsRef<OsStr>>(name: P) -> Result<Self, Error> {
        let filename = library_filename(name);
        Self::open(filename)
    }

    /// Load the system's default SQLite library.
    ///
    /// This attempts to load a library named "sqlite3" using the platform's
    /// standard naming convention (e.g., `libsqlite3.so` on Linux,
    /// `libsqlite3.dylib` on macOS, `sqlite3.dll` on Windows).
    ///
    /// # Safety
    ///
    /// The caller must ensure that the system's default SQLite library is
    /// compatible and has the expected function signatures.
    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    pub fn default() -> Result<Self, Error> {
        Self::resolve("sqlite3")
    }
}

impl Probe for Library {
    fn version(&self) -> Version {
        let num = unsafe { (self.version_number)() };
        Version::from_number(num)
    }

    fn is_set(&self, flag: Flag) -> bool {
        // Convert the flag name to a C string
        let name = flag.name();
        let c_name = name.as_bytes();

        // We need to create a null-terminated string
        let mut buf = [0u8; 64];
        let len = c_name.len().min(buf.len() - 1);
        buf[..len].copy_from_slice(&c_name[..len]);
        buf[len] = 0;

        let result = unsafe { (self.option_used)(buf.as_ptr() as *const c_char) };
        result != 0
    }

    fn threading(&self) -> Threading {
        let result = unsafe { (self.thread_safe)() };
        match result {
            0 => Threading::SingleThread,
            1 => Threading::Serialized,
            2 => Threading::MultiThread,
            _ => Threading::SingleThread, // Default to most conservative
        }
    }
}
