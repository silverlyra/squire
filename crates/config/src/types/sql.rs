#![allow(clippy::similar_names)]

use core::fmt;

#[cfg(feature = "build")]
use crate::build;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Default, Debug)]
pub struct DoubleQuotedStrings {
    pub in_ddl: bool,
    pub in_dml: bool,
}

impl DoubleQuotedStrings {
    #[inline]
    const fn new(in_ddl: bool, in_dml: bool) -> Self {
        Self { in_ddl, in_dml }
    }

    #[must_use]
    pub const fn disabled() -> Self {
        Self::new(false, false)
    }

    #[must_use]
    pub const fn enabled() -> Self {
        Self::new(true, true)
    }

    #[must_use]
    pub const fn value(self) -> i32 {
        ((self.in_ddl as i32) << 1) | (self.in_dml as i32)
    }

    #[must_use]
    pub const fn from_value(value: i32) -> Option<Self> {
        if value & !0b11 != 0 {
            return None;
        }

        let in_ddl = value & 0b01 != 0;
        let in_dml = value & 0b10 != 0;

        Some(Self { in_ddl, in_dml })
    }
}

impl fmt::Display for DoubleQuotedStrings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match *self {
            Self {
                in_ddl: false,
                in_dml: false,
            } => "disabled",
            Self {
                in_ddl: true,
                in_dml: false,
            } => "DDL only",
            Self {
                in_ddl: false,
                in_dml: true,
            } => "DML only",
            Self {
                in_ddl: true,
                in_dml: true,
            } => "enabled",
        })
    }
}

#[cfg(feature = "build")]
#[cfg_attr(docsrs, doc(cfg(feature = "build")))]
impl build::DirectiveValue for DoubleQuotedStrings {
    fn parse_value(key: build::DirectiveKey, s: Option<&str>) -> Result<Self, build::Error> {
        let s = s.ok_or_else(|| build::Error::value(key, build::ValueError::Missing))?;
        let value: i32 = s.parse().map_err(|_| build::Error::invalid(key))?;
        Self::from_value(value).ok_or_else(|| build::Error::invalid(key))
    }

    fn write_value(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value())
    }
}
