#[cfg(feature = "alloc")]
use alloc::string::{String, ToString};
use core::{error::Error, fmt, str::FromStr};

/// A compile-time flag which enables or omits a SQLite feature.
#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub enum Flag {
    EnableApiArmor,
    EnableCaseSensitiveLike,
    EnableColumnMetadata,
    EnableFts3,
    EnableFts5,
    EnableJson,
    EnableMemoryManagement,
    EnableNormalizeSql,
    EnablePreUpdateHook,
    EnableProgressCallback,
    EnableSession,
    EnableSnapshot,
    EnableSoundex,
    EnableStat4,
    OmitAttach,
    OmitAuthorization,
    OmitAutomaticInitialize,
    OmitAutomaticReset,
    OmitBlobIo,
    OmitBlobLike,
    OmitColumnDeclaredType,
    OmitComplete,
    OmitDeprecated,
    OmitGetTable,
    OmitJson,
    OmitLoadExtension,
    OmitMemoryDatabases,
    OmitSerialize,
    OmitSharedCache,
    OmitTclVariables,
    OmitTemporaryDatabase,
    OmitTrace,
    OmitUtf16,
}

impl Flag {
    const PREFIX: &str = "SQLITE_";

    /// Returns the name of the compile-time option this `Flag` corresponds to.
    ///
    /// ```rust
    /// # use squire_sqlite3_features::Flag;
    /// assert_eq!(Flag::EnableJson.name(), "SQLITE_ENABLE_JSON1");
    /// assert_eq!(Flag::OmitJson.name(), "SQLITE_OMIT_JSON");
    /// ```
    pub const fn name(&self) -> &'static str {
        match *self {
            Flag::EnableApiArmor => "SQLITE_ENABLE_API_ARMOR",
            Flag::EnableCaseSensitiveLike => "SQLITE_CASE_SENSITIVE_LIKE",
            Flag::EnableColumnMetadata => "SQLITE_ENABLE_COLUMN_METADATA",
            Flag::EnableFts3 => "SQLITE_ENABLE_FTS3",
            Flag::EnableFts5 => "SQLITE_ENABLE_FTS5",
            Flag::EnableJson => "SQLITE_ENABLE_JSON1",
            Flag::EnableMemoryManagement => "SQLITE_ENABLE_MEMORY_MANAGEMENT",
            Flag::EnableNormalizeSql => "SQLITE_ENABLE_NORMALIZE",
            Flag::EnablePreUpdateHook => "SQLITE_ENABLE_PREUPDATE_HOOK",
            Flag::EnableProgressCallback => "SQLITE_ENABLE_PROGRESS_CALLBACK",
            Flag::EnableSession => "SQLITE_ENABLE_SESSION",
            Flag::EnableSnapshot => "SQLITE_ENABLE_SNAPSHOT",
            Flag::EnableSoundex => "SQLITE_SOUNDEX",
            Flag::EnableStat4 => "SQLITE_ENABLE_STAT4",
            Flag::OmitAttach => "SQLITE_OMIT_ATTACH",
            Flag::OmitAuthorization => "SQLITE_OMIT_AUTHORIZATION",
            Flag::OmitAutomaticInitialize => "SQLITE_OMIT_AUTOINIT",
            Flag::OmitAutomaticReset => "SQLITE_OMIT_AUTORESET",
            Flag::OmitBlobIo => "SQLITE_OMIT_BLOB_LITERAL",
            Flag::OmitBlobLike => "SQLITE_OMIT_LIKE_OPTIMIZATION",
            Flag::OmitColumnDeclaredType => "SQLITE_OMIT_DECLTYPE",
            Flag::OmitComplete => "SQLITE_OMIT_COMPLETE",
            Flag::OmitDeprecated => "SQLITE_OMIT_DEPRECATED",
            Flag::OmitGetTable => "SQLITE_OMIT_GET_TABLE",
            Flag::OmitJson => "SQLITE_OMIT_JSON",
            Flag::OmitLoadExtension => "SQLITE_OMIT_LOAD_EXTENSION",
            Flag::OmitMemoryDatabases => "SQLITE_OMIT_MEMORYDB",
            Flag::OmitSerialize => "SQLITE_OMIT_DESERIALIZE",
            Flag::OmitSharedCache => "SQLITE_OMIT_SHARED_CACHE",
            Flag::OmitTclVariables => "SQLITE_OMIT_TCL_VARIABLE",
            Flag::OmitTemporaryDatabase => "SQLITE_OMIT_TEMPDB",
            Flag::OmitTrace => "SQLITE_OMIT_TRACE",
            Flag::OmitUtf16 => "SQLITE_OMIT_UTF16",
        }
    }

    /// The [`name`](Self::name) of the flag, without the `SQLITE_` prefix.
    pub fn base_name(&self) -> &'static str {
        let name = self.name();
        &name[Self::PREFIX.len()..]
    }

    /// Parses a flag name into a `Flag`.
    ///
    /// This method accepts flag names with or without the `SQLITE_` prefix.
    ///
    /// ```rust
    /// # use squire_sqlite3_features::Flag;
    /// assert_eq!(Flag::of("SQLITE_ENABLE_JSON1"), Some(Flag::EnableJson));
    /// assert_eq!(Flag::of("ENABLE_JSON1"), Some(Flag::EnableJson));
    /// assert_eq!(Flag::of("OMIT_JSON"), Some(Flag::OmitJson));
    /// assert_eq!(Flag::of("INVALID_FLAG"), None);
    /// ```
    pub fn of(s: impl AsRef<str>) -> Option<Self> {
        let s = s.as_ref();
        let normalized = s.strip_prefix(Self::PREFIX).unwrap_or(s);

        match normalized {
            "ENABLE_API_ARMOR" => Some(Flag::EnableApiArmor),
            "CASE_SENSITIVE_LIKE" => Some(Flag::EnableCaseSensitiveLike),
            "ENABLE_COLUMN_METADATA" => Some(Flag::EnableColumnMetadata),
            "ENABLE_FTS3" => Some(Flag::EnableFts3),
            "ENABLE_FTS5" => Some(Flag::EnableFts5),
            "ENABLE_JSON1" => Some(Flag::EnableJson),
            "ENABLE_MEMORY_MANAGEMENT" => Some(Flag::EnableMemoryManagement),
            "ENABLE_NORMALIZE" => Some(Flag::EnableNormalizeSql),
            "ENABLE_PREUPDATE_HOOK" => Some(Flag::EnablePreUpdateHook),
            "ENABLE_PROGRESS_CALLBACK" => Some(Flag::EnableProgressCallback),
            "ENABLE_SESSION" => Some(Flag::EnableSession),
            "ENABLE_SNAPSHOT" => Some(Flag::EnableSnapshot),
            "SOUNDEX" => Some(Flag::EnableSoundex),
            "ENABLE_STAT4" => Some(Flag::EnableStat4),
            "OMIT_ATTACH" => Some(Flag::OmitAttach),
            "OMIT_AUTHORIZATION" => Some(Flag::OmitAuthorization),
            "OMIT_AUTOINIT" => Some(Flag::OmitAutomaticInitialize),
            "OMIT_AUTORESET" => Some(Flag::OmitAutomaticReset),
            "OMIT_BLOB_LITERAL" => Some(Flag::OmitBlobIo),
            "OMIT_LIKE_OPTIMIZATION" => Some(Flag::OmitBlobLike),
            "OMIT_DECLTYPE" => Some(Flag::OmitColumnDeclaredType),
            "OMIT_COMPLETE" => Some(Flag::OmitComplete),
            "OMIT_DEPRECATED" => Some(Flag::OmitDeprecated),
            "OMIT_GET_TABLE" => Some(Flag::OmitGetTable),
            "OMIT_JSON" => Some(Flag::OmitJson),
            "OMIT_LOAD_EXTENSION" => Some(Flag::OmitLoadExtension),
            "OMIT_MEMORYDB" => Some(Flag::OmitMemoryDatabases),
            "OMIT_DESERIALIZE" => Some(Flag::OmitSerialize),
            "OMIT_SHARED_CACHE" => Some(Flag::OmitSharedCache),
            "OMIT_TCL_VARIABLE" => Some(Flag::OmitTclVariables),
            "OMIT_TEMPDB" => Some(Flag::OmitTemporaryDatabase),
            "OMIT_TRACE" => Some(Flag::OmitTrace),
            "OMIT_UTF16" => Some(Flag::OmitUtf16),
            _ => None,
        }
    }
}

impl fmt::Display for Flag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseFlagError {
    #[cfg(feature = "alloc")]
    input: String,
}

impl fmt::Display for ParseFlagError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[cfg(feature = "alloc")]
        write!(f, "unknown SQLite flag: {}", self.input)?;
        #[cfg(not(feature = "alloc"))]
        write!(f, "unknown SQLite flag")?;

        Ok(())
    }
}

impl Error for ParseFlagError {}

impl FromStr for Flag {
    type Err = ParseFlagError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Flag::of(s).ok_or_else(|| ParseFlagError {
            #[cfg(feature = "alloc")]
            input: s.to_string(),
        })
    }
}
