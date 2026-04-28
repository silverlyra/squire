use core::ffi::{CStr, c_int};

use crate::directive::ParseDirectiveError;
#[cfg(feature = "alloc")]
use crate::probe::Probe;
use crate::version::Version;
#[cfg(feature = "alloc")]
use crate::{directive::DirectiveMap, version::Override};

use libloading::AsFilename;
pub use libloading::Error;
#[cfg(feature = "std")]
use libloading::library_filename;
#[cfg(feature = "std")]
use std::ffi::OsStr;

mod sqlite {
    use core::ffi::{c_char, c_int};

    pub(super) type CompileOptionGet = unsafe extern "C" fn(c_int) -> *const c_char;
    pub(super) type SourceId = unsafe extern "C" fn() -> *const c_char;
    pub(super) type VersionNumber = unsafe extern "C" fn() -> c_int;
}

/// A dynamically-linkable SQLite library.
pub struct Library {
    lib: libloading::Library,
    option_get: sqlite::CompileOptionGet,
    source_id: sqlite::SourceId,
    version_number: sqlite::VersionNumber,
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

        let option_get =
            unsafe { *lib.get::<sqlite::CompileOptionGet>(b"sqlite3_compileoption_get\0")? };

        let source_id = unsafe { *lib.get::<sqlite::SourceId>(b"sqlite3_sourceid\0")? };

        let version_number =
            unsafe { *lib.get::<sqlite::VersionNumber>(b"sqlite3_libversion_number\0")? };

        Ok(Self {
            lib,
            option_get,
            source_id,
            version_number,
        })
    }

    /// Unload the dynamic library.
    pub fn close(self) -> Result<(), Error> {
        self.lib.close()
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

    pub fn version(&self) -> Version {
        let num = unsafe { (self.version_number)() };
        Version::from_number(num)
    }

    pub fn source_id(&self) -> &'static str {
        let source_id = unsafe { (self.source_id)() };
        let source_id = unsafe { CStr::from_ptr(source_id) };
        unsafe { core::str::from_utf8_unchecked(source_id.to_bytes()) }
    }

    #[cfg(feature = "alloc")]
    #[cfg_attr(docsrs, doc(cfg(any(feature = "std", feature = "alloc"))))]
    fn directives(&self) -> Result<DirectiveMap, ParseDirectiveError> {
        let mut directives = DirectiveMap::new();

        let mut i: c_int = 0;
        loop {
            let directive = unsafe { (self.option_get)(i) };
            if directive.is_null() {
                break;
            }

            let directive = unsafe { CStr::from_ptr(directive) };
            let directive = unsafe { core::str::from_utf8_unchecked(directive.to_bytes()) };

            i += 1;

            match directive.parse() {
                Ok(directive) => directives.insert(directive),
                Err(ParseDirectiveError::UnknownKey) => continue,
                Err(err) => return Err(err),
            };
        }

        Ok(directives)
    }
}

#[cfg(feature = "alloc")]
impl Probe for Library {
    type Error = ParseDirectiveError;

    fn probe(&self) -> Result<crate::info::Library, Self::Error> {
        let version = self.version();
        let directives = self.directives()?;

        let version = Override::check(version, self.source_id());

        Ok(crate::info::Library::new(version, directives))
    }
}
