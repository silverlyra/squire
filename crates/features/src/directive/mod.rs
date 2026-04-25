mod declare;
#[cfg(feature = "alloc")]
mod map;
mod value;

use core::{error::Error, fmt};

use declare::*;
#[cfg(feature = "alloc")]
pub use map::DirectiveMap;
pub use value::{DirectiveValue, DoubleQuotedStrings, Synchronous, TemporaryStorage, Threading};

directives! {
    CASE_SENSITIVE_LIKE => LikeOperatorCaseSenstive,
    DEBUG => Debug,
    DEFAULT_AUTOMATIC_INDEX => DefaultAutomaticIndex,
    DEFAULT_AUTOVACUUM => DefaultAutomaticVacuum,
    DEFAULT_FOREIGN_KEYS => DefaultForeignKeys,
    DEFAULT_MEMSTATUS => DefaultMemoryStatus(bool),
    DEFAULT_SYNCHRONOUS => DefaultSynchronous(Synchronous),
    DEFAULT_WAL_SYNCHRONOUS => DefaultWalSynchronous(Synchronous),
    DQS => DoubleQuotedStrings(DoubleQuotedStrings),
    ENABLE_API_ARMOR => EnableApiArmor,
    ENABLE_COLUMN_METADATA => EnableColumnMetadata,
    ENABLE_DBPAGE_VTAB => EnableDatabasePageVirtualTable,
    ENABLE_DBSTAT_VTAB => EnableDatabaseStatisticsVirtualTable,
    ENABLE_FTS3 => EnableFts3,
    ENABLE_FTS5 => EnableFts5,
    ENABLE_GEOPOLY => EnableGeopoly,
    ENABLE_JSON1 => EnableJson1,
    ENABLE_MEMORY_MANAGEMENT => EnableMemoryManagement,
    ENABLE_NORMALIZE => EnableNormalizeSql,
    ENABLE_PREUPDATE_HOOK => EnablePreUpdateHook,
    ENABLE_RTREE => EnableRtree,
    ENABLE_SESSION => EnableSession,
    ENABLE_SNAPSHOT => EnableSnapshot,
    ENABLE_STAT4 => EnableStat4,
    JSON_MAX_DEPTH => MaxJsonDepth(usize),
    LIKE_DOESNT_MATCH_BLOBS => LikeOperatorDoesntMatchBlob,
    MAX_ATTACHED => MaxAttachedDatabases(usize),
    MAX_COLUMN => MaxColumns(usize),
    MAX_DEFAULT_PAGE_SIZE => MaxDefaultPageSize(usize),
    MAX_EXPR_DEPTH => MaxExpressionDepth(usize),
    MAX_LENGTH => MaxLength(usize),
    MAX_MMAP_SIZE => MaxMmapSize(usize),
    MAX_PAGE_COUNT => MaxPageCount(usize),
    MAX_PAGE_SIZE => MaxPageSize(usize),
    MAX_VARIABLE_NUMBER => MaxVariables(usize),
    MAX_WORKER_THREADS => MaxWorkerThreads(usize),
    OMIT_ATTACH => OmitAttach,
    OMIT_AUTHORIZATION => OmitAuthorization,
    OMIT_AUTOINIT => OmitAutomaticInitialize,
    OMIT_AUTORESET => OmitAutomaticReset,
    OMIT_BLOB_LITERAL => OmitBlobLiteral,
    OMIT_COMPLETE => OmitComplete,
    OMIT_DECLTYPE => OmitColumnDeclaredType,
    OMIT_DEPRECATED => OmitDeprecated,
    OMIT_DESERIALIZE => OmitSerialize,
    OMIT_GET_TABLE => OmitGetTable,
    OMIT_INCRBLOB => OmitBlobIo,
    OMIT_JSON => OmitJson,
    OMIT_LIKE_OPTIMIZATION => OmitBlobLike,
    OMIT_LOAD_EXTENSION => OmitLoadExtension,
    OMIT_MEMORYDB => OmitMemoryDatabases,
    OMIT_PROGRESS_CALLBACK => OmitProgressCallback,
    OMIT_SHARED_CACHE => OmitSharedCache,
    OMIT_TCL_VARIABLE => OmitTclVariables,
    OMIT_TEMPDB => OmitTemporaryDatabase,
    OMIT_TRACE => OmitTrace,
    OMIT_UTF16 => OmitUtf16,
    OMIT_VIRTUALTABLE => OmitVirtualTable,
    OMIT_WAL => OmitWal,
    SECURE_DELETE => SecureDelete,
    SOUNDEX => EnableSoundex,
    TEMP_STORE => TemporaryStorage(TemporaryStorage),
    THREADSAFE => Threading(Threading),
    USE_ALLOCA => UseAlloca,
    USE_URI => UseDatabaseUri,
}

impl Ord for DirectiveKey {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.base_name().cmp(other.base_name())
    }
}

impl PartialOrd for DirectiveKey {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Error returned when parsing a [`Directive`] from a `PRAGMA compile_options`
/// style line fails.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum ParseDirectiveError {
    /// The directive name didn't match any known compile-time option.
    UnknownKey,
    /// The directive expects a `=value` suffix but none was provided.
    MissingValue(DirectiveKey),
    /// The directive is a flag (no value expected) but a `=value` was given.
    UnexpectedValue(DirectiveKey),
    /// The directive's value was present but could not be parsed.
    InvalidValue(DirectiveKey),
}

impl fmt::Display for ParseDirectiveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownKey => f.write_str("unknown directive key"),
            Self::MissingValue(key) => write!(f, "missing directive value for {key:?}"),
            Self::UnexpectedValue(key) => write!(f, "unexpected value for {key:?}"),
            Self::InvalidValue(key) => write!(f, "invalid directive value for {key:?}"),
        }
    }
}

impl Error for ParseDirectiveError {}
