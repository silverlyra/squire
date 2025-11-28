use core::{fmt, ops::Deref};

#[cfg(not(feature = "multi-thread"))]
use std::rc::Rc;
#[cfg(feature = "multi-thread")]
use std::sync::Arc;

/// An [error](core::error::Error) from a crate that Squire integrates with.
#[derive(Clone, Debug)]
pub enum IntegrationError {
    /// An error from the [`jiff`][] crate.
    ///
    /// [`jiff`]: https://crates.io/crates/jiff
    #[cfg(feature = "jiff")]
    #[cfg_attr(docsrs, doc(cfg(feature = "jiff")))]
    Jiff(jiff::Error),

    /// An error from [`serde_json`][].
    ///
    /// [`serde_json`]: https://lib.rs/serde_json
    #[cfg(all(feature = "serde", feature = "json"))]
    #[cfg_attr(docsrs, doc(cfg(all(feature = "serde", feature = "json"))))]
    Json(ErrorContainer<squire_serde::json::Error>),

    /// An error from [`serde_sqlite_jsonb`][].
    ///
    /// [`serde_sqlite_jsonb`]: https://lib.rs/serde_sqlite_jsonb
    #[cfg(all(feature = "serde", feature = "jsonb"))]
    #[cfg_attr(docsrs, doc(cfg(all(feature = "serde", feature = "jsonb"))))]
    Jsonb(ErrorContainer<squire_serde::jsonb::Error>),

    /// An error from the [`url`][] crate.
    ///
    /// [`url`]: https://lib.rs/url
    #[cfg(feature = "url")]
    #[cfg_attr(docsrs, doc(cfg(feature = "url")))]
    Url(url::ParseError),
}

#[cfg(feature = "jiff")]
#[cfg_attr(docsrs, doc(cfg(feature = "jiff")))]
impl IntegrationError {
    /// `true` if this is a [`jiff::Error`]; `false` if otherwise.
    pub fn is_jiff(&self) -> bool {
        matches!(self, Self::Jiff(_))
    }

    /// Access the [`jiff::Error`] contained in this [`IntegrationError`].
    ///
    /// Returns `None` if this is not a `Jiff` error.
    pub fn as_jiff(&self) -> Option<&jiff::Error> {
        match self {
            Self::Jiff(error) => Some(error),
            _ => None,
        }
    }
}

#[cfg(feature = "jiff")]
#[cfg_attr(docsrs, doc(cfg(feature = "jiff")))]
impl From<jiff::Error> for IntegrationError {
    fn from(error: jiff::Error) -> Self {
        Self::Jiff(error)
    }
}

#[cfg(all(feature = "serde", feature = "json"))]
#[cfg_attr(docsrs, doc(cfg(all(feature = "serde", feature = "json"))))]
impl IntegrationError {
    /// `true` if this is a `serde_json::Error`; `false` if otherwise.
    pub fn is_json(&self) -> bool {
        matches!(self, Self::Json(_))
    }

    /// Access the `serde_json::Error` contained in this [`IntegrationError`].
    ///
    /// Returns `None` if this is not a `Json` error.
    pub fn as_json(&self) -> Option<&squire_serde::json::Error> {
        match self {
            Self::Json(container) => Some(container.as_ref()),
            _ => None,
        }
    }
}

#[cfg(all(feature = "serde", feature = "json"))]
#[cfg_attr(docsrs, doc(cfg(all(feature = "serde", feature = "json"))))]
impl From<squire_serde::json::Error> for IntegrationError {
    fn from(error: squire_serde::json::Error) -> Self {
        Self::Json(ErrorContainer::new(error))
    }
}

#[cfg(all(feature = "serde", feature = "jsonb"))]
#[cfg_attr(docsrs, doc(cfg(all(feature = "serde", feature = "jsonb"))))]
impl IntegrationError {
    /// `true` if this is a `serde_sqlite_jsonb::Error`; `false` if otherwise.
    pub fn is_jsonb(&self) -> bool {
        matches!(self, Self::Jsonb(_))
    }

