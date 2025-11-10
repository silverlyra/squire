use super::{Feature, FeatureKey};
use crate::probe::{Flag, Probe};

/// Detects support for [database snapshots][snapshot].
///
/// [snapshot]: https://sqlite.org/c3ref/snapshot.html
#[derive(Debug)]
#[doc(alias = "SQLITE_ENABLE_SNAPSHOT")]
pub struct Snapshot;

#[cfg_attr(feature = "inherent", inherent::inherent)]
impl Feature for Snapshot {
    fn is_supported<P: Probe>(&self, probe: &P) -> bool {
        probe.is_set(Flag::EnableSnapshot)
    }

    fn key(&self) -> FeatureKey {
        FeatureKey::Snapshot
    }
}

/// Detects support for [database serialization][serialize] via `sqlite3_serialize()` and `sqlite3_deserialize()`.
///
/// [serialize]: https://sqlite.org/c3ref/serialize.html
#[derive(Debug)]
#[doc(alias = "SQLITE_OMIT_DESERIALIZE")]
pub struct Serialize;

#[cfg_attr(feature = "inherent", inherent::inherent)]
impl Feature for Serialize {
    fn is_supported<P: Probe>(&self, probe: &P) -> bool {
        !probe.is_set(Flag::OmitSerialize)
    }

    fn key(&self) -> FeatureKey {
        FeatureKey::Serialize
    }
}

/// Detects support for [in-memory databases][inmem].
///
/// [inmem]: https://sqlite.org/inmemorydb.html
#[derive(Debug)]
#[doc(alias = "SQLITE_OMIT_MEMORYDB")]
pub struct MemoryDatabases;

#[cfg_attr(feature = "inherent", inherent::inherent)]
impl Feature for MemoryDatabases {
    fn is_supported<P: Probe>(&self, probe: &P) -> bool {
        !probe.is_set(Flag::OmitMemoryDatabases)
    }

    fn key(&self) -> FeatureKey {
        FeatureKey::MemoryDatabases
    }
}
