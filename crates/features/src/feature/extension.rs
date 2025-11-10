use super::{Feature, FeatureKey};
use crate::probe::{Flag, Probe};

/// Detects support for [runtime loadable extensions][ext].
///
/// [ext]: https://sqlite.org/loadext.html
#[derive(Debug)]
#[doc(alias = "SQLITE_OMIT_LOAD_EXTENSION")]
pub struct LoadExtension;

#[cfg_attr(feature = "inherent", inherent::inherent)]
impl Feature for LoadExtension {
    fn is_supported<P: Probe>(&self, probe: &P) -> bool {
        !probe.is_set(Flag::OmitLoadExtension)
    }

    fn key(&self) -> FeatureKey {
        FeatureKey::LoadExtension
    }
}

/// Detects support for the SQLite [session extension][session] for tracking and
/// applying database changes.
///
/// [session]: https://sqlite.org/sessionintro.html
#[derive(Debug)]
#[doc(alias = "SQLITE_ENABLE_SESSION")]
pub struct Session;

#[cfg_attr(feature = "inherent", inherent::inherent)]
impl Feature for Session {
    fn is_supported<P: Probe>(&self, probe: &P) -> bool {
        probe.is_set(Flag::EnableSession)
    }

    fn key(&self) -> FeatureKey {
        FeatureKey::Session
    }
}
