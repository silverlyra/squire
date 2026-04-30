#![cfg_attr(not(feature = "std"), no_std)]
#![warn(clippy::pedantic)]
#![cfg_attr(docsrs, feature(doc_cfg), deny(rustdoc::broken_intra_doc_links))]
#![doc = concat!("# ", env!("CARGO_PKG_NAME"))]
//!
//! [Squire][] is a crate for embedding [SQLite][] in Rust. This crate
//! represents [versions](Version) of the SQLite library.
//!
//! Users of Squire don't need to interact with this crate directly, and can
//! treat it as an implementation detail.
//!
//! ```rust
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
#![doc = concat!("use ", env!("CARGO_CRATE_NAME"), "::Version;")]
//!
//! let version = Version::parse("3.49.2")?;
//!
//! assert_eq!(version.minor(), 49);
//! assert!(version < Version::new(3, 51, 0)?);
//!
//! assert_eq!(Version::from_number(3_049_002)?, version);
//! # Ok(())
//! # }
//! ```
//!
//! [Squire]: https://lib.rs/squire
//! [SQLite]: https://sqlite.org/
//! [C API]: https://sqlite.org/cintro.html

use core::{ffi::c_int, fmt, num::NonZero, str::FromStr};

/// A version of the SQLite library.
///
/// ```rust
#[doc = concat!("use ", env!("CARGO_CRATE_NAME"), "::Version;")]
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let version: Version = "3.50.4".parse()?;
/// assert!(version < Version::new(3, 51, 0)?);
/// assert!(version == Version::from_number(3_050_004)?);
/// assert!(version == (3, 50, 4).try_into()?);
/// # Ok(())
/// # }
/// ```
///
/// Since SQLite 3.9.0 (2015), [SQLite versions][version] follow [semver][].
///
/// > All SQLite releases starting with 3.9.0 use a three-number "semantic version" \[…].
///
/// > \[The [`major`](Self::major) version] is only increased when there is a
/// > change that breaks backward compatibility. \[… The] SQLite developers plan
/// > to support the current SQLite database file format, SQL syntax, and C
/// > interface through [at least the year 2050][lts]. Hence, one can expect
/// > that all future versions of SQLite for the next several decades will begin
/// > with “`3.`”.
///
/// > \[The [`minor`](Self::minor) version] is incremented for any change that
/// > breaks forward compatibility by adding new features.
///
/// > \[The [`patch`](Self::patch) version] is incremented for releases
/// > consisting of only small changes that implement performance enhancements
/// > and/or bug fixes.
///
/// [version]: https://sqlite.org/versionnumbers.html
/// [semver]: https://semver.org/spec/v1.0.0.html
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Copy, Clone, Debug)]
#[repr(transparent)]
pub struct Version(NonZero<u32>);

impl Version {
    /// Create a [`Version`] from `major`.`minor`.`patch`.
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[doc = concat!("use ", env!("CARGO_CRATE_NAME"), "::Version;")]
    ///
    /// assert_eq!(Version::new(3, 49, 2), "3.49.2".parse());
    ///
    /// assert!(Version::new(0, 0, 0).is_err());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error`] if all components are zero, `major` ≥ 256, or `minor`
    /// or `patch` ≥ 1024. (SQLite's own [numeric version format][versions]
    /// requires `minor` and `patch` ≤ 1000, and the next `major` version isn't
    /// expected until at least [2050][lts]).
    ///
    /// [versions]: https://sqlite.org/c3ref/c_scm_branch.html
    /// [lts]: https://sqlite.org/lts.html
    pub const fn new(major: u32, minor: u32, patch: u32) -> Result<Self, Error> {
        if major >= 1u32 << field::major::WIDTH
            || minor >= 1u32 << field::minor::WIDTH
            || patch >= 1u32 << field::patch::WIDTH
        {
            return Err(Error);
        }

        let value = (major << field::major::SHIFT)
            | (minor << field::minor::SHIFT)
            | (patch << field::patch::SHIFT);

        match NonZero::new(value) {
            Some(value) => Ok(Self(value)),
            None => Err(Error),
        }
    }

