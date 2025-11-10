use super::{Feature, FeatureKey};
use crate::probe::{Flag, Probe};

/// Detects support for the [ATTACH DATABASE][attach] statement.
///
/// [attach]: https://sqlite.org/lang_attach.html
#[derive(Debug)]
#[doc(alias = "SQLITE_OMIT_ATTACH")]
pub struct Attach;

#[cfg_attr(feature = "inherent", inherent::inherent)]
impl Feature for Attach {
    fn is_supported<P: Probe>(&self, probe: &P) -> bool {
        !probe.is_set(Flag::OmitAttach)
    }

    fn key(&self) -> FeatureKey {
        FeatureKey::Attach
    }
}

/// Detects support for [temporary databases][tempdb].
///
/// [tempdb]: https://sqlite.org/tempfiles.html
#[derive(Debug)]
#[doc(alias = "SQLITE_OMIT_TEMPDB")]
pub struct TemporaryDatabase;

#[cfg_attr(feature = "inherent", inherent::inherent)]
impl Feature for TemporaryDatabase {
    fn is_supported<P: Probe>(&self, probe: &P) -> bool {
        !probe.is_set(Flag::OmitTemporaryDatabase)
    }

    fn key(&self) -> FeatureKey {
        FeatureKey::TemporaryDatabase
    }
}

/// Detects support for [shared cache mode][shared].
///
/// [shared]: https://sqlite.org/sharedcache.html
#[derive(Debug)]
#[doc(alias = "SQLITE_OMIT_SHARED_CACHE")]
pub struct SharedCache;

#[cfg_attr(feature = "inherent", inherent::inherent)]
impl Feature for SharedCache {
    fn is_supported<P: Probe>(&self, probe: &P) -> bool {
        !probe.is_set(Flag::OmitSharedCache)
    }

    fn key(&self) -> FeatureKey {
        FeatureKey::SharedCache
    }
}
