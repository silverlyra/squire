use core::ffi::{c_char, c_int};

unsafe extern "C" {
    /// The [version][] of the SQLite library, as a string.
    ///
    /// [version]: https://sqlite.org/c3ref/libversion.html
    pub fn sqlite3_libversion() -> *const c_char;

    /// The [version][] of the SQLite library, as a comparable integer.
    ///
    /// [version]: https://sqlite.org/c3ref/libversion.html
    pub fn sqlite3_libversion_number() -> c_int;

    /// The full [build version][] of the SQLite library.
    ///
    /// [build version]: https://sqlite.org/c3ref/libversion.html
    pub fn sqlite3_sourceid() -> *const c_char;
}