    /// Create a [`Version`] from `major`.`minor`.`patch` without checking that
    /// the components are valid.
    ///
    /// # Safety
    ///
    /// Callers must ensure that at least one component is non-zero, and that
    /// `major`, `minor`, or `patch` fit within the bits that `Version` reserves
    /// for them (see [`new`](Self::new)).
    #[must_use]
    pub const unsafe fn new_unchecked(major: u32, minor: u32, patch: u32) -> Self {
        let value = (major << field::major::SHIFT)
            | (minor << field::minor::SHIFT)
            | (patch << field::patch::SHIFT);

        // SAFETY: the caller guarantees that at least one component is
        // non-zero, so the combined value is non-zero.
        Self(unsafe { NonZero::new_unchecked(value) })
    }

    /// Create a [`Version`] from `major`.`minor`.`0`.
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[doc = concat!("use ", env!("CARGO_CRATE_NAME"), "::Version;")]
    ///
    /// assert_eq!(Version::release(3, 53)?, "3.53.0".parse()?);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error`] under the same conditions as [`new`](Self::new).
    #[inline]
    pub const fn release(major: u32, minor: u32) -> Result<Self, Error> {
        Self::new(major, minor, 0)
    }

    #[inline]
    const fn value(self) -> u32 {
        self.0.get()
    }

    /// The SQLite major version [component](Self::components).
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[doc = concat!("use ", env!("CARGO_CRATE_NAME"), "::Version;")]
    ///
    /// let version = Version::parse("3.49.2")?;
    /// assert_eq!(version.major(), 3);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Since SQLite 3.9.0 (2015), [SQLite versions][version] follow [semver][].
    ///
    /// > \[The `major` version] is only increased when there is a change that
    /// > breaks backward compatibility. \[… The] SQLite developers plan to
    /// > support the current SQLite database file format, SQL syntax, and C
    /// > interface through [at least the year 2050][lts]. Hence, one can expect
    /// > that all future versions of SQLite for the next several decades will
    /// > begin with “`3.`”.
    ///
    /// [version]: https://sqlite.org/versionnumbers.html
    /// [semver]: https://semver.org/spec/v1.0.0.html
    /// [lts]: https://sqlite.org/lts.html
    #[must_use]
    pub const fn major(self) -> u32 {
        (self.value() & field::major::MASK) >> field::major::SHIFT
    }

    /// The SQLite minor version [component](Self::components).
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[doc = concat!("use ", env!("CARGO_CRATE_NAME"), "::Version;")]
    ///
    /// let version = Version::parse("3.49.2")?;
    /// assert_eq!(version.minor(), 49);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Since SQLite 3.9.0 (2015), [SQLite versions][version] follow [semver][].
    ///
    /// > \[The `minor` version] is incremented for any change that breaks
    /// > forward compatibility by adding new features.
    ///
    /// [version]: https://sqlite.org/versionnumbers.html
    /// [semver]: https://semver.org/spec/v1.0.0.html
    #[must_use]
    pub const fn minor(self) -> u32 {
        (self.value() & field::minor::MASK) >> field::minor::SHIFT
    }

    /// The SQLite patch version [component](Self::components).
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[doc = concat!("use ", env!("CARGO_CRATE_NAME"), "::Version;")]
    ///
    /// let version = Version::parse("3.49.2")?;
    /// assert_eq!(version.patch(), 2);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Since SQLite 3.9.0 (2015), [SQLite versions][version] follow [semver][].
    ///
    /// > \[The `patch` version] is incremented for releases consisting of only
    /// > small changes that implement performance enhancements and/or bug fixes.
    ///
    /// [version]: https://sqlite.org/versionnumbers.html
    /// [semver]: https://semver.org/spec/v1.0.0.html
    #[must_use]
    pub const fn patch(self) -> u32 {
        (self.value() & field::patch::MASK) >> field::patch::SHIFT
    }

