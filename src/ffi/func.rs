use core::{
    ffi::{c_char, c_int},
    fmt,
    marker::PhantomData,
    mem, ptr, slice,
};

use sqlite::{
    sqlite3, sqlite3_context, sqlite3_context_db_handle, sqlite3_result_error,
    sqlite3_result_error_code, sqlite3_user_data, sqlite3_value,
};

use super::{bind::Bind, connection::Connected};
use crate::ffi::ValueRef;

#[cfg(not(feature = "multi-thread"))]
pub trait Function: 'static {
    fn call<'a>(&self, context: &'a mut ContextRef<'a>, arguments: &'a [ValueRef<'a>]);
}

#[cfg(feature = "multi-thread")]
pub trait Function: Send + 'static {
    fn call<'a>(&self, context: &'a mut ContextRef<'a>, arguments: &'a [ValueRef<'a>]);
}

pub(super) unsafe extern "C" fn call<F: Function>(
    context: *mut sqlite3_context,
    argc: c_int,
    argv: *mut *mut sqlite3_value,
) {
    let function = unsafe { sqlite3_user_data(context).cast::<F>() };
    debug_assert!(function.is_aligned());
    debug_assert!(!function.is_null());

    let mut context = ContextRef::new(context).expect("context");

    let arguments = unsafe { slice::from_raw_parts(argv, argc as usize) };
    let arguments: &[ValueRef<'_>] = unsafe { mem::transmute(arguments) };

    unsafe {
        (&*function).call(&mut context, arguments);
    }
}

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

    /// Set the return value of the function.
    ///
    /// # Safety
    ///
    /// See [`Bind::bind_return`].
    pub unsafe fn set_result<'b, B>(&mut self, value: B)
    where
        B: Bind<'b>,
        'b: 'a,
    {
        unsafe { value.bind_return(self) }
    }

    pub fn set_error(&mut self, message: &str) {
        unsafe {
            sqlite3_result_error(
                self.as_ptr(),
                message.as_ptr().cast::<c_char>(),
                message.len() as c_int,
            )
        };
    }

    pub fn set_error_code(&mut self, code: i32) {
        unsafe { sqlite3_result_error_code(self.as_ptr(), code) };
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
