//! [Compile-time options][compile] for SQLite.
//!
//! [compile]: https://sqlite.org/compile.html

use core::fmt;

use crate::types;

mod declare;
mod map;
mod value;

use declare::directives;
use map::DirectiveKeys;
pub use value::DirectiveValue;

/// A [compile-time option][compile] for SQLite.
///
/// [compile]: https://sqlite.org/compile.html
pub trait Directive: Copy + From<Self::Value> + fmt::Debug + fmt::Display + PartialEq {
    /// The name of this [compile-time option][compile],
    /// starting with `SQLITE_`.
    ///
    /// [compile]: https://sqlite.org/compile.html
    const NAME: &str;

    /// The name of this [compile-time option][compile],
    /// without the `SQLITE_` prefix.
    ///
    /// [compile]: https://sqlite.org/compile.html
    const BASE_NAME: &str;

    /// The [key](DirectiveKey) naming the [compile-time option][compile]
    /// this [`Directive`] sets.
    ///
    /// [compile]: https://sqlite.org/compile.html
    const KEY: DirectiveKey;

    /// The type of value carried by this [compile-time option][compile].
    ///
    /// For flags without values (like [`SQLITE_OMIT_JSON`](OmitJson)),
    /// `Value` is `()`.
    ///
    /// [compile]: https://sqlite.org/compile.html
    type Value: DirectiveValue;

    /// The value of this [compile-time option][compile].
    ///
    /// [compile]: https://sqlite.org/compile.html
    fn value(self) -> Self::Value;

    /// Parse this [`Directive`], given its [value](Self::Value).
    ///
    /// Use `parse` to read SQLite’s [compile-time option reflection][].
    ///
    /// If a `=` was present in the input, provide the text after the `=` as
    /// `value`. If there was no `=`, set `value` to `None`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    #[doc = concat!("use ", env!("CARGO_CRATE_NAME"), "::build::{self, Directive as _};")]
    ///
    /// assert_eq!(
    ///     build::MaxWorkerThreads::parse(Some("4"))?,
    ///     build::MaxWorkerThreads(4),
    /// );
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns any error from [`parse_value`](DirectiveValue::parse_value).
    ///
    /// [compile-time option reflection]: https://sqlite.org/c3ref/compileoption_get.html
    fn parse(value: Option<&str>) -> Result<Self, Error> {
        let value = Self::Value::parse_value(Self::KEY, value)?;
        Ok(Self::from(value))
    }

    /// Retrieve this [`Directive`] out of a [`DirectiveMap`].
    fn get(map: &DirectiveMap) -> Option<Self>;

    /// Update a [`DirectiveMap`] with this [`Directive`].
    fn set(self, map: &mut DirectiveMap) -> Option<Self>;

    /// Remove any value for this [`Directive`] from a [`DirectiveMap`].
    fn unset(map: &mut DirectiveMap) -> Option<Self>;
}

