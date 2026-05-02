use core::fmt;

#[cfg(feature = "build")]
use crate::build;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum TemporaryStorage {
    Filesystem { always: bool },
    Memory { always: bool },
}

impl TemporaryStorage {
    #[must_use]
    pub const fn is_always(self) -> bool {
        match self {
            TemporaryStorage::Filesystem { always } | TemporaryStorage::Memory { always } => always,
        }
    }

    #[must_use]
    pub const fn is_filesystem(self) -> bool {
        matches!(self, Self::Filesystem { .. })
    }

    #[must_use]
    pub const fn is_memory(self) -> bool {
        matches!(self, Self::Memory { .. })
    }
}

impl Default for TemporaryStorage {
    fn default() -> Self {
        Self::Filesystem { always: false }
    }
}

impl fmt::Display for TemporaryStorage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            TemporaryStorage::Filesystem { always: false } => "default filesystem",
            TemporaryStorage::Filesystem { always: true } => "always filesystem",
            TemporaryStorage::Memory { always: false } => "default memory",
            TemporaryStorage::Memory { always: true } => "always memory",
        })
    }
}

#[cfg(feature = "build")]
#[cfg_attr(docsrs, doc(cfg(feature = "build")))]
impl build::DirectiveValue for TemporaryStorage {
    fn parse_value(key: build::DirectiveKey, s: Option<&str>) -> Result<Self, build::Error> {
        let s = s.ok_or_else(|| build::Error::value(key, build::ValueError::Missing))?;
        let value: i32 = s.parse().map_err(|_| build::Error::invalid(key))?;

        match value {
            0 => Ok(Self::Filesystem { always: true }),
            1 => Ok(Self::Filesystem { always: false }),
            2 => Ok(Self::Memory { always: false }),
            3 => Ok(Self::Memory { always: true }),
            _ => Err(build::Error::invalid(key)),
        }
    }

    fn write_value(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            TemporaryStorage::Filesystem { always: true } => "0",
            TemporaryStorage::Filesystem { always: false } => "1",
            TemporaryStorage::Memory { always: false } => "2",
            TemporaryStorage::Memory { always: true } => "3",
        })
    }
}