    /// The `(minor, major, patch)` version components.
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[doc = concat!("use ", env!("CARGO_CRATE_NAME"), "::Version;")]
    ///
    /// let version = Version::parse("3.49.2")?;
    /// assert_eq!(version.components(), (3, 49, 2));
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub const fn components(self) -> (u32, u32, u32) {
        (self.major(), self.minor(), self.patch())
    }

    /// Create a [`Version`] from [`SQLITE_VERSION_NUMBER`][versions].
    ///
    /// ```rust
    #[doc = concat!("use ", env!("CARGO_CRATE_NAME"), "::Version;")]
    ///
    /// assert_eq!(
    ///     Version::from_number(3_053_000),
    ///     Version::new(3, 53, 0),
    /// );
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error`] if the resulting components do not fit (see
    /// [`new`](Self::new)).
    ///
    /// [versions]: https://sqlite.org/c3ref/c_scm_branch.html
    pub const fn from_number(num: c_int) -> Result<Self, Error> {
        let num = num.cast_unsigned();

        let (major, num) = (num / field::major::MAGNITUDE, num % field::major::MAGNITUDE);
        let (minor, patch) = (num / field::minor::MAGNITUDE, num % field::minor::MAGNITUDE);

        Self::new(major, minor, patch)
    }

    /// Convert the [`Version`] to [`SQLITE_VERSION_NUMBER`][versions]
    /// representation.
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[doc = concat!("use ", env!("CARGO_CRATE_NAME"), "::Version;")]
    ///
    /// let version = Version::parse("3.49.2")?;
    /// assert_eq!(version.to_number(), 3_049_002);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [versions]: https://sqlite.org/c3ref/c_scm_branch.html
    #[must_use]
    pub const fn to_number(self) -> c_int {
        (self.major() * field::major::MAGNITUDE
            + self.minor() * field::minor::MAGNITUDE
            + self.patch())
        .cast_signed() as c_int
    }

    /// Parse a version string (`major.minor[.patch]`) as a constant expression.
    ///
    /// The [patch version](Self::patch) defaults to `0` when omitted.
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[doc = concat!("use ", env!("CARGO_CRATE_NAME"), "::Version;")]
    ///
    /// assert_eq!(Version::parse("3.51.3")?, Version::new(3, 51, 3).unwrap());
    /// assert_eq!(Version::parse("3.53.0")?, Version::new(3, 53, 0).unwrap());
    ///
    /// assert!(Version::parse("3").is_err());
    /// assert!(Version::parse("3.8.11.1").is_err());
    /// assert!(Version::parse("3.46.1-7+deb13u1").is_err());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error`] if `s` does not match `"x.y.z"` or `"x.y"`, where
    /// `x`, `y`, and `z` are all integers that fit within their respective
    /// component widths. No leading or trailing characters are accepted.
    pub const fn parse(s: &str) -> Result<Self, Error> {
        #[allow(clippy::many_single_char_names)]
        const fn split(s: &str, c: u8) -> (&str, Option<&str>) {
            let b = s.as_bytes();
            let mut i = 0;

            while i < b.len() {
                if b[i] == c {
                    return match s.split_at_checked(i) {
                        Some((a, b)) => (a, Some(b)),
                        None => (s, None),
                    };
                }
                i += 1;
            }

            (s, None)
        }
        const fn part<'a>((a, b): (&'a str, Option<&'a str>)) -> Option<(u32, Option<&'a str>)> {
            if a.is_empty() || a.as_bytes()[0] == b'+' {
                return None; // otherwise, + is accepted by from_str_radix
            }
            let Ok(a) = u32::from_str_radix(a, 10) else {
                return None;
            };

            let b = match b {
                Some(b) => match b.split_at_checked(1) {
                    Some((_, b)) => Some(b),
                    _ => return None,
                },
                None => None,
            };

            Some((a, b))
        }

        let Some((major, Some(s))) = part(split(s, b'.')) else {
            return Err(Error);
        };
        let Some((minor, s)) = part(split(s, b'.')) else {
            return Err(Error);
        };

        let patch = match s {
            Some(s) => {
                if let Some((patch, None)) = part(split(s, b'.')) {
                    patch
                } else {
                    return Err(Error);
                }
            }
            None => 0,
        };

        Self::new(major, minor, patch)
    }

    /// [Parse](Self::parse) a [`Version`] in a `const` context.
    ///
    /// # Panics
    ///
    /// Panics if [`parse(s)`](Self::parse) returns an [`Error`].
    #[must_use]
    pub const fn declare(s: &str) -> Self {
        match Self::parse(s) {
            Ok(s) => s,
            Err(_) => panic!("invalid SQLite version"),
        }
    }
}

