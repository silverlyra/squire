use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime, TimeDelta, Utc};

use crate::{
    bind::Bind,
    error::{Error, ErrorCode},
    fetch::Fetch,
    ffi,
    types::Borrowed,
};

#[cfg_attr(docsrs, doc(cfg(feature = "chrono")))]
impl Bind<'_> for NaiveDate {
    type Value = ffi::String;

    fn into_bind_value(self) -> crate::Result<Self::Value> {
        ffi::String::display(&self)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "chrono")))]
impl<'b> Fetch<'b> for NaiveDate {
    type Value = Borrowed<'b, str>;

    fn from_value(value: Self::Value) -> crate::Result<Self> {
        value.parse().map_err(Error::from_fetch)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "chrono")))]
impl Bind<'_> for NaiveTime {
    type Value = ffi::String;

    fn into_bind_value(self) -> crate::Result<Self::Value> {
        ffi::String::display(&self)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "chrono")))]
impl<'b> Fetch<'b> for NaiveTime {
    type Value = Borrowed<'b, str>;

    fn from_value(value: Self::Value) -> crate::Result<Self> {
        value.parse().map_err(Error::from_fetch)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "chrono")))]
impl Bind<'_> for NaiveDateTime {
    type Value = ffi::String;

    fn into_bind_value(self) -> crate::Result<Self::Value> {
        ffi::String::display(&self)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "chrono")))]
impl<'b> Fetch<'b> for NaiveDateTime {
    type Value = Borrowed<'b, str>;

    fn from_value(value: Self::Value) -> crate::Result<Self> {
        value.parse().map_err(Error::from_fetch)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "chrono")))]
impl Bind<'_> for DateTime<Utc> {
    type Value = ffi::String;

    fn into_bind_value(self) -> crate::Result<Self::Value> {
        ffi::String::display(&self)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "chrono")))]
impl<'b> Fetch<'b> for DateTime<Utc> {
    type Value = Borrowed<'b, str>;

    fn from_value(value: Self::Value) -> crate::Result<Self> {
        value.parse().map_err(Error::from_fetch)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "chrono")))]
impl Bind<'_> for DateTime<FixedOffset> {
    type Value = ffi::String;

    fn into_bind_value(self) -> crate::Result<Self::Value> {
        ffi::String::display(&self)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "chrono")))]
impl<'b> Fetch<'b> for DateTime<FixedOffset> {
    type Value = Borrowed<'b, str>;

    fn from_value(value: Self::Value) -> crate::Result<Self> {
        value.parse().map_err(Error::from_fetch)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "chrono")))]
impl Bind<'_> for TimeDelta {
    type Value = i64;

    fn into_bind_value(self) -> crate::Result<Self::Value> {
        self.num_nanoseconds().ok_or_else(|| {
            Error::with_detail(
                ErrorCode::SQUIRE_PARAMETER_RANGE,
                "duration out of nanosecond range",
            )
        })
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "chrono")))]
impl Fetch<'_> for TimeDelta {
    type Value = i64;

    fn from_value(value: Self::Value) -> crate::Result<Self> {
        Ok(TimeDelta::nanoseconds(value))
    }
}
