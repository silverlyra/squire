#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg), deny(rustdoc::broken_intra_doc_links))]

#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std as alloc;

#[cfg(feature = "metadata")]
use core::{error::Error, fmt};
#[cfg(feature = "std")]
use std::collections::BTreeSet;

mod feature;
mod probe;
mod version;

pub use feature::{
    Feature, FeatureKey,
    api::{
        ApiArmor, AutomaticInitialize, AutomaticReset, Complete, Deprecated, ErrorOffset, GetTable,
        MemoryManagement, Trace,
    },
    blob::{BlobIo, BlobLike},
    database::{Attach, SharedCache, TemporaryDatabase},
    extension::{LoadExtension, Session},
    fts::{Fts3, Fts5},
    hooks::{AuthorizationCallback, PreUpdateHook, ProgressCallback},
    json::{Json, Jsonb},
    metadata::{ColumnDeclaredType, ColumnMetadata},
    sql::{NormalizeSql, PrepareQuiet, TclVariables},
    statistics::Stat4,
    storage::{MemoryDatabases, Serialize, Snapshot},
    text::{CaseSensitiveLike, Soundex, Utf16},
};
#[cfg(feature = "build")]
#[cfg_attr(docsrs, doc(cfg(feature = "build")))]
pub use probe::build;
#[cfg(feature = "dynamic")]
#[cfg_attr(docsrs, doc(cfg(feature = "dynamic")))]
pub use probe::dynamic;
pub use probe::{Flag, Probe, Threading};
pub use version::Version;

/// A fully-[feature probed](Probe) SQLite library.
#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
pub struct Library {
    version: Version,
    threading: Threading,
    features: BTreeSet<FeatureKey>,
}

#[cfg(feature = "std")]
impl Library {
    /// Populate a [`Library`].
    pub fn new(
        version: Version,
        threading: Threading,
        features: impl Iterator<Item = FeatureKey>,
    ) -> Self {
        Self {
            version,
            threading,
            features: features.collect(),
        }
    }

    /// Populate a [`Library`] from a [`Probe`].
    pub fn probe<P: Probe>(probe: &P) -> Self {
        Self::new(
            probe.version(),
            probe.threading(),
            FeatureKey::supported(probe),
        )
    }

    /// The [`Version`] of the SQLite [library](Library).
    pub const fn version(&self) -> Version {
        self.version
    }

    /// The [thread safety](Threading) of the SQLite [library](Library).
    pub const fn threading(&self) -> Threading {
        self.threading
    }

    /// The SQLite [feature keys](FeatureKey) supported by this [library](Library).
    pub fn features(&self) -> impl Iterator<Item = FeatureKey> {
        self.features.iter().copied()
    }

    /// `true` if the [feature key](FeatureKey) is in this [library](Library)’s
    /// supported SQLite features.
    pub fn is_supported<T>(&self, key: FeatureKey) -> bool {
        self.features.contains(&key)
    }

    #[cfg(feature = "metadata")]
    #[cfg_attr(docsrs, doc(cfg(feature = "metadata")))]
    pub fn emit_cargo_metadata(&self) {
        println!("cargo:version={}", self.version);
        println!("cargo:threading={}", self.threading.as_str());

        let features: Vec<_> = self.features.iter().map(|k| k.as_str()).collect();
        println!("cargo:features={}", features.join(","));
    }

    #[cfg(feature = "metadata")]
    #[cfg_attr(docsrs, doc(cfg(feature = "metadata")))]
    pub fn emit_cfg(&self) {
        // Register all possible feature configs
        for key in FeatureKey::all() {
            let name = key.as_str();
            println!("cargo:rustc-check-cfg=cfg(sqlite_has_{name})");
        }

        // Enable configs for supported features
        for key in &self.features {
            let name = key.as_str();
            println!("cargo:rustc-cfg=sqlite_has_{name}");
        }

        // Include SQLite library version
        println!("cargo::rustc-env=SQUIRE_SQLITE_VERSION={}", self.version());
    }

    #[cfg(feature = "metadata")]
    #[cfg_attr(docsrs, doc(cfg(feature = "metadata")))]
    pub fn from_cargo_metadata() -> Result<Self, MetadataError> {
        use std::env;

        let version = env::var("DEP_SQLITE3_VERSION")
            .map_err(|_| MetadataError::MissingVersion)?
            .parse()
            .map_err(|_| MetadataError::InvalidVersion)?;

        let threading: Threading = env::var("DEP_SQLITE3_THREADING")
            .map_err(|_| MetadataError::MissingThreading)?
            .parse()
            .map_err(|_| MetadataError::InvalidThreading)?;

        let features_str =
            env::var("DEP_SQLITE3_FEATURES").map_err(|_| MetadataError::MissingFeatures)?;

        let features = if features_str.is_empty() {
            BTreeSet::new()
        } else {
            features_str
                .split(',')
                .map(|s| s.parse().map_err(|_| MetadataError::InvalidFeature))
                .collect::<Result<BTreeSet<_>, _>>()?
        };

        Ok(Self {
            version,
            threading,
            features,
        })
    }
}

#[cfg(feature = "metadata")]
#[cfg_attr(docsrs, doc(cfg(feature = "metadata")))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetadataError {
    MissingVersion,
    InvalidVersion,
    MissingThreading,
    InvalidThreading,
    MissingFeatures,
    InvalidFeature,
}

#[cfg(feature = "metadata")]
impl fmt::Display for MetadataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MetadataError::MissingVersion => write!(f, "missing DEP_SQLITE3_VERSION"),
            MetadataError::InvalidVersion => write!(f, "invalid version format"),
            MetadataError::MissingThreading => write!(f, "missing DEP_SQLITE3_THREADING"),
            MetadataError::InvalidThreading => write!(f, "invalid threading mode"),
            MetadataError::MissingFeatures => write!(f, "missing DEP_SQLITE3_FEATURES"),
            MetadataError::InvalidFeature => write!(f, "invalid feature key"),
        }
    }
}

#[cfg(feature = "metadata")]
impl Error for MetadataError {}