impl TryFrom<(u32, u32, u32)> for Version {
    type Error = Error;

    fn try_from((major, minor, patch): (u32, u32, u32)) -> Result<Self, Self::Error> {
        Self::new(major, minor, patch)
    }
}

impl TryFrom<[u32; 3]> for Version {
    type Error = Error;

    fn try_from([major, minor, patch]: [u32; 3]) -> Result<Self, Self::Error> {
        Self::new(major, minor, patch)
    }
}

impl From<Version> for (u32, u32, u32) {
    fn from(version: Version) -> Self {
        version.components()
    }
}

impl From<Version> for [u32; 3] {
    fn from(version: Version) -> Self {
        [version.major(), version.minor(), version.patch()]
    }
}

impl FromStr for Version {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (major, minor, patch) = self.components();
        write!(f, "{major}.{minor}.{patch}")
    }
}

/// The `Err` returned when a [`Version`] cannot be constructed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Error;

impl core::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid SQLite version")
    }
}

#[cfg(feature = "serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
impl serde::Serialize for Version {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

#[cfg(feature = "serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
impl<'de> serde::Deserialize<'de> for Version {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct VersionVisitor;

        impl serde::de::Visitor<'_> for VersionVisitor {
            type Value = Version;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a SQLite version string (e.g. \"3.50.4\")")
            }

            fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
                Version::parse(v).map_err(E::custom)
            }
        }

        deserializer.deserialize_str(VersionVisitor)
    }
}

mod field {
    pub mod major {
        pub const MASK: u32 = 0xFF_000_000;
        pub const SHIFT: u32 = super::minor::SHIFT + super::minor::WIDTH;
        pub const WIDTH: u32 = 8;

        pub const MAGNITUDE: u32 = 1_000_000;
    }
    pub mod minor {
        pub const MASK: u32 = 0x00_FFF_000;
        pub const SHIFT: u32 = super::patch::SHIFT + super::patch::WIDTH;
        pub const WIDTH: u32 = super::patch::WIDTH;

        pub const MAGNITUDE: u32 = 1_000;
    }
    pub mod patch {
        pub const MASK: u32 = 0x00_000_FFF;
        pub const SHIFT: u32 = 0;
        pub const WIDTH: u32 = 12;
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use core::mem::size_of;

    use super::Version;

    #[test]
    fn test_new() {
        let version = Version::new(3, 50, 4).unwrap();
        assert_eq!(version.major(), 3);
        assert_eq!(version.minor(), 50);
        assert_eq!(version.patch(), 4);
    }

    #[test]
    fn test_new_validates_widths() {
        assert!(Version::new(256, 0, 0).is_err());
        assert!(Version::new(0, 4096, 0).is_err());
        assert!(Version::new(0, 0, 4096).is_err());
        assert!(Version::new(255, 4095, 4095).is_ok());
    }

    #[test]
    fn test_new_rejects_zero() {
        assert!(Version::new(0, 0, 0).is_err());
        assert!(Version::new(0, 0, 1).is_ok());
    }

    #[test]
    fn test_size() {
        assert_eq!(size_of::<Version>(), size_of::<u32>());
        assert_eq!(size_of::<Version>(), size_of::<Option<Version>>());
    }

    #[test]
    fn test_from_number() {
        let version = Version::from_number(3_050_004).unwrap();
        assert_eq!(version, Version::new(3, 50, 4).unwrap());

        let version = Version::from_number(3_046_000).unwrap();
        assert_eq!(version, Version::new(3, 46, 0).unwrap());

        let version = Version::from_number(3_000_001).unwrap();
        assert_eq!(version, Version::new(3, 0, 1).unwrap());
    }

