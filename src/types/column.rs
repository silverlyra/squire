use core::ffi::c_int;

use crate::error::{Error, ErrorCategory, Result};

/// A SQLite column index, used for [reading values][] out of queried rows.
///
/// [reading values]: https://sqlite.org/c3ref/column_blob.html
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
#[repr(transparent)]
pub struct ColumnIndex(c_int);

impl ColumnIndex {
    /// The first numbered column index (`0`).
    ///
    /// ```rust
    /// # use squire::ColumnIndex;
    /// let first = ColumnIndex::INITIAL;
    /// assert_eq!(usize::from(first), 0);
    /// ```
    pub const INITIAL: Self = Self(0);

    pub const fn new(value: c_int) -> Self {
        Self(value)
    }

    /// Access the underlying SQLite column index as a C [`int`](c_int).
    #[inline]
    pub const fn value(&self) -> c_int {
        self.0
    }

    /// Add `1` to this [column index](Self).
    ///
    /// ```rust
    /// # use squire::ColumnIndex;
    /// let first = ColumnIndex::INITIAL;
    /// assert_eq!(usize::from(first), 0);
    ///
    /// let second = first.next();
    /// assert_eq!(usize::from(second), 1);
    /// ```
    pub const fn next(&self) -> Self {
        Self(self.value() + 1)
    }
}

impl From<ColumnIndex> for i32 {
    fn from(index: ColumnIndex) -> Self {
        index.value() as Self
    }
}

impl From<ColumnIndex> for u32 {
    fn from(index: ColumnIndex) -> Self {
        index.value() as Self
    }
}

impl From<ColumnIndex> for i64 {
    fn from(index: ColumnIndex) -> Self {
        index.value() as Self
    }
}

impl From<ColumnIndex> for u64 {
    fn from(index: ColumnIndex) -> Self {
        index.value() as Self
    }
}

impl From<ColumnIndex> for isize {
    fn from(index: ColumnIndex) -> Self {
        index.value() as Self
    }
}

impl From<ColumnIndex> for usize {
    fn from(index: ColumnIndex) -> Self {
        index.value() as Self
    }
}

impl TryFrom<i32> for ColumnIndex {
    type Error = Error;

    fn try_from(value: i32) -> Result<Self> {
        if value >= 0 {
            Ok(Self::new(value as c_int))
        } else {
            Err(invalid_index())
        }
    }
}

impl TryFrom<i64> for ColumnIndex {
    type Error = Error;

    fn try_from(value: i64) -> Result<Self> {
        match c_int::try_from(value) {
            Ok(value) if value >= 0 => Ok(Self::new(value)),
            _ => Err(invalid_index()),
        }
    }
}

impl TryFrom<isize> for ColumnIndex {
    type Error = Error;

    fn try_from(value: isize) -> Result<Self> {
        match c_int::try_from(value) {
            Ok(value) if value >= 0 => Ok(Self::new(value)),
            _ => Err(invalid_index()),
        }
    }
}

impl TryFrom<u32> for ColumnIndex {
    type Error = Error;

    fn try_from(value: u32) -> Result<Self> {
        match c_int::try_from(value) {
            Ok(value) => Ok(Self::new(value)),
            Err(_) => Err(invalid_index()),
        }
    }
}

impl TryFrom<u64> for ColumnIndex {
    type Error = Error;

    fn try_from(value: u64) -> Result<Self> {
        match c_int::try_from(value) {
            Ok(value) => Ok(Self::new(value)),
            Err(_) => Err(invalid_index()),
        }
    }
}

impl TryFrom<usize> for ColumnIndex {
    type Error = Error;

    fn try_from(value: usize) -> Result<Self> {
        match c_int::try_from(value) {
            Ok(value) => Ok(Self::new(value)),
            Err(_) => Err(invalid_index()),
        }
    }
}

#[cold]
fn invalid_index() -> Error {
    ErrorCategory::Range.into()
}

#[cfg(feature = "lang-step-trait")]
impl core::iter::Step for ColumnIndex {
    fn steps_between(start: &Self, end: &Self) -> (usize, Option<usize>) {
        if start.0 <= end.0 {
            let steps = (end.0 - start.0) as usize;
            (steps, Some(steps))
        } else {
            (0, None)
        }
    }

    fn forward_checked(start: Self, count: usize) -> Option<Self> {
        let count = c_int::try_from(count).ok()?;
        let new_value = start.0.checked_add(count)?;
        Some(Self(new_value))
    }

    fn backward_checked(start: Self, count: usize) -> Option<Self> {
        let count = c_int::try_from(count).ok()?;
        let new_value = start.0.checked_sub(count)?;
        Some(Self(new_value))
    }
}
