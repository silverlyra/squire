use crate::{
    error::{Error, FetchError, Result},
    ffi::{self, Fetch as _},
    statement::Statement,
};

pub use ffi::Column;

pub trait Fetch<'r>: Sized {
    type Value: ffi::Fetch<'r>;

    fn fetch<'c>(statement: &'r Statement<'c>, column: Column) -> Result<Self> {
        let value = unsafe { Self::Value::fetch(statement.internal_ref(), column) };
        Self::from_column_value(value)
    }

    fn from_column_value(value: Self::Value) -> Result<Self>;
}

/// Defines [`Fetch`] for a type that implements [`ffi::Fetch`].
macro_rules! identity {
    ($($t:ty),+) => {
        $(
            impl<'r> Fetch<'r> for $t {
                type Value = Self;

                #[inline]
                fn from_column_value(value: Self::Value) -> Result<Self> {
                    Ok(value)
                }
            }
        )+
    };

    ($($t:ty),+ ,) => {
        identity!($($t),+);
    };
}

macro_rules! primitive {
    ($v:ty => $t:ty) => {
        impl<'r> Fetch<'r> for $t {
            type Value = $v;

            #[inline]
            fn from_column_value(value: Self::Value) -> Result<Self> {
                Ok($t::from(value))
            }
        }
    };

    ($v:ty :> $t:ty) => {
        impl<'r> Fetch<'r> for $t {
            type Value = $v;

            #[inline]
            fn from_column_value(value: Self::Value) -> Result<Self> {
                <$t as TryFrom<$v>>::try_from(value).map_err(
                    #[cold]
                    |err| Error::fetch(FetchError::Range, err.to_string()),
                )
            }
        }
    };
}

primitive!(i32 :> i8);
primitive!(i32 :> u8);
primitive!(i32 :> i16);
primitive!(i32 :> u16);
identity!(i32);
primitive!(i64 :> u32);
identity!(i64);
primitive!(i64 :> u64);
