use crate::{
    blob::Reservation,
    error::{Error, Result},
    ffi,
    statement::{Binding, Statement},
};

pub use ffi::Index;

/// A value which can be [bound as a parameter][bind] in SQLite [prepared
/// statements](crate::Statement).
///
/// [bind]: https://sqlite.org/c3ref/bind_blob.html
pub trait Bind<'b> {
    type Value: ffi::Bind<'b>;

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

pub trait Parameters<'s> {
    type Indexes: Copy + Sized;

    fn resolve<'c>(statement: &Statement<'c>) -> Option<Self::Indexes>;

    fn bind<'c>(self, binding: &mut Binding<'c, 's>, indexes: Self::Indexes) -> Result<()>
    where
        'c: 's;
}

impl<'s, T> Parameters<'s> for T
where
    T: Bind<'s>,
{
    type Indexes = ();

    #[inline(always)]
    fn resolve<'c>(_statement: &Statement<'c>) -> Option<Self::Indexes> {
        Some(())
    }

    fn bind<'c>(self, binding: &mut Binding<'c, 's>, _indexes: Self::Indexes) -> Result<()>
    where
        'c: 's,
    {
        binding.set(Index::INITIAL, self)?;
        Ok(())
    }
}

macro_rules! parameter_tuples {
    ($i:literal => $t:ident) => {};

    ($($ih:literal => $th:ident),*, $it:literal => $tt:ident) => {};
}
