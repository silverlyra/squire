#[cfg(feature = "json")]
use squire_serde::json;
#[cfg(feature = "jsonb")]
use squire_serde::jsonb;
use squire_serde::{Deserialize, Serialize};

use crate::{
    bind::Bind,
    error::{Error, Result},
    types::Borrowed,
    fetch::Fetch,
};

/// A value which is stored in SQLite [serialized](Serialize) as JSON.
#[cfg(feature = "json")]
#[cfg_attr(docsrs, doc(cfg(all(feature = "json", feature = "serde"))))]
pub struct Json<T>(pub T);

#[cfg(feature = "json")]
#[cfg_attr(docsrs, doc(cfg(all(feature = "jsonb", feature = "serde"))))]
impl<'b, T> Bind<'b> for Json<T>
where
    T: Serialize,
{
    type Value = Vec<u8>;

    fn into_bind_value(self) -> Result<Self::Value> {
        json::to_vec(&self.0).map_err(Error::from_bind)
    }
}

#[cfg(feature = "json")]
#[cfg_attr(docsrs, doc(cfg(all(feature = "json", feature = "serde"))))]
impl<'r, T> Fetch<'r> for Json<T>
where
    T: Deserialize<'r>,
{
    type Value = Borrowed<'r, [u8]>;

    fn from_column_value(value: Self::Value) -> Result<Self> {
        match json::from_slice(value.into_inner()) {
            Ok(value) => Ok(Self(value)),
            Err(err) => Err(Error::from_fetch(err)),
        }
    }
}

/// A value which is stored in SQLite [serialized](Serialize) as [JSONB][jsonb].
///
/// # ⚠️ Warning
///
/// [`Jsonb`] is implemented with [`serde_sqlite_jsonb`][], which provides its
/// own implementation of the SQLite [binary JSON format][jsonb]. This
/// implementation may contain bugs not present in SQLite.
///
/// For a safer alternative, use [`Json`] instead, and [convert][] to and from
/// JSONB in your queries using the `jsonb()` and `json()` SQL functions.
///
/// [convert]: https://sqlite.org/json1.html
/// [jsonb]: https://lib.rs/crates/serde_sqlite_jsonb
/// [`serde_sqlite_jsonb`]: https://lib.rs/crates/serde_sqlite_jsonb
#[cfg(feature = "jsonb")]
#[cfg_attr(docsrs, doc(cfg(all(feature = "jsonb", feature = "serde"))))]
pub struct Jsonb<T>(pub T);

#[cfg(feature = "jsonb")]
#[cfg_attr(docsrs, doc(cfg(all(feature = "jsonb", feature = "serde"))))]
impl<'b, T> Bind<'b> for Jsonb<T>
where
    T: Serialize,
{
    type Value = Vec<u8>;

    fn into_bind_value(self) -> Result<Self::Value> {
        jsonb::to_vec(&self.0).map_err(Error::from_bind)
    }
}

#[cfg(feature = "jsonb")]
#[cfg_attr(docsrs, doc(cfg(all(feature = "jsonb", feature = "serde"))))]
impl<'r, T> Fetch<'r> for Jsonb<T>
where
    T: Deserialize<'r>,
{
    type Value = Borrowed<'r, [u8]>;

    fn from_column_value(value: Self::Value) -> Result<Self> {
        match jsonb::from_slice(value.into_inner()) {
            Ok(value) => Ok(Self(value)),
            Err(err) => Err(Error::from_fetch(err)),
        }
    }
}
