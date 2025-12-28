use core::{borrow::Borrow, fmt, marker::PhantomData, mem, ptr};

use sqlite::{sqlite3_value, sqlite3_value_dup, sqlite3_value_free};

use super::fetch::Fetch;

/// A thin wrapper around an owned [`sqlite3_value`].
#[cfg_attr(docsrs, doc(cfg(any(feature = "functions", feature = "value"))))]
#[repr(transparent)]
pub struct Value {
    handle: ptr::NonNull<sqlite3_value>,
}

#[cfg(any(feature = "multi-thread", feature = "serialized"))]
#[cfg_attr(
    docsrs,
    doc(cfg(any(feature = "multi-thread", feature = "serialized")))
)]
unsafe impl Send for Value {}

#[cfg(feature = "serialized")]
#[cfg_attr(docsrs, doc(cfg(feature = "serialized")))]
unsafe impl Sync for Value {}

impl Value {
    /// Wrap a [`sqlite3_value`] dynamic value object.
    #[inline]
    #[must_use]
    pub(crate) const fn new(handle: *mut sqlite3_value) -> Option<Self> {
        match ptr::NonNull::new(handle) {
            Some(handle) => Some(Self { handle }),
            None => None,
        }
    }

    pub(crate) fn duplicate(handle: *const sqlite3_value) -> Option<Self> {
        let handle = unsafe { sqlite3_value_dup(handle) };
        Self::new(handle)
    }

    /// [Unpack][fetch] a typed value from this [`Value`].
    ///
    /// [fetch]: https://sqlite.org/c3ref/value_blob.html
    pub fn fetch<'r, T: Fetch<'r>>(&'r self) -> T {
        unsafe { T::fetch_value(self.reference()) }
    }

    /// Deallocate the value with [`sqlite3_value_free`].
    #[inline]
    pub fn free(mut self) {
        unsafe { self.destroy() }
    }

    #[inline]
    #[must_use]
    const fn reference<'c>(&self) -> &ValueRef<'c> {
        // SAFETY: Both Value and ValueRef are `repr(transparent)` around
        // `sqlite3_value*`.
        unsafe { mem::transmute(self) }
    }

    #[inline]
    pub(crate) unsafe fn destroy(&mut self) {
        unsafe { sqlite3_value_free(self.as_ptr()) }
    }

    /// Access the raw [`sqlite3_value`] pointer.
    #[inline]
    pub const fn as_ptr(&self) -> *mut sqlite3_value {
        self.handle.as_ptr()
    }
}

impl<'a> AsRef<ValueRef<'a>> for Value {
    fn as_ref(&self) -> &ValueRef<'a> {
        // SAFETY: Both are `repr(transparent)` around `sqlite3_value*`.
        unsafe { mem::transmute(self) }
    }
}

impl<'a> Borrow<ValueRef<'a>> for Value {
    fn borrow(&self) -> &ValueRef<'a> {
        // SAFETY: Both are `repr(transparent)` around `sqlite3_value*`.
        unsafe { mem::transmute(self) }
    }
}

impl<'a> AsRef<OpaqueValueRef<'a>> for Value {
    fn as_ref(&self) -> &OpaqueValueRef<'a> {
        // SAFETY: Both are `repr(transparent)` around `sqlite3_value*`.
        unsafe { mem::transmute(self) }
    }
}

impl<'a> Borrow<OpaqueValueRef<'a>> for Value {
    fn borrow(&self) -> &OpaqueValueRef<'a> {
        // SAFETY: Both are `repr(transparent)` around `sqlite3_value*`.
        unsafe { mem::transmute(self) }
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Value({:p})", self.handle)
    }
}

/// A thin wrapper around a [`sqlite3_value`] reference.
///
/// `ValueRef` is used only to represent _protected_ `sqlite3_value` objects,
/// such as those passed to user-defined functions or those returned by
/// [`sqlite3_value_dup`]. [`OpaqueValueRef`] is instead used to represent the
/// _unprotected_ values returned from `sqlite3_column_*` functions.
#[cfg_attr(docsrs, doc(cfg(any(feature = "functions", feature = "value"))))]
#[repr(transparent)]
pub struct ValueRef<'a> {
    handle: ptr::NonNull<sqlite3_value>,
    _value: PhantomData<fn() -> &'a sqlite3_value>,
}

#[cfg(any(feature = "multi-thread", feature = "serialized"))]
#[cfg_attr(
    docsrs,
    doc(cfg(any(feature = "multi-thread", feature = "serialized")))
)]
unsafe impl<'c> Send for ValueRef<'c> {}

#[cfg(feature = "serialized")]
#[cfg_attr(docsrs, doc(cfg(feature = "serialized")))]
unsafe impl<'c> Sync for ValueRef<'c> {}

impl<'a> ValueRef<'a> {
    /// Wrap a [`sqlite3_value`] dynamic value object.
    #[inline]
    #[must_use]
    pub const fn new(handle: *mut sqlite3_value) -> Option<Self> {
        match ptr::NonNull::new(handle) {
            Some(handle) => Some(Self {
                handle,
                _value: PhantomData,
            }),
            None => None,
        }
    }

    /// [Unpack][fetch] a typed value from this [`Value`].
    ///
    /// [fetch]: https://sqlite.org/c3ref/value_blob.html
    ///
    /// # Safety
    ///
    /// Callers are responsible for managing the `ffi::Value` lifecycle.
    pub unsafe fn fetch<'r, T: Fetch<'r>>(&'r self) -> T {
        unsafe { T::fetch_value(self) }
    }

    /// Access the raw [`sqlite3_value`] pointer.
    #[inline]
    pub const fn as_ptr(&self) -> *mut sqlite3_value {
        self.handle.as_ptr()
    }
}

impl<'a> ToOwned for ValueRef<'a> {
    type Owned = Value;

    fn to_owned(&self) -> Self::Owned {
        Value::duplicate(self.as_ptr()).expect("duplicated sqlite3_value")
    }
}

impl fmt::Debug for ValueRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ValueRef({:p})", self.handle)
    }
}

/// A thin wrapper around an _unprotected_ [`sqlite3_value`] reference.
///
/// See [`ValueRef`] for a _protected_ `sqlite3_value` reference.
#[cfg_attr(docsrs, doc(cfg(any(feature = "functions", feature = "value"))))]
#[repr(transparent)]
pub struct OpaqueValueRef<'a> {
    handle: ptr::NonNull<sqlite3_value>,
    _value: PhantomData<fn() -> &'a sqlite3_value>,
}

impl<'a> OpaqueValueRef<'a> {
    /// Wrap a [`sqlite3_value`] dynamic value object.
    #[inline]
    #[must_use]
    pub(crate) const fn new(handle: *mut sqlite3_value) -> Option<Self> {
        match ptr::NonNull::new(handle) {
            Some(handle) => Some(Self {
                handle,
                _value: PhantomData,
            }),
            None => None,
        }
    }

    /// Access the raw [`sqlite3_value`] pointer.
    #[inline]
    pub const fn as_ptr(&self) -> *mut sqlite3_value {
        self.handle.as_ptr()
    }
}

impl<'a> ToOwned for OpaqueValueRef<'a> {
    type Owned = Value;

    fn to_owned(&self) -> Self::Owned {
        Value::duplicate(self.as_ptr()).expect("duplicated sqlite3_value")
    }
}

impl fmt::Debug for OpaqueValueRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "OpaqueValueRef({:p})", self.handle)
    }
}