    #[test]
    fn test_to_number() {
        assert_eq!(Version::new(3, 50, 4).unwrap().to_number(), 3_050_004);
        assert_eq!(Version::new(3, 46, 0).unwrap().to_number(), 3_046_000);
        assert_eq!(Version::new(3, 0, 1).unwrap().to_number(), 3_000_001);
    }

    #[test]
    fn test_roundtrip() {
        let version = Version::new(3, 50, 4).unwrap();
        assert_eq!(Version::from_number(version.to_number()).unwrap(), version);

        let num = 3_046_000;
        assert_eq!(Version::from_number(num).unwrap().to_number(), num);
    }

    #[test]
    fn test_from_str() {
        assert_eq!(
            "3.50.4".parse::<Version>().unwrap(),
            Version::new(3, 50, 4).unwrap()
        );
        assert_eq!(
            "3.46.0".parse::<Version>().unwrap(),
            Version::new(3, 46, 0).unwrap()
        );
        assert_eq!(
            "0.0.1".parse::<Version>().unwrap(),
            Version::new(0, 0, 1).unwrap()
        );
        assert_eq!(
            "3.50".parse::<Version>().unwrap(),
            Version::new(3, 50, 0).unwrap()
        );
        assert_eq!(
            "3.46".parse::<Version>().unwrap(),
            Version::new(3, 46, 0).unwrap()
        );
    }

    #[test]
    fn test_from_str_errors() {
        assert!("3.50.4.1".parse::<Version>().is_err());
        assert!("3".parse::<Version>().is_err());
        assert!("".parse::<Version>().is_err());
        assert!("a.b.c".parse::<Version>().is_err());
        assert!("3.+49.2".parse::<Version>().is_err());
        assert!(".3.50.4".parse::<Version>().is_err());
        assert!("0.0.0".parse::<Version>().is_err());
        assert!("256.0.0".parse::<Version>().is_err());
    }

    #[test]
    fn test_declare() {
        assert_eq!(Version::declare("3.50.4"), Version::new(3, 50, 4).unwrap());
        assert_eq!(Version::declare("3.50"), Version::new(3, 50, 0).unwrap());
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_display() {
        assert_eq!(Version::new(3, 50, 4).unwrap().to_string(), "3.50.4");
        assert_eq!(Version::new(3, 46, 0).unwrap().to_string(), "3.46.0");
        assert_eq!(Version::new(0, 0, 1).unwrap().to_string(), "0.0.1");
    }

    #[test]
    fn test_try_from_tuple() {
        let version: Version = (3, 50, 4).try_into().unwrap();
        assert_eq!(version, Version::new(3, 50, 4).unwrap());

        assert!(Version::try_from((0, 0, 0)).is_err());
        assert!(Version::try_from((256, 0, 0)).is_err());
    }

    #[test]
    fn test_try_from_array() {
        let version: Version = [3, 50, 4].try_into().unwrap();
        assert_eq!(version, Version::new(3, 50, 4).unwrap());
    }

    #[test]
    fn test_ordering() {
        assert!(Version::new(3, 50, 4).unwrap() < Version::new(3, 51, 0).unwrap());
        assert!(Version::new(3, 50, 4).unwrap() < Version::new(4, 0, 0).unwrap());
        assert!(Version::new(3, 50, 3).unwrap() < Version::new(3, 50, 4).unwrap());
        assert!(Version::new(3, 50, 4).unwrap() == Version::new(3, 50, 4).unwrap());
        assert!(Version::new(3, 51, 0).unwrap() > Version::new(3, 50, 4).unwrap());
    }

    #[test]
    fn test_new_unchecked() {
        // SAFETY: 3, 50, 4 fit in their component widths and are not all zero.
        let version = unsafe { Version::new_unchecked(3, 50, 4) };
        assert_eq!(version, Version::new(3, 50, 4).unwrap());
    }
}
