#![allow(dead_code)]

//! Detect SQLite [features](Feature), which either require a particular
//! [version](Version) or [compile-time option](crate::Directive) to be present
//! (or absent).

use core::{error::Error, fmt};

use crate::directive::DirectiveKey;
#[cfg(feature = "alloc")]
use crate::directive::{Directive, DirectiveMap};
use crate::info::Library;
use crate::version::Version;

/// A SQLite feature whose support can be [probed](Probe) based on the library
/// version and/or compile-time [flags](crate::Flag).
pub trait Feature {
    /// Check if this [`Feature`] is available in the [`Library`].
    fn is_available(&self, library: &Library) -> bool;

    /// The [key](FeatureKey) distinguishing this feature from others.
    fn key(&self) -> FeatureKey;

    #[cfg(feature = "alloc")]
    #[cfg_attr(docsrs, doc(cfg(any(feature = "std", feature = "alloc"))))]
    fn apply(
        &self,
        enabled: bool,
        directives: &mut DirectiveMap,
        version: Version,
        config: &Configuration,
    );
}

mod declare;
use declare::features;

mod config;
pub use config::Configuration;

features! {
    AggregateFunctionSelfOrdered @"3.47",
    ApiArmor +EnableApiArmor,
    Attach -OmitAttach,
    AuthorizationCallback -OmitAuthorization,
    AutomaticInitialize -OmitAutomaticInitialize,
    AutomaticReset -OmitAutomaticReset,
    AutomaticVacuum +DefaultAutomaticVacuum,
    BlobIo -OmitBlobIo,
    BlobLike -OmitBlobLike,
    CaseSensitiveLike +LikeOperatorCaseSenstive,
    ColumnDeclaredType -OmitColumnDeclaredType,
    ColumnMetadata +EnableColumnMetadata,
    Complete -OmitComplete,
    DatabasePageVirtualTable +EnableDatabasePageVirtualTable,
    DatabaseStatisticsVirtualTable +EnableDatabaseStatisticsVirtualTable,
    Deprecated -OmitDeprecated,
    ErrorOffset @"3.38",
    Fts3 +EnableFts3,
    Fts5 +EnableFts5,
    FunctionDirectOnlyOption @"3.30",
    FunctionInnocuousOption @"3.31",
    FunctionStrictSubtypes @"3.45",
    FunctionSubtypeOption @"3.31",
    Geopoly +EnableGeopoly,
    GetTable -OmitGetTable,
    Json ?is_enabled,
    Jsonb @"3.45" ^Json,
    LoadExtension -OmitLoadExtension,
    MemoryDatabase -OmitMemoryDatabases,
    MemoryManagement +EnableMemoryManagement,
    MemoryStatus ?is_enabled,
    NormalizeSql +EnableNormalizeSql,
    Percentile @"3.51" +EnablePercentile,
    PreUpdateHook +EnablePreUpdateHook,
    PrepareQuiet @"3.48",
    ProgressCallback -OmitProgressCallback,
    Rtree +EnableRtree,
    Serialize -OmitSerialize,
    Session ^PreUpdateHook +EnableSession,
    SharedCache -OmitSharedCache,
    Snapshot +EnableSnapshot,
    Soundex +EnableSoundex,
    Stat4 +EnableStat4,
    TclVariables -OmitTclVariables,
    TemporaryDatabase -OmitTemporaryDatabase,
    Trace -OmitTrace,
    Utf16 -OmitUtf16,
    VirtualTable -OmitVirtualTable,
    Wal -OmitWal,
}

#[cfg(feature = "metadata")]
impl FeatureKey {
    pub(crate) fn cfg_name(&self) -> String {
        let name = self.name();
        let bytes = name.as_bytes();
        let mut out = String::with_capacity(name.len() + 8);
        for (i, &b) in bytes.iter().enumerate() {
            if b.is_ascii_uppercase() && i > 0 {
                let prev = bytes[i - 1];
                let next = bytes.get(i + 1).copied();
                if prev.is_ascii_lowercase()
                    || prev.is_ascii_digit()
                    || (prev.is_ascii_uppercase() && next.is_some_and(|n| n.is_ascii_lowercase()))
                {
                    out.push('_');
                }
            }
            out.push(b.to_ascii_lowercase() as char);
        }
        out
    }
}

impl Json {
    /// The SQLite version where JSON support became opt-out.
    pub const ENABLED_BY_DEFAULT: Version = Version::release(3, 38);

    fn is_enabled(&self, library: &Library) -> bool {
        if library.version() < Self::ENABLED_BY_DEFAULT {
            library.has_directive(DirectiveKey::EnableJson1)
        } else {
            !library.has_directive(DirectiveKey::OmitJson)
        }
    }

    #[cfg(feature = "alloc")]
    fn apply(&self, enabled: bool, directives: &mut DirectiveMap, version: Version) {
        if version < Self::ENABLED_BY_DEFAULT {
            if enabled {
                directives.insert(Directive::EnableJson1);
            } else {
                directives.remove(DirectiveKey::EnableJson1);
            }
        } else if !enabled {
            directives.insert(Directive::OmitJson);
        } else {
            directives.remove(DirectiveKey::OmitJson);
        }
    }
}

impl MemoryStatus {
    fn is_enabled(&self, library: &Library) -> bool {
        matches!(
            library.directive(DirectiveKey::DefaultMemoryStatus),
            Some(Directive::DefaultMemoryStatus(true))
        )
    }

    #[cfg(feature = "alloc")]
    fn apply(&self, enabled: bool, directives: &mut DirectiveMap, _version: Version) {
        directives.insert(Directive::DefaultMemoryStatus(enabled));
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct UnknownFeatureError(alloc::string::String);

impl fmt::Display for UnknownFeatureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown SQLite feature {:?}", &self.0)
    }
}

impl Error for UnknownFeatureError {}
