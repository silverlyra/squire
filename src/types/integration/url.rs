use url::Url;

use crate::{bind::Bind, error::Error, types::Borrowed, fetch::Fetch};

#[cfg_attr(docsrs, doc(cfg(feature = "url")))]
impl<'b> Bind<'b> for Url {
    type Value = String;

    fn into_bind_value(self) -> crate::Result<Self::Value> {
        Ok(self.into())
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "url")))]
impl<'a, 'b> Bind<'b> for &'a Url
where
    'a: 'b,
{
    type Value = Borrowed<'b, str>;

    fn into_bind_value(self) -> crate::Result<Self::Value> {
        Ok(Borrowed::new(self.as_str()))
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "url")))]
impl<'b> Fetch<'b> for Url {
    type Value = Borrowed<'b, str>;

    fn from_column_value(value: Self::Value) -> crate::Result<Self> {
        value.parse().map_err(Error::from_fetch)
    }
}
