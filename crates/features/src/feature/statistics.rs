use super::{Feature, FeatureKey};
use crate::probe::{Flag, Probe};

/// Detects support for enhanced query planner [statistics][stat4] via STAT4 tables.
///
/// [stat4]: https://sqlite.org/fileformat2.html#stat4tab
#[derive(Debug)]
#[doc(alias = "SQLITE_ENABLE_STAT4")]
pub struct Stat4;

#[cfg_attr(feature = "inherent", inherent::inherent)]
impl Feature for Stat4 {
    fn is_supported<P: Probe>(&self, probe: &P) -> bool {
        probe.is_set(Flag::EnableStat4)
    }

    fn key(&self) -> FeatureKey {
        FeatureKey::Stat4
    }
}
