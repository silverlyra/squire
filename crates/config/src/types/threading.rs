use core::fmt;

#[cfg(feature = "build")]
use crate::build;

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Default, Debug)]
pub enum Threading {
    #[default]
    SingleThread = 0,
    MultiThread = 1,
    Serialized = 2,
}

impl Threading {
    #[must_use]
    pub const fn is_single_threaded(self) -> bool {
        matches!(self, Self::SingleThread)
    }

    #[must_use]
    pub const fn value(self) -> i32 {
        self as i32
    }

    #[must_use]
    pub const fn from_value(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::SingleThread),
            1 => Some(Self::MultiThread),
            2 => Some(Self::Serialized),
            _ => None,
        }
    }
}

impl fmt::Display for Threading {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match *self {
            Self::SingleThread => "single-threaded",
            Self::MultiThread => "multi-threaded",
            Self::Serialized => "serialized",
        })
    }
}

#[cfg(feature = "build")]
#[cfg_attr(docsrs, doc(cfg(feature = "build")))]
impl build::DirectiveValue for Threading {
    fn parse_value(key: build::DirectiveKey, s: Option<&str>) -> Result<Self, build::Error> {
        let s = s.ok_or_else(|| build::Error::value(key, build::ValueError::Missing))?;
        let value: i32 = s.parse().map_err(|_| build::Error::invalid(key))?;
        Self::from_value(value).ok_or_else(|| build::Error::invalid(key))
    }

    fn write_value(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value())
    }
}
