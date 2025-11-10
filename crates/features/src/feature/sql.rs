use super::{Feature, FeatureKey};
use crate::probe::{Flag, Probe};
use crate::version::Version;

/// Detects support for [SQL normalization][normalize] via
/// `sqlite3_normalized_sql()`.
///
/// [normalize]: https://sqlite.org/c3ref/expanded_sql.html
#[derive(Debug)]
#[doc(alias = "SQLITE_ENABLE_NORMALIZE")]
#[doc(alias = "sqlite3_normalized_sql")]
pub struct NormalizeSql;

#[cfg_attr(feature = "inherent", inherent::inherent)]
impl Feature for NormalizeSql {
    fn is_supported<P: Probe>(&self, probe: &P) -> bool {
        probe.is_set(Flag::EnableNormalizeSql)
    }

    fn key(&self) -> FeatureKey {
        FeatureKey::NormalizeSql
    }
}

/// Detects support for [silencing error logging][] from `prepare`.
///
/// [silencing error logging]: https://sqlite.org/c3ref/c_prepare_dont_log.html#sqlitepreparedontlog
#[derive(Debug)]
#[doc(alias = "SQLITE_PREPARE_DONT_LOG")]
pub struct PrepareQuiet;

impl PrepareQuiet {
    /// The SQLite version where `SQLITE_PREPARE_DONT_LOG` was introduced.
    pub const AVAILABLE: Version = Version::release(3, 48);
}

#[cfg_attr(feature = "inherent", inherent::inherent)]
impl Feature for PrepareQuiet {
    fn is_supported<P: Probe>(&self, probe: &P) -> bool {
        probe.version() >= Self::AVAILABLE
    }

    fn key(&self) -> FeatureKey {
        FeatureKey::PrepareQuiet
    }
}

/// Detects support for TCL variable substitution in SQL.
#[derive(Debug)]
#[doc(alias = "SQLITE_OMIT_TCL_VARIABLE")]
pub struct TclVariables;

#[cfg_attr(feature = "inherent", inherent::inherent)]
impl Feature for TclVariables {
    fn is_supported<P: Probe>(&self, probe: &P) -> bool {
        !probe.is_set(Flag::OmitTclVariables)
    }

    fn key(&self) -> FeatureKey {
        FeatureKey::TclVariables
    }
}
