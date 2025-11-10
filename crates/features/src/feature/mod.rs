#![allow(dead_code)]

use core::{error::Error, fmt, str::FromStr};

use crate::probe::Probe;

pub mod api;
pub mod blob;
pub mod database;
pub mod extension;
pub mod fts;
pub mod hooks;
pub mod json;
pub mod metadata;
pub mod sql;
pub mod statistics;
pub mod storage;
pub mod text;

/// A SQLite feature whose support can be [probed](Probe) based on the library
/// version and/or compile-time [flags](crate::Flag).
pub trait Feature {
    /// Check if this [`Feature`] is supported by the [probed](Probe) library.
    fn is_supported<P: Probe>(&self, probe: &P) -> bool;

    /// The [key](FeatureKey) distinguishing this feature from others.
    fn key(&self) -> FeatureKey;
}

/// Identifies each detectable SQLite [`Feature`].
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Copy, Clone, Debug)]
pub enum FeatureKey {
    ApiArmor,
    Attach,
    AuthorizationCallback,
    AutomaticInitialize,
    AutomaticReset,
    BlobIo,
    BlobLike,
    CaseSensitiveLike,
    ColumnDeclaredType,
    ColumnMetadata,
    Complete,
    Deprecated,
    ErrorOffset,
    Fts3,
    Fts5,
    GetTable,
    Json,
    Jsonb,
    LoadExtension,
    MemoryDatabases,
    MemoryManagement,
    NormalizeSql,
    PreUpdateHook,
    PrepareQuiet,
    ProgressCallback,
    Serialize,
    Session,
    SharedCache,
    Snapshot,
    Soundex,
    Stat4,
    TclVariables,
    TemporaryDatabase,
    Trace,
    Utf16,
}

impl FeatureKey {
    /// Returns an iterator over all [feature keys](FeatureKey).
    pub fn all() -> impl Iterator<Item = Self> {
        use FeatureKey::*;
        [
            ApiArmor,
            Attach,
            AuthorizationCallback,
            AutomaticInitialize,
            AutomaticReset,
            BlobIo,
            BlobLike,
            CaseSensitiveLike,
            ColumnDeclaredType,
            ColumnMetadata,
            Complete,
            Deprecated,
            ErrorOffset,
            Fts3,
            Fts5,
            GetTable,
            Json,
            Jsonb,
            LoadExtension,
            MemoryDatabases,
            MemoryManagement,
            NormalizeSql,
            PreUpdateHook,
            PrepareQuiet,
            ProgressCallback,
            Serialize,
            Session,
            SharedCache,
            Snapshot,
            Soundex,
            Stat4,
            TclVariables,
            TemporaryDatabase,
            Trace,
            Utf16,
        ]
        .into_iter()
    }

    /// [Probe] for supported [feature keys](FeatureKey).
    pub fn supported<P: Probe>(probe: &P) -> impl Iterator<Item = Self> {
        Self::all().filter(|key| key.is_supported(probe))
    }

    /// [Probe] if this [`FeatureKey`] is supported.
    pub fn is_supported<P: Probe>(&self, probe: &P) -> bool {
        match *self {
            FeatureKey::ApiArmor => api::ApiArmor.is_supported(probe),
            FeatureKey::Attach => database::Attach.is_supported(probe),
            FeatureKey::AuthorizationCallback => hooks::AuthorizationCallback.is_supported(probe),
            FeatureKey::AutomaticInitialize => api::AutomaticInitialize.is_supported(probe),
            FeatureKey::AutomaticReset => api::AutomaticReset.is_supported(probe),
            FeatureKey::BlobIo => blob::BlobIo.is_supported(probe),
            FeatureKey::BlobLike => blob::BlobLike.is_supported(probe),
            FeatureKey::CaseSensitiveLike => text::CaseSensitiveLike.is_supported(probe),
            FeatureKey::ColumnDeclaredType => metadata::ColumnDeclaredType.is_supported(probe),
            FeatureKey::ColumnMetadata => metadata::ColumnMetadata.is_supported(probe),
            FeatureKey::Complete => api::Complete.is_supported(probe),
            FeatureKey::Deprecated => api::Deprecated.is_supported(probe),
            FeatureKey::ErrorOffset => api::ErrorOffset.is_supported(probe),
            FeatureKey::Fts3 => fts::Fts3.is_supported(probe),
            FeatureKey::Fts5 => fts::Fts5.is_supported(probe),
            FeatureKey::GetTable => api::GetTable.is_supported(probe),
            FeatureKey::Json => json::Json.is_supported(probe),
            FeatureKey::Jsonb => json::Jsonb.is_supported(probe),
            FeatureKey::LoadExtension => extension::LoadExtension.is_supported(probe),
            FeatureKey::MemoryDatabases => storage::MemoryDatabases.is_supported(probe),
            FeatureKey::MemoryManagement => api::MemoryManagement.is_supported(probe),
            FeatureKey::NormalizeSql => sql::NormalizeSql.is_supported(probe),
            FeatureKey::PreUpdateHook => hooks::PreUpdateHook.is_supported(probe),
            FeatureKey::PrepareQuiet => sql::PrepareQuiet.is_supported(probe),
            FeatureKey::ProgressCallback => hooks::ProgressCallback.is_supported(probe),
            FeatureKey::Serialize => storage::Serialize.is_supported(probe),
            FeatureKey::Session => extension::Session.is_supported(probe),
            FeatureKey::SharedCache => database::SharedCache.is_supported(probe),
            FeatureKey::Snapshot => storage::Snapshot.is_supported(probe),
            FeatureKey::Soundex => text::Soundex.is_supported(probe),
            FeatureKey::Stat4 => statistics::Stat4.is_supported(probe),
            FeatureKey::TclVariables => sql::TclVariables.is_supported(probe),
            FeatureKey::TemporaryDatabase => database::TemporaryDatabase.is_supported(probe),
            FeatureKey::Trace => api::Trace.is_supported(probe),
            FeatureKey::Utf16 => text::Utf16.is_supported(probe),
        }
    }

