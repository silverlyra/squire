use jiff::{SignedDuration, Span, Timestamp, Zoned, civil};

use crate::{bind::Bind, error::Error, fetch::Fetch, types::Borrowed};

#[cfg_attr(docsrs, doc(cfg(feature = "jiff")))]
impl Bind<'_> for civil::Date {
    type Value = String;

    fn into_bind_value(self) -> crate::Result<Self::Value> {
        Ok(self.to_string())
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "jiff")))]
impl<'b> Fetch<'b> for civil::Date {
    type Value = Borrowed<'b, str>;

    fn from_value(value: Self::Value) -> crate::Result<Self> {
        value.parse().map_err(Error::from_fetch)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "jiff")))]
impl Bind<'_> for civil::DateTime {
    type Value = String;

    fn into_bind_value(self) -> crate::Result<Self::Value> {
        Ok(self.to_string())
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "jiff")))]
impl<'b> Fetch<'b> for civil::DateTime {
    type Value = Borrowed<'b, str>;

    fn from_value(value: Self::Value) -> crate::Result<Self> {
        value.parse().map_err(Error::from_fetch)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "jiff")))]
impl Bind<'_> for civil::Time {
    type Value = String;

    fn into_bind_value(self) -> crate::Result<Self::Value> {
        Ok(self.to_string())
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "jiff")))]
impl<'b> Fetch<'b> for civil::Time {
    type Value = Borrowed<'b, str>;

    fn from_value(value: Self::Value) -> crate::Result<Self> {
        value.parse().map_err(Error::from_fetch)
    }
}

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

    fn from_value(value: Self::Value) -> crate::Result<Self> {
        Ok(Self::from_nanos(value))
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "jiff")))]
impl Bind<'_> for Span {
    type Value = String;

    fn into_bind_value(self) -> crate::Result<Self::Value> {
        Ok(self.to_string())
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "jiff")))]
impl<'b> Fetch<'b> for Span {
    type Value = Borrowed<'b, str>;

    fn from_value(value: Self::Value) -> crate::Result<Self> {
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

    fn from_value(value: Self::Value) -> crate::Result<Self> {
        Timestamp::from_nanosecond(value as i128).map_err(Error::from_fetch)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "jiff")))]
impl Bind<'_> for Zoned {
    type Value = String;

    fn into_bind_value(self) -> crate::Result<Self::Value> {
        Ok(self.to_string())
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "jiff")))]
impl<'a, 'b> Bind<'b> for &'a Zoned
where
    'a: 'b,
{
    type Value = String;

    fn into_bind_value(self) -> crate::Result<Self::Value> {
        Ok(self.to_string())
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "jiff")))]
impl<'b> Fetch<'b> for Zoned {
    type Value = Borrowed<'b, str>;

    fn from_value(value: Self::Value) -> crate::Result<Self> {
        value.parse().map_err(Error::from_fetch)
    }
}
