use core::fmt;

#[cfg(feature = "build")]
use crate::build;

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Default, Debug)]
pub enum Synchronous {
    Off = 0,
    Normal = 1,
    #[default]
    Full = 2,
    Extra = 3,
}

impl Synchronous {
    #[must_use]
    pub const fn is_on(self) -> bool {
        !matches!(self, Self::Off)
    }

    #[must_use]
    pub const fn value(self) -> i32 {
        self as i32
    }

    #[must_use]
    pub const fn from_value(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::Off),
            1 => Some(Self::Normal),
            2 => Some(Self::Full),
            3 => Some(Self::Extra),
            _ => None,
        }
    }
}

impl fmt::Display for Synchronous {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match *self {
            Self::Off => "off",
            Synchronous::Normal => "normal",
            Synchronous::Full => "full",
            Synchronous::Extra => "extra",
        })
    }
}

#[cfg(feature = "build")]
#[cfg_attr(docsrs, doc(cfg(feature = "build")))]
impl build::DirectiveValue for Synchronous {
    fn parse_value(key: build::DirectiveKey, s: Option<&str>) -> Result<Self, build::Error> {
        let s = s.ok_or_else(|| build::Error::value(key, build::ValueError::Missing))?;
        let value: i32 = s.parse().map_err(|_| build::Error::invalid(key))?;
        Self::from_value(value).ok_or_else(|| build::Error::invalid(key))
    }

    fn write_value(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value())
    }
}
