use super::{Feature, FeatureKey};
use crate::probe::{Flag, Probe};

/// Detects support for SQLite [FTS3 and FTS4][fts3] full-text search.
///
/// [fts3]: https://sqlite.org/fts3.html
#[derive(Debug)]
#[doc(alias = "SQLITE_ENABLE_FTS3")]
pub struct Fts3;

#[cfg_attr(feature = "inherent", inherent::inherent)]
impl Feature for Fts3 {
    fn is_supported<P: Probe>(&self, probe: &P) -> bool {
        probe.is_set(Flag::EnableFts3)
    }

    fn key(&self) -> FeatureKey {
        FeatureKey::Fts3
    }
}

/// Detects support for SQLite [FTS5][fts5] full-text search.
///
/// [fts5]: https://sqlite.org/fts5.html
#[derive(Debug)]
#[doc(alias = "SQLITE_ENABLE_FTS5")]
pub struct Fts5;

#[cfg_attr(feature = "inherent", inherent::inherent)]
impl Feature for Fts5 {
    fn is_supported<P: Probe>(&self, probe: &P) -> bool {
        probe.is_set(Flag::EnableFts5)
    }

    fn key(&self) -> FeatureKey {
        FeatureKey::Fts5
    }
}
