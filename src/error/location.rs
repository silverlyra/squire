#[cfg(sqlite_has_error_offset)]
use core::ffi::c_int;

/// The offset within a SQL source input of an [`Error`](crate::Error).
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
#[cfg_attr(
    all(nightly, feature = "lang-rustc-scalar-valid-range"),
    rustc_layout_scalar_valid_range_start(0)
)]
pub struct ErrorLocation(i32);

impl ErrorLocation {
    #[cfg(sqlite_has_error_offset)]
    #[allow(clippy::unnecessary_cast)]
    pub(super) const fn new(location: c_int) -> Option<Self> {
        if location >= 0 {
            #[cfg(all(nightly, feature = "lang-rustc-scalar-valid-range"))]
            {
                Some(unsafe { Self(location as i32) })
            }
            #[cfg(not(all(nightly, feature = "lang-rustc-scalar-valid-range")))]
            {
                Some(Self(location as i32))
            }
        } else {
            None
        }
    }

    pub const fn offset(&self) -> usize {
        self.0 as usize
    }

    pub fn prefix<'a>(&self, sql: &'a str) -> &'a str {
        &sql[..self.offset()]
    }
}
