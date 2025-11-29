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
    /// [version]: https://sqlite.org/c3ref/libversion.html
    pub fn sqlite3_sourceid() -> *const c_char;

    /// Check if a SQLite [compile-time option][] was used.
    ///
    /// [compile-time option]: https://sqlite.org/c3ref/compileoption_get.html
    pub fn sqlite3_compileoption_used(zOptName: *const c_char) -> c_int;

    /// Enumerate SQLite [compile-time options][] was used.
    ///
    /// [compile-time options]: https://sqlite.org/c3ref/compileoption_get.html
    pub fn sqlite3_compileoption_get(n: c_int) -> *const c_char;

    /// Check the compiled [thread-safety][] mode of SQLite.
    ///
    /// [thread-safety]: https://sqlite.org/c3ref/threadsafe.html
    pub fn sqlite3_threadsafe() -> c_int;
}
