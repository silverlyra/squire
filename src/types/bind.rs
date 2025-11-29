use core::{ffi::c_int, num::NonZero};

use crate::error::{Error, ErrorReason, ParameterError, Result};

/// A SQLite [prepared statement](crate::Statement) parameter index, used when
/// [binding](crate::Bind) values to a statement.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
#[repr(transparent)]
pub struct BindIndex(NonZero<c_int>);

impl BindIndex {
    /// The first numbered binding parameter index (`1`).
    ///
    /// ```rust
    /// # use squire::BindIndex;
    /// let first = BindIndex::INITIAL;
    /// assert_eq!(usize::from(first), 1);
    /// ```
    pub const INITIAL: Self = Self(NonZero::new(1).unwrap());

    pub const fn new(value: c_int) -> Option<Self> {
        match NonZero::new(value) {
            Some(index) if value > 0 => Some(Self(index)),
            _ => None,
        }
    }

    /// Initialize a [`BindIndex`] without validating the `value`.
    ///
    /// # Safety
    ///
    /// Callers are responsible for ensuring `value` is non-zero.
    pub const unsafe fn new_unchecked(value: c_int) -> Self {
        Self(unsafe { NonZero::new_unchecked(value) })
    }

    /// Access the underlying parameter index value as a C [`int`](c_int).
    #[inline]
    pub const fn value(&self) -> c_int {
        self.0.get()
    }

    pub const fn iter(&self) -> BindIndexes {
        BindIndexes::new(*self)
    }

    /// Add `1` to this [`BindIndex`].
    ///
    /// ```rust
    /// # use squire::BindIndex;
    /// let first = BindIndex::INITIAL;
    /// assert_eq!(usize::from(first), 1);
    ///
    /// let second = first.next();
    /// assert_eq!(usize::from(second), 2);
    /// ```
    pub const fn next(&self) -> Self {
        unsafe { Self::new_unchecked(self.value() + 1) }
    }
}

impl IntoIterator for BindIndex {
    type Item = BindIndex;
    type IntoIter = BindIndexes;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl From<BindIndex> for i32 {
    fn from(index: BindIndex) -> Self {
        index.value() as Self
    }
}

impl From<BindIndex> for u32 {
    fn from(index: BindIndex) -> Self {
        index.value() as Self
    }
}

impl From<BindIndex> for i64 {
    fn from(index: BindIndex) -> Self {
        index.value() as Self
    }
}

impl From<BindIndex> for u64 {
    fn from(index: BindIndex) -> Self {
        index.value() as Self
    }
}

impl From<BindIndex> for isize {
    fn from(index: BindIndex) -> Self {
        index.value() as Self
    }
}

impl From<BindIndex> for usize {
    fn from(index: BindIndex) -> Self {
        index.value() as Self
    }
}

impl TryFrom<i32> for BindIndex {
    type Error = Error;

    fn try_from(value: i32) -> Result<Self> {
        Self::new(value as c_int).ok_or_else(invalid_index)
    }
}

impl TryFrom<i64> for BindIndex {
    type Error = Error;

    fn try_from(value: i64) -> Result<Self> {
        Self::new(value as c_int).ok_or_else(invalid_index)
    }
}

impl TryFrom<isize> for BindIndex {
    type Error = Error;

    fn try_from(value: isize) -> Result<Self> {
        Self::new(value as c_int).ok_or_else(invalid_index)
    }
}

impl TryFrom<u32> for BindIndex {
    type Error = Error;

    fn try_from(value: u32) -> Result<Self> {
        Self::new(value as c_int).ok_or_else(invalid_index)
    }
}

impl TryFrom<u64> for BindIndex {
    type Error = Error;

    fn try_from(value: u64) -> Result<Self> {
        Self::new(value as c_int).ok_or_else(invalid_index)
    }
}

impl TryFrom<usize> for BindIndex {
    type Error = Error;

    fn try_from(value: usize) -> Result<Self> {
        Self::new(value as c_int).ok_or_else(invalid_index)
    }
}

#[cold]
fn invalid_index() -> Error {
    ErrorReason::Parameter(ParameterError::InvalidIndex).into()
}

#[cfg(all(nightly, feature = "lang-step-trait"))]
impl core::iter::Step for BindIndex {
    fn steps_between(start: &Self, end: &Self) -> (usize, Option<usize>) {
        if start.0.get() <= end.0.get() {
            let steps = (end.0.get() - start.0.get()) as usize;
            (steps, Some(steps))
        } else {
            (0, None)
        }
    }

    fn forward_checked(start: Self, count: usize) -> Option<Self> {
        let count = c_int::try_from(count).ok()?;
        let new_value = start.0.get().checked_add(count)?;
        NonZero::new(new_value).map(Self)
    }

    fn backward_checked(start: Self, count: usize) -> Option<Self> {
        let count = c_int::try_from(count).ok()?;
        let new_value = start.0.get().checked_sub(count)?;
        NonZero::new(new_value).map(Self)
    }
}

pub struct BindIndexes {
    current: BindIndex,
}

impl BindIndexes {
    const fn new(initial: BindIndex) -> Self {
        Self { current: initial }
    }
}

impl Iterator for BindIndexes {
    type Item = BindIndex;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current;
        self.current = self.current.next();
        Some(current)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (usize::MAX, None)
    }
}
