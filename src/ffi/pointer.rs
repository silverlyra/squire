use core::{
    ffi::CStr,
    fmt,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

pub struct Pointer<'a, T: Pointee + ?Sized> {
    target: *const T,
    _value: PhantomData<fn() -> &'a T>,
}

impl<'a, T: Pointee + ?Sized> Pointer<'a, T> {
    /// Create a [`Pointer`] from a raw pointer.
    ///
    /// # Safety
    ///
    /// `target` must be [convertible to `&'a T`][safety].
    ///
    /// [safety]: https://doc.rust-lang.org/std/ptr/index.html#pointer-to-reference-conversion
    pub const unsafe fn new(target: *const T) -> Option<Self> {
        if !target.is_null() {
            Some(Self {
                target,
                _value: PhantomData,
            })
        } else {
            None
        }
    }

    pub const fn of(value: &'a T) -> Self {
        Self {
            target: value,
            _value: PhantomData,
        }
    }
}

impl<T: Pointee + ?Sized> Deref for Pointer<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        // SAFETY: guaranteed by `new` / `of` constructors
        unsafe { &*self.target }
    }
}

impl<T: Pointee + ?Sized> AsRef<T> for Pointer<'_, T> {
    fn as_ref(&self) -> &T {
        self
    }
}

impl<T: Pointee + fmt::Debug + ?Sized> fmt::Debug for Pointer<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Pointer")
            .field("type", &T::TYPE)
            .field("ptr", &self.target)
            .finish()
    }
}

pub struct PointerMut<'a, T: Pointee + ?Sized> {
    target: *mut T,
    _value: PhantomData<fn() -> &'a mut T>,
}

impl<'a, T: Pointee + ?Sized> PointerMut<'a, T> {
    /// Create a [`PointerMut`] from a raw pointer.
    ///
    /// # Safety
    ///
    /// `target` must be [convertible to a `&'a mut T`][safety].
    ///
    /// [safety]: https://doc.rust-lang.org/std/ptr/index.html#pointer-to-reference-conversion
    pub const unsafe fn new(target: *mut T) -> Option<Self> {
        if !target.is_null() {
            Some(Self {
                target,
                _value: PhantomData,
            })
        } else {
            None
        }
    }

    pub const fn of(value: &'a mut T) -> Self {
        Self {
            target: value,
            _value: PhantomData,
        }
    }
}

impl<T: Pointee + ?Sized> Deref for PointerMut<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        // SAFETY: guaranteed by `new` / `of` constructors
        unsafe { &*self.target }
    }
}

impl<T: Pointee + ?Sized> DerefMut for PointerMut<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY: guaranteed by `new` / `of` constructors
        unsafe { &mut *self.target }
    }
}

impl<T: Pointee + ?Sized> AsRef<T> for PointerMut<'_, T> {
    fn as_ref(&self) -> &T {
        self
    }
}

impl<T: Pointee + fmt::Debug + ?Sized> fmt::Debug for PointerMut<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PointerMut")
            .field("type", &T::TYPE)
            .field("ptr", &self.target)
            .finish()
    }
}

/// A type that can use the SQLite [pointer passing interface][].
///
/// [pointer passing interface]: https://sqlite.org/bindptr.html
pub trait Pointee {
    const TYPE: &'static CStr;
}
