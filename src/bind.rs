use crate::{
    blob::Reservation,
    error::{Error, Result},
    ffi,
    types::RowId,
};

/// A value which can be [bound as a parameter][bind] in SQLite [prepared
/// statements](crate::Statement).
///
/// The lifetime parameter `'b` represents the lifetime for which SQLite may
/// access the bound data.
///
/// [bind]: https://sqlite.org/c3ref/bind_blob.html
pub trait Bind<'b> {
    /// A type directly handled by the SQLite [parameter binding][bind] API.
    ///
    /// [bind]: https://sqlite.org/c3ref/bind_blob.html
    type Value: ffi::Bind<'b>;

    /// Convert `self` into a directly bindable type.
    ///
    /// If conversion fails, return an `Err` result.
    fn into_bind_value(self) -> Result<Self::Value>;
}

/// Defines [`Bind`] for a type that implements [`ffi::Bind`].
macro_rules! identity {
    ($($t:ty),+) => {
        $(
            impl<'b> Bind<'b> for $t {
                type Value = Self;

                #[inline]
                fn into_bind_value(self) -> Result<Self::Value> {
                    Ok(self)
                }
            }
        )+
    };

    ($($t:ty),+ ,) => {
        identity!($($t),+);
    };
}

/// Defines [`Bind`] for a primitive integer type.
macro_rules! primitive {
    ($($t:ty as $b:ty),+) => {
        $(
            impl<'b> Bind<'b> for $t {
                type Value = $b;

                #[inline]
                fn into_bind_value(self) -> Result<Self::Value> {
                    Ok(self as $b)
                }
            }
        )+
    };

    ($($t:ty as $b:ty),+ ,) => {
        primitive!($($t as $b),+);
    };
}

primitive!(
    char as i32,
    f32 as f64,
    f64 as f64,
    i8 as i32,
    u8 as i32,
    i16 as i32,
    u16 as i32,
    i32 as i32,
    u32 as i64,
    i64 as i64,
);

#[cfg(target_pointer_width = "32")]
primitive!(isize as i32, usize as i64);
#[cfg(target_pointer_width = "64")]
primitive!(isize as i64);

impl<'b> Bind<'b> for u64 {
    type Value = i64;

    fn into_bind_value(self) -> Result<Self::Value> {
        i64::try_from(self).map_err(
            #[cold]
            |_| Error::bind("u64 value cannot fit in i64 parameter"),
        )
    }
}

#[cfg(target_pointer_width = "64")]
impl<'b> Bind<'b> for usize {
    type Value = i64;

    fn into_bind_value(self) -> Result<Self::Value> {
        i64::try_from(self).map_err(
            #[cold]
            |_| Error::bind("usize value cannot fit in i64 parameter"),
        )
    }
}

/// [`bool`] values are bound as `1` (for `true`) or `0` (for `false`).
impl<'b> Bind<'b> for bool {
    type Value = i32;

    fn into_bind_value(self) -> Result<Self::Value> {
        Ok(if self { 1 } else { 0 })
    }
}

identity!(&str, String, &[u8], Vec<u8>, Reservation);

impl<'b> Bind<'b> for RowId {
    type Value = i64;

    fn into_bind_value(self) -> Result<Self::Value> {
        Ok(self.into_inner())
    }
}

impl<'a, 'b> Bind<'b> for ffi::Static<'a, str>
where
    'a: 'b,
{
    type Value = Self;

    #[inline]
    fn into_bind_value(self) -> Result<Self::Value> {
        Ok(self)
    }
}

impl<'b, T> Bind<'b> for Option<T>
where
    T: Bind<'b>,
{
    type Value = Option<<T as Bind<'b>>::Value>;

    fn into_bind_value(self) -> Result<Self::Value> {
        if let Some(value) = self {
            value.into_bind_value().map(Some)
        } else {
            Ok(None)
        }
    }
}
