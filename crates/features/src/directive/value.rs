use core::fmt;

use super::{DirectiveKey, ParseDirectiveError};

/// Parsing and formatting for the `=value` suffix of a
/// [`Directive`](super::Directive).
///
/// Used instead of [`core::str::FromStr`] / [`core::fmt::Display`] so that:
/// - parse errors carry the actual [`DirectiveKey`] of the directive being
///   parsed, rather than a single hardcoded key per value type, and
/// - the formatted form can differ from the value type's natural `Display`
///   (e.g. [`bool`] formats as `0`/`1` rather than `false`/`true`).
pub trait DirectiveValue: Sized {
    fn parse_value(key: DirectiveKey, s: &str) -> Result<Self, ParseDirectiveError>;

    fn write_value(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;
}

impl DirectiveValue for bool {
    fn parse_value(key: DirectiveKey, s: &str) -> Result<Self, ParseDirectiveError> {
        match s {
            "0" => Ok(false),
            "1" => Ok(true),
            _ => Err(ParseDirectiveError::InvalidValue(key)),
        }
    }

    fn write_value(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(if *self { "1" } else { "0" })
    }
}

impl DirectiveValue for usize {
    fn parse_value(key: DirectiveKey, s: &str) -> Result<Self, ParseDirectiveError> {
        let parsed = match s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
            Some(hex) => usize::from_str_radix(hex, 16),
            None => s.parse(),
        };
        parsed.map_err(|_| ParseDirectiveError::InvalidValue(key))
    }

    fn write_value(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

macro_rules! impl_directive_value_enum {
    ($t:ty) => {
        impl DirectiveValue for $t {
            fn parse_value(key: DirectiveKey, s: &str) -> Result<Self, ParseDirectiveError> {
                let n: i32 = s
                    .parse()
                    .map_err(|_| ParseDirectiveError::InvalidValue(key))?;
                Self::from_value(n).ok_or(ParseDirectiveError::InvalidValue(key))
            }

            fn write_value(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt::Display::fmt(&self.value(), f)
            }
        }
    };
}

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

    pub const fn disabled() -> Self {
        Self::new(false, false)
    }

    pub const fn enabled() -> Self {
        Self::new(true, true)
    }

    pub const fn value(&self) -> i32 {
        ((self.in_ddl as i32) << 1) | (self.in_dml as i32)
    }

    pub const fn from_value(value: i32) -> Option<Self> {
        if value & !0b11 != 0 {
            return None;
        }

        let in_ddl = value & 0b01 != 0;
        let in_dml = value & 0b10 != 0;

        Some(Self { in_ddl, in_dml })
    }
}

impl_directive_value_enum!(DoubleQuotedStrings);

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Default, Debug)]
#[repr(u32)]
pub enum TemporaryStorage {
    AlwaysFilesystem = 0,
    #[default]
    DefaultFilesystem = 1,
    DefaultMemory = 2,
    AlwaysMemory = 3,
}

impl TemporaryStorage {
    pub const fn value(&self) -> i32 {
        *self as i32
    }

    pub const fn from_value(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::AlwaysFilesystem),
            1 => Some(Self::DefaultFilesystem),
            2 => Some(Self::DefaultMemory),
            3 => Some(Self::AlwaysMemory),
            _ => None,
        }
    }
}

impl_directive_value_enum!(TemporaryStorage);

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Default, Debug)]
#[repr(u32)]
pub enum Threading {
    #[default]
    SingleThread = 0,
    MultiThread = 1,
    Serialized = 2,
}

impl Threading {
    pub const fn is_single_threaded(&self) -> bool {
        matches!(*self, Self::SingleThread)
    }

    pub const fn value(&self) -> i32 {
        *self as i32
    }

    pub const fn from_value(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::SingleThread),
            1 => Some(Self::MultiThread),
            2 => Some(Self::Serialized),
            _ => None,
        }
    }
}

impl_directive_value_enum!(Threading);

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Default, Debug)]
#[repr(u32)]
pub enum Synchronous {
    Off = 0,
    Normal = 1,
    #[default]
    Full = 2,
    Extra = 3,
}

impl Synchronous {
    pub const fn value(&self) -> i32 {
        *self as i32
    }

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

impl_directive_value_enum!(Synchronous);
