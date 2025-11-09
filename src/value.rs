use crate::{
    error::{Error, FetchError, Result},
    ffi::{self, Fetch as _},
    statement::Statement,
    types::{Borrowed, ColumnIndex, RowId},
};

pub trait Fetch<'r>: Sized {
    type Value: ffi::Fetch<'r>;

    fn fetch<'c>(statement: &'r Statement<'c>, column: ColumnIndex) -> Result<Self> {
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

identity!(f64);
primitive!(i32 :> i8);
primitive!(i32 :> u8);
primitive!(i32 :> i16);
primitive!(i32 :> u16);
identity!(i32);
primitive!(i64 :> u32);
#[cfg(target_pointer_width = "32")]
primitive!(i32 :> isize);
#[cfg(target_pointer_width = "64")]
primitive!(i64 :> isize);
primitive!(i64 :> usize);
identity!(i64);
primitive!(i64 :> u64);

/// Read the column as an [`f64`] with
/// [`sqlite3_column_double`](sqlite::sqlite3_column_double), and cast to
/// [`f32`] with `value as f32`.
///
/// If the value overflows an `f32` (a previously [finite](f64::is_finite())
/// `f64` became [infinite](f32::is_infinite())), returns a [range
/// error](FetchError::Range).
impl<'r> Fetch<'r> for f32 {
    type Value = f64;

    #[inline]
    fn from_column_value(value: Self::Value) -> Result<Self> {
        let result = value as f32;

        // Check if a finite value became infinite (overflow)
        if value.is_finite() && result.is_infinite() {
            Err(Error::fetch(
                FetchError::Range,
                format!("f64 value {} overflows f32 range", value),
            ))
        } else {
            Ok(result)
        }
    }
}

/// Read the column as an [`i32`] with
/// [`sqlite3_column_int`](sqlite::sqlite3_column_int); any nonzero value is
/// `true`, and `0` is `false`.
///
/// **Note** that when applied to a column of a different data type, such as the
/// text `'true'`, SQLite may simply return `0`, which Squire interprets as
/// `false`.
impl<'r> Fetch<'r> for bool {
    type Value = i32;

    fn from_column_value(value: Self::Value) -> Result<Self> {
        Ok(value != 0)
    }
}

impl<'r> Fetch<'r> for RowId {
    type Value = i64;

    fn from_column_value(value: Self::Value) -> Result<Self> {
        RowId::new(value).ok_or_else(
            #[cold]
            || Error::fetch(FetchError::Range, "SQLite row ID cannot be 0"),
        )
    }
}

impl<'r, 'a> Fetch<'r> for &'a str
where
    'r: 'a,
{
    type Value = Borrowed<'r, str>;

    fn from_column_value(value: Self::Value) -> Result<Self> {
        // SAFETY: We have 'r: 'a, so shortening the lifetime from 'r to 'a is sound.
        // The caller ensures 'r outlives 'a, so the reference remains valid.
        unsafe { Ok(core::mem::transmute::<&'r str, &'a str>(value.into_inner())) }
    }
}

impl<'r, 'a> Fetch<'r> for &'a [u8]
where
    'r: 'a,
{
    type Value = Borrowed<'r, [u8]>;

    fn from_column_value(value: Self::Value) -> Result<Self> {
        // SAFETY: We have 'r: 'a, so shortening the lifetime from 'r to 'a is sound.
        // The caller ensures 'r outlives 'a, so the reference remains valid.
        unsafe {
            Ok(core::mem::transmute::<&'r [u8], &'a [u8]>(
                value.into_inner(),
            ))
        }
    }
}

impl<'r> Fetch<'r> for String {
    type Value = Borrowed<'r, str>;

    fn from_column_value(value: Self::Value) -> Result<Self> {
        Ok(value.to_owned())
    }
}

impl<'r> Fetch<'r> for Vec<u8> {
    type Value = Borrowed<'r, [u8]>;

    fn from_column_value(value: Self::Value) -> Result<Self> {
        Ok(value.to_owned())
    }
}

impl<'r, T> Fetch<'r> for Option<T>
where
    T: Fetch<'r>,
{
    type Value = Option<T::Value>;

    fn from_column_value(value: Self::Value) -> Result<Self> {
        Ok(match value {
            Some(value) => Some(T::from_column_value(value)?),
            None => None,
        })
    }
}
