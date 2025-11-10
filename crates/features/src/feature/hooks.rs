use super::{Feature, FeatureKey};
use crate::probe::{Flag, Probe};

/// Detects support for [pre-update hooks][preupdate].
///
/// [preupdate]: https://sqlite.org/c3ref/preupdate_count.html
#[derive(Debug)]
#[doc(alias = "SQLITE_ENABLE_PREUPDATE_HOOK")]
pub struct PreUpdateHook;

#[cfg_attr(feature = "inherent", inherent::inherent)]
impl Feature for PreUpdateHook {
    fn is_supported<P: Probe>(&self, probe: &P) -> bool {
        probe.is_set(Flag::EnablePreUpdateHook)
    }

    fn key(&self) -> FeatureKey {
        FeatureKey::PreUpdateHook
    }
}

/// Detects support for [progress handler callbacks][progress].
///
/// [progress]: https://sqlite.org/c3ref/progress_handler.html
#[derive(Debug)]
#[doc(alias = "SQLITE_ENABLE_PROGRESS_CALLBACK")]
pub struct ProgressCallback;

#[cfg_attr(feature = "inherent", inherent::inherent)]
impl Feature for ProgressCallback {
    fn is_supported<P: Probe>(&self, probe: &P) -> bool {
        probe.is_set(Flag::EnableProgressCallback)
    }

    fn key(&self) -> FeatureKey {
        FeatureKey::ProgressCallback
    }
}

/// Detects support for the [authorizer callback][auth].
///
/// [auth]: https://sqlite.org/c3ref/set_authorizer.html
#[derive(Debug)]
#[doc(alias = "SQLITE_OMIT_AUTHORIZATION")]
pub struct AuthorizationCallback;

#[cfg_attr(feature = "inherent", inherent::inherent)]
impl Feature for AuthorizationCallback {
    fn is_supported<P: Probe>(&self, probe: &P) -> bool {
        !probe.is_set(Flag::OmitAuthorization)
    }

    fn key(&self) -> FeatureKey {
        FeatureKey::AuthorizationCallback
    }
}
