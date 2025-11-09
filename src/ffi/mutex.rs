use core::{marker::PhantomData, ptr::NonNull};

use sqlite::{
    SQLITE_MUTEX_FAST, SQLITE_MUTEX_RECURSIVE, SQLITE_MUTEX_STATIC_APP1, SQLITE_MUTEX_STATIC_APP2,
    SQLITE_MUTEX_STATIC_APP3, SQLITE_MUTEX_STATIC_LRU, SQLITE_MUTEX_STATIC_MAIN,
    SQLITE_MUTEX_STATIC_MEM, SQLITE_MUTEX_STATIC_OPEN, SQLITE_MUTEX_STATIC_PMEM,
    SQLITE_MUTEX_STATIC_PRNG, SQLITE_MUTEX_STATIC_VFS1, SQLITE_MUTEX_STATIC_VFS2,
    SQLITE_MUTEX_STATIC_VFS3, sqlite3, sqlite3_db_mutex, sqlite3_mutex, sqlite3_mutex_alloc,
    sqlite3_mutex_enter, sqlite3_mutex_free, sqlite3_mutex_leave, sqlite3_mutex_try,
};

use super::call::call;
use crate::error::Result;

#[derive(PartialEq, Eq, Debug)]
#[repr(transparent)]
pub struct Mutex {
    inner: MutexInner,
    _lifetime: PhantomData<sqlite3_mutex>,
}

impl Mutex {
    pub fn new(recursive: bool) -> Option<Self> {
        let mode = if recursive {
            SQLITE_MUTEX_RECURSIVE
        } else {
            SQLITE_MUTEX_FAST
        };

        MutexInner::new(mode).map(|inner| Self {
            inner,
            _lifetime: PhantomData,
        })
    }

    pub fn close(self) {
        // Let `Drop` free the mutex
    }

    pub fn as_ref(&self) -> MutexRef<'_> {
        MutexRef {
            inner: self.inner,
            _lifetime: PhantomData,
        }
    }

    pub fn lock(&self) -> MutexGuard<'_> {
        self.inner.lock();
        MutexGuard::new(self.inner)
    }

    pub fn try_lock(&self) -> Option<MutexGuard<'_>> {
        match self.inner.try_lock() {
            Ok(_) => Some(MutexGuard::new(self.inner)),
            Err(_) => None,
        }
    }
}

impl Drop for Mutex {
    fn drop(&mut self) {
        unsafe { self.inner.close() };
    }
}

#[derive(PartialEq, Eq, Debug)]
#[repr(transparent)]
pub struct MutexRef<'a> {
    inner: MutexInner,
    _lifetime: PhantomData<&'a sqlite3_mutex>,
}

impl<'a> MutexRef<'a> {
    pub(super) fn from_connection(connection: *mut sqlite3) -> Option<Self> {
        let ptr = unsafe { sqlite3_db_mutex(connection) };
        Self::new(ptr)
    }

    const fn new(mutex: *mut sqlite3_mutex) -> Option<Self> {
        match MutexInner::from_ptr(mutex) {
            Some(inner) => Some(Self {
                inner,
                _lifetime: PhantomData,
            }),
            None => None,
        }
    }

    pub fn lock(&self) -> MutexGuard<'_> {
        self.inner.lock();
        MutexGuard::new(self.inner)
    }

    pub fn try_lock(&self) -> Option<MutexGuard<'_>> {
        match self.inner.try_lock() {
            Ok(_) => Some(MutexGuard::new(self.inner)),
            Err(_) => None,
        }
    }
}

impl MutexRef<'static> {
    pub fn global(mutex: StaticMutex) -> Option<Self> {
        MutexInner::global(mutex).map(|inner| Self {
                inner,
                _lifetime: PhantomData,
            })
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct MutexGuard<'a> {
    inner: MutexInner,
    _lifetime: PhantomData<MutexRef<'a>>,
}

