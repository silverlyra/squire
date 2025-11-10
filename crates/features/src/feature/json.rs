use super::{Feature, FeatureKey};
use crate::probe::{Flag, Probe};
use crate::version::Version;

/// Detects support for SQLite [JSON functions and operators][json].
///
/// [json]: https://sqlite.org/json1.html
#[derive(Debug)]
#[doc(alias = "SQLITE_ENABLE_JSON1")]
#[doc(alias = "SQLITE_OMIT_JSON")]
pub struct Json;

impl Json {
    /// The SQLite version where JSON support became opt-out.
    pub const ENABLED_BY_DEFAULT: Version = Version::release(3, 38);
}

#[cfg_attr(feature = "inherent", inherent::inherent)]
impl Feature for Json {
    fn is_supported<P: Probe>(&self, probe: &P) -> bool {
        if probe.version() < Self::ENABLED_BY_DEFAULT {
            probe.is_set(Flag::EnableJson)
        } else {
            !probe.is_set(Flag::OmitJson)
        }
    }

    fn key(&self) -> FeatureKey {
        FeatureKey::Json
    }
}

/// Detects support for the SQLite [binary JSON encoding][jsonb].
///
/// [jsonb]: https://sqlite.org/jsonb.html
#[derive(Debug)]
pub struct Jsonb;

impl Jsonb {
    /// The SQLite version where JSONB was introduced.
    pub const AVAILABLE: Version = Version::release(3, 45);
}

#[cfg_attr(feature = "inherent", inherent::inherent)]
impl Feature for Jsonb {
    fn is_supported<P: Probe>(&self, probe: &P) -> bool {
        Json.is_supported(probe) && probe.version() >= Self::AVAILABLE
    }

    fn key(&self) -> FeatureKey {
        FeatureKey::Jsonb
    }
}
