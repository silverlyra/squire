/// A destructor / memory `free` function for a value passed to SQLite as a
/// [bound parameter][bind] or a [function result][].
///
/// A destructor can either be a `func`tion, or one of the [special values][]
/// `SQLITE_STATIC` (`0`) or `SQLITE_TRANSIENT` (`-1`).
///
/// [bind]: https://sqlite.org/c3ref/bind_blob.html
/// [function result]: https://sqlite.org/c3ref/result_blob.html
/// [special values]: https://sqlite.org/c3ref/c_static.html
#[derive(Copy, Clone)]
#[repr(C)]
pub union sqlite3_destructor_type {
    pub func: unsafe extern "C" fn(context: *mut ::std::os::raw::c_void),
    pub sentinel: isize,
}

unsafe impl Send for sqlite3_destructor_type {}
unsafe impl Sync for sqlite3_destructor_type {}

impl sqlite3_destructor_type {
    /// Provide a custom [destructor](sqlite3_destructor_type) to SQLite.
    pub const fn new(func: unsafe extern "C" fn(*mut ::std::os::raw::c_void)) -> Self {
        sqlite3_destructor_type { func }
    }

    /// Construct `SQLITE_STATIC` or `SQLITE_TRANSIENT`.
    pub(crate) const fn from_sentinel(sentinel: isize) -> Self {
        sqlite3_destructor_type { sentinel }
    }
}

impl ::core::fmt::Debug for sqlite3_destructor_type {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        // Safety: Both fields have the same size, so reading sentinel is always valid
        let sentinel = unsafe { self.sentinel };
        match sentinel {
            0 => write!(f, "sqlite3_destructor_type::SQLITE_STATIC"),
            -1 => write!(f, "sqlite3_destructor_type::SQLITE_TRANSIENT"),
            _ => write!(
                f,
                "sqlite3_destructor_type::func({:p})",
                sentinel as *const ()
            ),
        }
    }
}
