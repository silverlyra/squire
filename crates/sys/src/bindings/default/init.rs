use core::ffi::c_int;

unsafe extern "C" {
    /// [Initialize][initialize] the SQLite library.
    ///
    /// [initialize]: https://sqlite.org/c3ref/initialize.html
    pub fn sqlite3_initialize() -> c_int;

    /// Deallocate any resources allocated by [`sqlite3_initialize`].
    pub fn sqlite3_shutdown() -> c_int;
}
