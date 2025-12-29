use super::{Feature, FeatureKey};
use crate::probe::Probe;
use crate::version::Version;

/// Detects support for the `SQLITE_DIRECTONLY` [user-defined function option][].
///
/// [user-defined function option]: https://sqlite.org/c3ref/c_deterministic.html#sqlitedirectonly
#[derive(Debug)]
#[doc(alias = "SQLITE_DIRECTONLY")]
pub struct FunctionDirectOnlyOption;

impl FunctionDirectOnlyOption {
    /// The SQLite version where `SQLITE_DIRECTONLY` was introduced.
    pub const AVAILABLE: Version = Version::release(3, 30);
}

#[cfg_attr(feature = "inherent", inherent::inherent)]
impl Feature for FunctionDirectOnlyOption {
    fn is_supported<P: Probe>(&self, probe: &P) -> bool {
        probe.version() >= Self::AVAILABLE
    }

    fn key(&self) -> FeatureKey {
        FeatureKey::FunctionDirectOnlyOption
    }
}

/// Detects support for the `SQLITE_INNOCUOUS` [user-defined function option][].
///
/// [user-defined function option]: https://sqlite.org/c3ref/c_deterministic.html#sqlitedirectonly
#[derive(Debug)]
#[doc(alias = "SQLITE_INNOCUOUS")]
pub struct FunctionInnocuousOption;

impl FunctionInnocuousOption {
    /// The SQLite version where `SQLITE_INNOCUOUS` was introduced.
    pub const AVAILABLE: Version = Version::release(3, 31);
}

#[cfg_attr(feature = "inherent", inherent::inherent)]
impl Feature for FunctionInnocuousOption {
    fn is_supported<P: Probe>(&self, probe: &P) -> bool {
        probe.version() >= Self::AVAILABLE
    }

    fn key(&self) -> FeatureKey {
        FeatureKey::FunctionInnocuousOption
    }
}

/// Detects support for the `SQLITE_SUBTYPE` [user-defined function option][].
///
/// [user-defined function option]: https://sqlite.org/c3ref/c_deterministic.html#sqlitesubtype
#[derive(Debug)]
#[doc(alias = "SQLITE_SUBTYPE")]
pub struct FunctionSubtypeOption;

impl FunctionSubtypeOption {
    /// The SQLite version where `SQLITE_SUBTYPE` was introduced.
    pub const AVAILABLE: Version = Version::release(3, 31);
}

#[cfg_attr(feature = "inherent", inherent::inherent)]
impl Feature for FunctionSubtypeOption {
    fn is_supported<P: Probe>(&self, probe: &P) -> bool {
        probe.version() >= Self::AVAILABLE
    }

    fn key(&self) -> FeatureKey {
        FeatureKey::FunctionSubtypeOption
    }
}

/// Detects support for [`SQLITE_STRICT_SUBTYPE`][], enforcing
/// [user-defined function option][] `SQLITE_RESULT_SUBTYPE`.
///
/// [`SQLITE_STRICT_SUBTYPE`]: https://sqlite.org/compile.html#strict_subtype
/// [user-defined function option]: https://sqlite.org/c3ref/c_deterministic.html#sqlitedirectonly
#[derive(Debug)]
#[doc(alias = "SQLITE_STRICT_SUBTYPE")]
#[doc(alias = "SQLITE_RESULT_SUBTYPE")]
pub struct FunctionStrictSubtypes;

impl FunctionStrictSubtypes {
    /// The SQLite version where `SQLITE_STRICT_SUBTYPE` was introduced.
    pub const AVAILABLE: Version = Version::release(3, 45);
}

#[cfg_attr(feature = "inherent", inherent::inherent)]
impl Feature for FunctionStrictSubtypes {
    fn is_supported<P: Probe>(&self, probe: &P) -> bool {
        probe.version() >= Self::AVAILABLE
    }

    fn key(&self) -> FeatureKey {
        FeatureKey::FunctionStrictSubtypes
    }
}

/// Detects support for the `SQLITE_SELFORDER1` [aggregate function option][].
///
/// [aggregate function option]: https://sqlite.org/c3ref/c_deterministic.html#sqliteselforder1
#[derive(Debug)]
#[doc(alias = "SQLITE_SELFORDER1")]
pub struct AggregateFunctionSelfOrdered;

impl AggregateFunctionSelfOrdered {
    /// The SQLite version where `SQLITE_SELFORDER1` was introduced.
    pub const AVAILABLE: Version = Version::release(3, 47);
}

#[cfg_attr(feature = "inherent", inherent::inherent)]
impl Feature for AggregateFunctionSelfOrdered {
    fn is_supported<P: Probe>(&self, probe: &P) -> bool {
        probe.version() >= Self::AVAILABLE
    }

    fn key(&self) -> FeatureKey {
        FeatureKey::AggregateFunctionSelfOrdered
    }
}
