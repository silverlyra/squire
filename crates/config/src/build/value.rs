use core::fmt;

use super::{DirectiveKey, Error, ValueError};

/// A type which can appear as the value of a [`Directive`](super::Directive).
pub trait DirectiveValue: Copy + fmt::Debug + PartialEq {
    /// Parse the value of the C macro declaration.
    ///
    /// # Errors
    ///
    /// Returns [`Value`](Error::Value) if `s` is not valid for the value type.
    fn parse_value(key: DirectiveKey, s: Option<&str>) -> Result<Self, Error>;

    /// [Write](fmt::Formatter::write_str) this [`DirectiveValue`] as it would
    /// appear in a C macro declaration.
    #[allow(clippy::missing_errors_doc)]
    fn write_value(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;
}

impl DirectiveValue for () {
    fn parse_value(key: DirectiveKey, s: Option<&str>) -> Result<Self, Error> {
        if s.is_none() {
            Ok(())
        } else {
            Err(Error::value(key, ValueError::Unexpected))
        }
    }

    fn write_value(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}

impl DirectiveValue for bool {
    fn parse_value(key: DirectiveKey, s: Option<&str>) -> Result<Self, Error> {
        match s {
            Some("0") => Ok(false),
            Some("1") => Ok(true),
            Some(_) => Err(Error::invalid(key)),
            None => Err(Error::value(key, ValueError::Missing)),
        }
    }

    fn write_value(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(if *self { "1" } else { "0" })
    }
}

impl DirectiveValue for isize {
    fn parse_value(key: DirectiveKey, s: Option<&str>) -> Result<Self, Error> {
        let Some(s) = s else {
            return Err(Error::value(key, ValueError::Missing));
        };

        s.parse().map_err(|_| Error::invalid(key))
    }

    fn write_value(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl DirectiveValue for usize {
    fn parse_value(key: DirectiveKey, s: Option<&str>) -> Result<Self, Error> {
        let Some(s) = s else {
            return Err(Error::value(key, ValueError::Missing));
        };

        let parsed = match s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
            Some(hex) => usize::from_str_radix(hex, 16),
            None => s.parse(),
        };
        parsed.map_err(|_| Error::invalid(key))
    }

    fn write_value(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}
