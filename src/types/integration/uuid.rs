use uuid::Uuid;

use crate::{bind::Bind, error::Error, ffi, types::Borrowed, value::Fetch};

#[cfg_attr(docsrs, doc(cfg(feature = "uuid")))]
impl Bind<'_> for Uuid {
    type Value = ffi::Bytes;

    fn into_bind_value(self) -> crate::Result<Self::Value> {
        Ok(ffi::Bytes::from(self.into_bytes()))
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "uuid")))]
impl<'a, 'b> Bind<'b> for &'a Uuid
where
    'a: 'b,
{
    type Value = Borrowed<'b, [u8]>;

    fn into_bind_value(self) -> crate::Result<Self::Value> {
        Ok(Borrowed::new(self.as_bytes()))
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "uuid")))]
impl<'b> Fetch<'b> for Uuid {
    type Value = Borrowed<'b, [u8]>;

    fn from_column_value(value: Self::Value) -> crate::Result<Self> {
        Self::from_slice(&value).map_err(Error::from_fetch)
    }
}