impl<'a> MutexGuard<'a> {
    fn new(inner: MutexInner) -> Self {
        Self {
            inner,
            _lifetime: PhantomData,
        }
    }

    pub fn unlock(self) {
        // Let `Drop` release the mutex
    }
}

impl<'a> Drop for MutexGuard<'a> {
    fn drop(&mut self) {
        self.inner.unlock();
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
#[repr(transparent)]
struct MutexInner(NonNull<sqlite3_mutex>);

impl MutexInner {
    const fn from_ptr(mutex: *mut sqlite3_mutex) -> Option<Self> {
        match NonNull::new(mutex) {
            Some(ptr) => Some(MutexInner(ptr)),
            None => None,
        }
    }

    fn new(mode: i32) -> Option<Self> {
        let ptr = unsafe { sqlite3_mutex_alloc(mode) };
        Self::from_ptr(ptr)
    }

    fn global(mutex: StaticMutex) -> Option<Self> {
        let ptr = unsafe { sqlite3_mutex_alloc(mutex.value()) };
        Self::from_ptr(ptr)
    }

    unsafe fn close(&mut self) {
        unsafe { sqlite3_mutex_free(self.as_ptr()) };
        self.0 = NonNull::dangling();
    }

    #[inline]
    fn as_ptr(&self) -> *mut sqlite3_mutex {
        self.0.as_ptr()
    }

    #[inline]
    fn lock(&self) {
        unsafe { sqlite3_mutex_enter(self.as_ptr()) }
    }

    #[inline]
    fn try_lock(&self) -> Result<(), ()> {
        call! { sqlite3_mutex_try(self.as_ptr()) }
    }

    #[inline]
    fn unlock(&self) {
        unsafe { sqlite3_mutex_leave(self.as_ptr()) }
    }
}

/// Static SQLite mutex identifiers.
///
/// These mutexes are statically allocated and managed by SQLite.
#[derive(PartialEq, Eq, Clone, Copy, Default, Debug)]
#[repr(i32)]
pub enum StaticMutex {
    #[default]
    #[doc(alias = "SQLITE_MUTEX_STATIC_MAIN")]
    Main = SQLITE_MUTEX_STATIC_MAIN,

    #[doc(alias = "SQLITE_MUTEX_STATIC_MEM")]
    Mem = SQLITE_MUTEX_STATIC_MEM,

    #[doc(alias = "SQLITE_MUTEX_STATIC_OPEN")]
    Open = SQLITE_MUTEX_STATIC_OPEN,

    #[doc(alias = "SQLITE_MUTEX_STATIC_PRNG")]
    Prng = SQLITE_MUTEX_STATIC_PRNG,

    #[doc(alias = "SQLITE_MUTEX_STATIC_LRU")]
    Lru = SQLITE_MUTEX_STATIC_LRU,

    #[doc(alias = "SQLITE_MUTEX_STATIC_PMEM")]
    Pmem = SQLITE_MUTEX_STATIC_PMEM,

    #[doc(alias = "SQLITE_MUTEX_STATIC_APP1")]
    App1 = SQLITE_MUTEX_STATIC_APP1,

    #[doc(alias = "SQLITE_MUTEX_STATIC_APP2")]
    App2 = SQLITE_MUTEX_STATIC_APP2,

    #[doc(alias = "SQLITE_MUTEX_STATIC_APP3")]
    App3 = SQLITE_MUTEX_STATIC_APP3,

    #[doc(alias = "SQLITE_MUTEX_STATIC_VFS1")]
    Vfs1 = SQLITE_MUTEX_STATIC_VFS1,

    #[doc(alias = "SQLITE_MUTEX_STATIC_VFS2")]
    Vfs2 = SQLITE_MUTEX_STATIC_VFS2,

    #[doc(alias = "SQLITE_MUTEX_STATIC_VFS3")]
    Vfs3 = SQLITE_MUTEX_STATIC_VFS3,
}

impl StaticMutex {
    #[inline]
    pub(crate) const fn value(self) -> i32 {
        self as i32
    }
}