    /// Access the `serde_sqlite_jsonb::Error` contained in this [`IntegrationError`].
    ///
    /// Returns `None` if this is not a `Jsonb` error.
    pub fn as_jsonb(&self) -> Option<&squire_serde::jsonb::Error> {
        match self {
            Self::Jsonb(container) => Some(container.as_ref()),
            _ => None,
        }
    }
}

#[cfg(all(feature = "serde", feature = "jsonb"))]
#[cfg_attr(docsrs, doc(cfg(all(feature = "serde", feature = "jsonb"))))]
impl From<squire_serde::jsonb::Error> for IntegrationError {
    fn from(error: squire_serde::jsonb::Error) -> Self {
        Self::Jsonb(ErrorContainer::new(error))
    }
}

#[cfg(feature = "url")]
#[cfg_attr(docsrs, doc(cfg(feature = "url")))]
impl IntegrationError {
    /// `true` if this is a [`url::ParseError`]; `false` if otherwise.
    pub fn is_url(&self) -> bool {
        matches!(self, Self::Url(_))
    }

    /// Access the [`url::ParseError`] contained in this [`IntegrationError`].
    ///
    /// Returns `None` if this is not a `Url` error.
    pub fn as_url(&self) -> Option<&url::ParseError> {
        match self {
            Self::Url(error) => Some(error),
            _ => None,
        }
    }
}

#[cfg(feature = "url")]
#[cfg_attr(docsrs, doc(cfg(feature = "url")))]
impl From<url::ParseError> for IntegrationError {
    fn from(error: url::ParseError) -> Self {
        Self::Url(error)
    }
}

impl fmt::Display for IntegrationError {
    fn fmt(&self, #[allow(unused_variables)] f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            #[cfg(feature = "jiff")]
            IntegrationError::Jiff(ref error) => error.fmt(f),
            #[cfg(all(feature = "serde", feature = "json"))]
            IntegrationError::Json(ErrorContainer(ref error)) => error.fmt(f),
            #[cfg(all(feature = "serde", feature = "jsonb"))]
            IntegrationError::Jsonb(ErrorContainer(ref error)) => error.fmt(f),
            #[cfg(feature = "url")]
            IntegrationError::Url(ref error) => error.fmt(f),
        }
    }
}

impl core::error::Error for IntegrationError {}

/// Wraps an [`IntegrationError`] member that is not [cloneable](Clone)
/// in [`Rc`].
#[cfg(not(feature = "multi-thread"))]
#[derive(PartialEq, Eq)]
pub struct ErrorContainer<T>(pub Rc<T>);

/// Wraps an [`IntegrationError`] member that is not [cloneable](Clone)
/// in [`Arc`].
#[cfg(feature = "multi-thread")]
#[derive(PartialEq, Eq)]
pub struct ErrorContainer<T>(pub Arc<T>);

impl<T> ErrorContainer<T> {
    #[cfg_attr(not(feature = "multi-thread"), doc = "Wrap an error in [`Rc`].")]
    #[cfg_attr(feature = "multi-thread", doc = "Wrap an error in [`Arc`].")]
    pub fn new(error: T) -> Self {
        #[cfg(not(feature = "multi-thread"))]
        {
            Self(Rc::new(error))
        }

        #[cfg(feature = "multi-thread")]
        {
            Self(Arc::new(error))
        }
    }
}

impl<T> AsRef<T> for ErrorContainer<T> {
    fn as_ref(&self) -> &T {
        self.0.as_ref()
    }
}

impl<T> Deref for ErrorContainer<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl<T> From<T> for ErrorContainer<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T> Clone for ErrorContainer<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: fmt::Debug> fmt::Debug for ErrorContainer<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.as_ref().fmt(f)
    }
}

impl<T: fmt::Display> fmt::Display for ErrorContainer<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.as_ref().fmt(f)
    }
}

impl<T: core::error::Error> core::error::Error for ErrorContainer<T> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integration_error_size() {
        assert!(
            size_of::<IntegrationError>() <= 2 * size_of::<usize>(),
            "size of IntegrationError ({}) should be ≤ 2 words",
            size_of::<IntegrationError>(),
        );
    }
}
