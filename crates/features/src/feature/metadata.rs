use super::{Feature, FeatureKey};
use crate::probe::{Flag, Probe};

/// Detects support for [column metadata APIs][meta] like `sqlite3_column_database_name()`.
///
/// [meta]: https://sqlite.org/c3ref/column_database_name.html
#[derive(Debug)]
#[doc(alias = "SQLITE_ENABLE_COLUMN_METADATA")]
pub struct ColumnMetadata;

#[cfg_attr(feature = "inherent", inherent::inherent)]
impl Feature for ColumnMetadata {
    fn is_supported<P: Probe>(&self, probe: &P) -> bool {
        probe.is_set(Flag::EnableColumnMetadata)
    }

    fn key(&self) -> FeatureKey {
        FeatureKey::ColumnMetadata
    }
}

/// Detects support for the [`sqlite3_column_decltype()`][decltype] API.
///
/// [decltype]: https://sqlite.org/c3ref/column_decltype.html
#[derive(Debug)]
#[doc(alias = "SQLITE_OMIT_DECLTYPE")]
pub struct ColumnDeclaredType;

#[cfg_attr(feature = "inherent", inherent::inherent)]
impl Feature for ColumnDeclaredType {
    fn is_supported<P: Probe>(&self, probe: &P) -> bool {
        !probe.is_set(Flag::OmitColumnDeclaredType)
    }

    fn key(&self) -> FeatureKey {
        FeatureKey::ColumnDeclaredType
    }
}