    /// Returns a `snake_case` representation of this feature key.
    ///
    /// ```rust
    /// # use squire_sqlite3_features::FeatureKey;
    /// assert_eq!(FeatureKey::LoadExtension.as_str(), "load_extension");
    /// ```
    pub const fn as_str(&self) -> &'static str {
        match *self {
            FeatureKey::ApiArmor => "api_armor",
            FeatureKey::Attach => "attach",
            FeatureKey::AuthorizationCallback => "authorization_callback",
            FeatureKey::AutomaticInitialize => "automatic_initialize",
            FeatureKey::AutomaticReset => "automatic_reset",
            FeatureKey::BlobIo => "blob_io",
            FeatureKey::BlobLike => "blob_like",
            FeatureKey::CaseSensitiveLike => "case_sensitive_like",
            FeatureKey::ColumnDeclaredType => "column_declared_type",
            FeatureKey::ColumnMetadata => "column_metadata",
            FeatureKey::Complete => "complete",
            FeatureKey::Deprecated => "deprecated",
            FeatureKey::ErrorOffset => "error_offset",
            FeatureKey::Fts3 => "fts3",
            FeatureKey::Fts5 => "fts5",
            FeatureKey::GetTable => "get_table",
            FeatureKey::Json => "json",
            FeatureKey::Jsonb => "jsonb",
            FeatureKey::LoadExtension => "load_extension",
            FeatureKey::MemoryDatabases => "memory_databases",
            FeatureKey::MemoryManagement => "memory_management",
            FeatureKey::NormalizeSql => "normalize_sql",
            FeatureKey::PreUpdateHook => "pre_update_hook",
            FeatureKey::PrepareQuiet => "prepare_quiet",
            FeatureKey::ProgressCallback => "progress_callback",
            FeatureKey::Serialize => "serialize",
            FeatureKey::Session => "session",
            FeatureKey::SharedCache => "shared_cache",
            FeatureKey::Snapshot => "snapshot",
            FeatureKey::Soundex => "soundex",
            FeatureKey::Stat4 => "stat4",
            FeatureKey::TclVariables => "tcl_variables",
            FeatureKey::TemporaryDatabase => "temporary_database",
            FeatureKey::Trace => "trace",
            FeatureKey::Utf16 => "utf16",
        }
    }
}

impl FromStr for FeatureKey {
    type Err = UnknownFeature;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "api_armor" => Ok(FeatureKey::ApiArmor),
            "attach" => Ok(FeatureKey::Attach),
            "authorization_callback" => Ok(FeatureKey::AuthorizationCallback),
            "automatic_initialize" => Ok(FeatureKey::AutomaticInitialize),
            "automatic_reset" => Ok(FeatureKey::AutomaticReset),
            "blob_io" => Ok(FeatureKey::BlobIo),
            "blob_like" => Ok(FeatureKey::BlobLike),
            "case_sensitive_like" => Ok(FeatureKey::CaseSensitiveLike),
            "column_declared_type" => Ok(FeatureKey::ColumnDeclaredType),
            "column_metadata" => Ok(FeatureKey::ColumnMetadata),
            "complete" => Ok(FeatureKey::Complete),
            "deprecated" => Ok(FeatureKey::Deprecated),
            "error_offset" => Ok(FeatureKey::ErrorOffset),
            "fts3" => Ok(FeatureKey::Fts3),
            "fts5" => Ok(FeatureKey::Fts5),
            "get_table" => Ok(FeatureKey::GetTable),
            "json" => Ok(FeatureKey::Json),
            "jsonb" => Ok(FeatureKey::Jsonb),
            "load_extension" => Ok(FeatureKey::LoadExtension),
            "memory_databases" => Ok(FeatureKey::MemoryDatabases),
            "memory_management" => Ok(FeatureKey::MemoryManagement),
            "normalize_sql" => Ok(FeatureKey::NormalizeSql),
            "pre_update_hook" => Ok(FeatureKey::PreUpdateHook),
            "prepare_quiet" => Ok(FeatureKey::PrepareQuiet),
            "progress_callback" => Ok(FeatureKey::ProgressCallback),
            "serialize" => Ok(FeatureKey::Serialize),
            "session" => Ok(FeatureKey::Session),
            "shared_cache" => Ok(FeatureKey::SharedCache),
            "snapshot" => Ok(FeatureKey::Snapshot),
            "soundex" => Ok(FeatureKey::Soundex),
            "stat4" => Ok(FeatureKey::Stat4),
            "tcl_variables" => Ok(FeatureKey::TclVariables),
            "temporary_database" => Ok(FeatureKey::TemporaryDatabase),
            "trace" => Ok(FeatureKey::Trace),
            "utf16" => Ok(FeatureKey::Utf16),
            _ => Err(UnknownFeature),
        }
    }
}

/// Error returned when parsing an unknown feature key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnknownFeature;

impl fmt::Display for UnknownFeature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown feature key")
    }
}

impl Error for UnknownFeature {}