directives! {
    CASE_SENSITIVE_LIKE || case_sensitive_like => LikeOperatorCaseSenstive,
    DEBUG || debug => Debug,
    DEFAULT_AUTOMATIC_INDEX || default_automatic_index => DefaultAutomaticIndex(bool),
    DEFAULT_AUTOVACUUM || default_autovacuum => DefaultAutomaticVacuum(types::AutomaticVacuum),
    DEFAULT_FOREIGN_KEYS || default_foreign_keys => DefaultForeignKeys(bool),
    DEFAULT_MEMSTATUS || default_memstatus => DefaultMemoryStatus(bool),
    DEFAULT_SYNCHRONOUS || default_synchronous => DefaultSynchronous(types::Synchronous),
    DEFAULT_WAL_SYNCHRONOUS || default_wal_synchronous => DefaultWalSynchronous(types::Synchronous),
    DQS || dqs => DoubleQuotedStrings(types::DoubleQuotedStrings),
    ENABLE_API_ARMOR || enable_api_armor => EnableApiArmor,
    ENABLE_COLUMN_METADATA || enable_column_metadata => EnableColumnMetadata,
    ENABLE_DBPAGE_VTAB || enable_dbpage_vtab => EnableDatabasePageVirtualTable,
    ENABLE_DBSTAT_VTAB || enable_dbstat_vtab => EnableDatabaseStatisticsVirtualTable,
    ENABLE_FTS3 || enable_fts3 => EnableFts3,
    ENABLE_FTS5 || enable_fts5 => EnableFts5,
    ENABLE_GEOPOLY || enable_geopoly => EnableGeopoly,
    ENABLE_JSON1 || enable_json1 => EnableJson1,
    ENABLE_MEMORY_MANAGEMENT || enable_memory_management => EnableMemoryManagement,
    ENABLE_NORMALIZE || enable_normalize => EnableNormalizeSql,
    ENABLE_PERCENTILE || enable_percentile => EnablePercentile,
    ENABLE_PREUPDATE_HOOK || enable_preupdate_hook => EnablePreUpdateHook,
    ENABLE_RTREE || enable_rtree => EnableRtree,
    ENABLE_SESSION || enable_session => EnableSession,
    ENABLE_SNAPSHOT || enable_snapshot => EnableSnapshot,
    ENABLE_STAT4 || enable_stat4 => EnableStat4,
    JSON_MAX_DEPTH || json_max_depth => MaxJsonDepth(usize),
    LIKE_DOESNT_MATCH_BLOBS || like_doesnt_match_blobs => LikeOperatorDoesntMatchBlob,
    MAX_ATTACHED || max_attached => MaxAttachedDatabases(usize),
    MAX_COLUMN || max_column => MaxColumns(usize),
    MAX_DEFAULT_PAGE_SIZE || max_default_page_size => MaxDefaultPageSize(usize),
    MAX_EXPR_DEPTH || max_expr_depth => MaxExpressionDepth(usize),
    MAX_LENGTH || max_length => MaxLength(usize),
    MAX_MMAP_SIZE || max_mmap_size => MaxMmapSize(usize),
    MAX_PAGE_COUNT || max_page_count => MaxPageCount(usize),
    MAX_PAGE_SIZE || max_page_size => MaxPageSize(usize),
    MAX_VARIABLE_NUMBER || max_variable_number => MaxVariables(usize),
    MAX_WORKER_THREADS || max_worker_threads => MaxWorkerThreads(usize),
    OMIT_ATTACH || omit_attach => OmitAttach,
    OMIT_AUTHORIZATION || omit_authorization => OmitAuthorization,
    OMIT_AUTOINIT || omit_autoinit => OmitAutomaticInitialize,
    OMIT_AUTORESET || omit_autoreset => OmitAutomaticReset,
    OMIT_BLOB_LITERAL || omit_blob_literal => OmitBlobLiteral,
    OMIT_COMPLETE || omit_complete => OmitComplete,
    OMIT_DECLTYPE || omit_decltype => OmitColumnDeclaredType,
    OMIT_DEPRECATED || omit_deprecated => OmitDeprecated,
    OMIT_DESERIALIZE || omit_deserialize => OmitSerialize,
    OMIT_GET_TABLE || omit_get_table => OmitGetTable,
    OMIT_INCRBLOB || omit_incrblob => OmitBlobIo,
    OMIT_JSON || omit_json => OmitJson,
    OMIT_LIKE_OPTIMIZATION || omit_like_optimization => OmitLikeOptimization,
    OMIT_LOAD_EXTENSION || omit_load_extension => OmitLoadExtension,
    OMIT_MEMORYDB || omit_memorydb => OmitMemoryDatabases,
    OMIT_PROGRESS_CALLBACK || omit_progress_callback => OmitProgressCallback,
    OMIT_SHARED_CACHE || omit_shared_cache => OmitSharedCache,
    OMIT_TCL_VARIABLE || omit_tcl_variable => OmitTclVariables,
    OMIT_TEMPDB || omit_tempdb => OmitTemporaryDatabase,
    OMIT_TRACE || omit_trace => OmitTrace,
    OMIT_UTF16 || omit_utf16 => OmitUtf16,
    OMIT_VIRTUALTABLE || omit_virtualtable => OmitVirtualTable,
    OMIT_WAL || omit_wal => OmitWal,
    SECURE_DELETE || secure_delete => SecureDelete,
    SOUNDEX || soundex => EnableSoundex,
    TEMP_STORE || temp_store => TemporaryStorage(types::TemporaryStorage),
    THREADSAFE || threadsafe => Threading(types::Threading),
    USE_ALLOCA || use_alloca => UseAlloca,
    USE_URI || use_uri => UseDatabaseUri,
}

#[cfg_attr(
    feature = "std",
    doc = "An [error](std::error::Error) parsing a [`Directive`]."
)]
#[cfg_attr(
    not(feature = "std"),
    doc = "An [error](core::error::Error) parsing a [`Directive`]."
)]
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Error {
    /// The name didn’t match any known [`DirectiveKey`].
    Unknown,
    /// An [error](ValueError) with a [`Directive`]’s [value](Directive::Value).
    Value {
        key: DirectiveKey,
        reason: ValueError,
    },
}

impl Error {
    pub(crate) const fn value(key: DirectiveKey, reason: ValueError) -> Self {
        Self::Value { key, reason }
    }

    pub(crate) const fn invalid(key: DirectiveKey) -> Self {
        Self::value(key, ValueError::Invalid)
    }

    #[must_use]
    pub const fn is_unknown(self) -> bool {
        matches!(self, Self::Unknown)
    }

    #[must_use]
    pub const fn key(self) -> Option<DirectiveKey> {
        match self {
            Error::Unknown => None,
            Error::Value { key, .. } => Some(key),
        }
    }

    #[must_use]
    pub const fn value_error(self) -> Option<ValueError> {
        match self {
            Error::Unknown => None,
            Error::Value { reason, .. } => Some(reason),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unknown => f.write_str("unknown directive"),
            Self::Value { key, reason } => write!(f, "{reason} for {key}"),
        }
    }
}

impl core::error::Error for Error {}

/// The reason for an [`Error`] parsing a known [`DirectiveKey`].
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum ValueError {
    /// The [`Directive`] needs a [value](Directive::Value), but none was present.
    Missing,
    /// The [`Directive`] has no [value](Directive::Value), but one was present.
    Unexpected,
    /// The [`Directive`]’s [value](Directive::Value) could not be parsed.
    Invalid,
}

impl fmt::Display for ValueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing => f.write_str("missing value"),
            Self::Unexpected => f.write_str("unexpected value"),
            Self::Invalid => f.write_str("invalid directive value"),
        }
    }
}
