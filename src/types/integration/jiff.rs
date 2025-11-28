use jiff::{SignedDuration, Span, Timestamp, Zoned};

use crate::{bind::Bind, error::Error, ffi, types::Borrowed, value::Fetch};

#[cfg_attr(docsrs, doc(cfg(feature = "jiff")))]
impl Bind<'_> for SignedDuration {
    type Value = i64;

    fn into_bind_value(self) -> crate::Result<Self::Value> {
        Ok(self.as_nanos() as i64)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "jiff")))]
impl Fetch<'_> for SignedDuration {
    type Value = i64;

    fn from_column_value(value: Self::Value) -> crate::Result<Self> {
        Ok(Self::from_nanos(value))
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "jiff")))]
impl Bind<'_> for Span {
    type Value = ffi::String;

    fn into_bind_value(self) -> crate::Result<Self::Value> {
        ffi::String::display(&self)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "jiff")))]
impl<'b> Fetch<'b> for Span {
    type Value = Borrowed<'b, str>;

    fn from_column_value(value: Self::Value) -> crate::Result<Self> {
        value.parse().map_err(Error::from_fetch)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "jiff")))]
impl Bind<'_> for Timestamp {
    type Value = i64;

    fn into_bind_value(self) -> crate::Result<Self::Value> {
        Ok(self.as_nanosecond() as i64)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "jiff")))]
impl Fetch<'_> for Timestamp {
    type Value = i64;

    fn from_column_value(value: Self::Value) -> crate::Result<Self> {
        Timestamp::from_nanosecond(value as i128).map_err(Error::from_fetch)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "jiff")))]
impl Bind<'_> for Zoned {
    type Value = ffi::String;

    fn into_bind_value(self) -> crate::Result<Self::Value> {
        ffi::String::display(&self)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "jiff")))]
impl<'b> Fetch<'b> for Zoned {
    type Value = Borrowed<'b, str>;

    fn from_column_value(value: Self::Value) -> crate::Result<Self> {
        value.parse().map_err(Error::from_fetch)
    }
}
