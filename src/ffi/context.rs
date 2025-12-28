use core::{fmt, marker::PhantomData, ptr};

use sqlite::{sqlite3, sqlite3_context, sqlite3_context_db_handle};

use super::connection::Connected;

/// A thin wrapper around a [`sqlite3_context`] function context.
#[cfg_attr(docsrs, doc(cfg(feature = "functions")))]
#[repr(transparent)]
pub struct ContextRef<'a> {
    handle: ptr::NonNull<sqlite3_context>,
    _value: PhantomData<fn() -> &'a sqlite3_context>,
}

#[cfg(any(feature = "multi-thread", feature = "serialized"))]
#[cfg_attr(
    docsrs,
    doc(cfg(any(feature = "multi-thread", feature = "serialized")))
)]
unsafe impl<'c> Send for ContextRef<'c> {}

#[cfg(feature = "serialized")]
#[cfg_attr(docsrs, doc(cfg(feature = "serialized")))]
unsafe impl<'c> Sync for ContextRef<'c> {}

impl<'a> ContextRef<'a> {
    /// Wrap a [`sqlite3_context`].
    #[inline]
    #[must_use]
    pub const fn new(handle: *mut sqlite3_context) -> Option<Self> {
        match ptr::NonNull::new(handle) {
            Some(handle) => Some(Self {
                handle,
                _value: PhantomData,
            }),
            None => None,
        }
    }

    /// Access the raw [`sqlite3_context`] pointer.
    #[inline]
    pub const fn as_ptr(&self) -> *mut sqlite3_context {
        self.handle.as_ptr()
    }

    #[inline]
    pub(crate) unsafe fn connection_ptr(&self) -> *mut sqlite3 {
        unsafe { sqlite3_context_db_handle(self.as_ptr()) }
    }
}

impl Connected for ContextRef<'_> {
    fn as_connection_ptr(&self) -> *mut sqlite3 {
        unsafe { self.connection_ptr() }
    }
}

impl Connected for &ContextRef<'_> {
    fn as_connection_ptr(&self) -> *mut sqlite3 {
        unsafe { self.connection_ptr() }
    }
}

impl fmt::Debug for ContextRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ContextRef({:p})", self.handle)
    }
}
