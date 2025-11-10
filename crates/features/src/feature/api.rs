use super::{Feature, FeatureKey};
use crate::probe::{Flag, Probe};

/// Detects support for [API armor][armor], which adds extra defensive checks to the SQLite API.
///
/// [armor]: https://sqlite.org/compile.html#enable_api_armor
#[derive(Debug)]
#[doc(alias = "SQLITE_ENABLE_API_ARMOR")]
pub struct ApiArmor;

#[cfg_attr(feature = "inherent", inherent::inherent)]
impl Feature for ApiArmor {
    fn is_supported<P: Probe>(&self, probe: &P) -> bool {
        probe.is_set(Flag::EnableApiArmor)
    }

    fn key(&self) -> FeatureKey {
        FeatureKey::ApiArmor
    }
}

/// Detects support for the [`sqlite3_get_table()`][get_table] convenience API.
///
/// [get_table]: https://sqlite.org/c3ref/free_table.html
#[derive(Debug)]
#[doc(alias = "SQLITE_OMIT_GET_TABLE")]
#[doc(alias = "sqlite3_get_table")]
pub struct GetTable;

#[cfg_attr(feature = "inherent", inherent::inherent)]
impl Feature for GetTable {
    fn is_supported<P: Probe>(&self, probe: &P) -> bool {
        !probe.is_set(Flag::OmitGetTable)
    }

    fn key(&self) -> FeatureKey {
        FeatureKey::GetTable
    }
}

/// Detects support for the [`sqlite3_complete()`][complete] API for checking SQL statement completeness.
///
/// [complete]: https://sqlite.org/c3ref/complete.html
#[derive(Debug)]
#[doc(alias = "SQLITE_OMIT_COMPLETE")]
#[doc(alias = "sqlite3_complete")]
pub struct Complete;

#[cfg_attr(feature = "inherent", inherent::inherent)]
impl Feature for Complete {
    fn is_supported<P: Probe>(&self, probe: &P) -> bool {
        !probe.is_set(Flag::OmitComplete)
    }

    fn key(&self) -> FeatureKey {
        FeatureKey::Complete
    }
}

/// Detects support for deprecated APIs.
#[derive(Debug)]
#[doc(alias = "SQLITE_OMIT_DEPRECATED")]
pub struct Deprecated;

#[cfg_attr(feature = "inherent", inherent::inherent)]
impl Feature for Deprecated {
    fn is_supported<P: Probe>(&self, probe: &P) -> bool {
        !probe.is_set(Flag::OmitDeprecated)
    }

    fn key(&self) -> FeatureKey {
        FeatureKey::Deprecated
    }
}

/// Detects support for [tracing and profiling][trace] APIs.
///
/// [trace]: https://sqlite.org/c3ref/profile.html
#[derive(Debug)]
#[doc(alias = "SQLITE_OMIT_TRACE")]
#[doc(alias = "sqlite3_trace")]
#[doc(alias = "sqlite3_profile")]
pub struct Trace;

#[cfg_attr(feature = "inherent", inherent::inherent)]
impl Feature for Trace {
    fn is_supported<P: Probe>(&self, probe: &P) -> bool {
        !probe.is_set(Flag::OmitTrace)
    }

    fn key(&self) -> FeatureKey {
        FeatureKey::Trace
    }
}

/// Detects support for automatic initialization via [`sqlite3_initialize()`][init].
///
/// [init]: https://sqlite.org/c3ref/initialize.html
#[derive(Debug)]
#[doc(alias = "SQLITE_OMIT_AUTOINIT")]
#[doc(alias = "sqlite3_initialize")]
pub struct AutomaticInitialize;

#[cfg_attr(feature = "inherent", inherent::inherent)]
impl Feature for AutomaticInitialize {
    fn is_supported<P: Probe>(&self, probe: &P) -> bool {
        !probe.is_set(Flag::OmitAutomaticInitialize)
    }

    fn key(&self) -> FeatureKey {
        FeatureKey::AutomaticInitialize
    }
}

/// Detects support for automatic reset of prepared statements.
#[derive(Debug)]
#[doc(alias = "SQLITE_OMIT_AUTORESET")]
pub struct AutomaticReset;

#[cfg_attr(feature = "inherent", inherent::inherent)]
impl Feature for AutomaticReset {
    fn is_supported<P: Probe>(&self, probe: &P) -> bool {
        !probe.is_set(Flag::OmitAutomaticReset)
    }

    fn key(&self) -> FeatureKey {
        FeatureKey::AutomaticReset
    }
}

/// Detects support for enhanced [memory management][mem] APIs.
///
/// [mem]: https://sqlite.org/malloc.html
#[derive(Debug)]
#[doc(alias = "SQLITE_ENABLE_MEMORY_MANAGEMENT")]
pub struct MemoryManagement;

#[cfg_attr(feature = "inherent", inherent::inherent)]
impl Feature for MemoryManagement {
    fn is_supported<P: Probe>(&self, probe: &P) -> bool {
        probe.is_set(Flag::EnableMemoryManagement)
    }

    fn key(&self) -> FeatureKey {
        FeatureKey::MemoryManagement
    }
}
