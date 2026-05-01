use core::ffi::{c_char, c_int};

use super::connection::sqlite3;

unsafe extern "C" {
    /// Set a [library-wide configuration option][].
    ///
    /// [library-wide configuration option]: https://sqlite.org/c3ref/config.html
    pub fn sqlite3_config(option: c_int, ...) -> c_int;

    /// Set a [connection configuration option][].
    ///
    /// [connection configuration option]: https://sqlite.org/c3ref/db_config.html
    pub fn sqlite3_db_config(db: *mut sqlite3, option: c_int, ...) -> c_int;

    /// Check if a SQLite [compile-time option][] was used.
    ///
    /// [compile-time option]: https://sqlite.org/c3ref/compileoption_get.html
    pub fn sqlite3_compileoption_used(option_name: *const c_char) -> c_int;

    /// Enumerate the [compile-time options][] SQLite was built with.
    ///
    /// [compile-time options]: https://sqlite.org/c3ref/compileoption_get.html
    pub fn sqlite3_compileoption_get(n: c_int) -> *const c_char;

    /// Check the compiled [thread-safety][] mode of SQLite.
    ///
    /// [thread-safety]: https://sqlite.org/c3ref/threadsafe.html
    pub fn sqlite3_threadsafe() -> c_int;
}
