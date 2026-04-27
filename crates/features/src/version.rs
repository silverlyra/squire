use core::{error::Error, ffi::c_int, fmt, str::FromStr};

/// A version of the SQLite library.
///
/// ```rust
/// use squire_sqlite3_features::Version;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let version: Version = "3.50.4".parse()?;
/// assert!(version < Version::new(3, 51, 0));
/// assert!(version == Version::from_number(3050004));
/// assert!(version == (3, 50, 4).into());
/// # Ok(())
/// # }
/// ```
#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Debug)]
pub struct Version {
    pub major: usize,
    pub minor: usize,
    pub patch: usize,
}

impl Version {
    const MAJOR_MAGNITUDE: usize = 1_000_000;
    const MINOR_MAGNITUDE: usize = 1_000;

    /// Create a [`Version`] from `major`.`minor`.`patch`.
    pub const fn new(major: usize, minor: usize, patch: usize) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    /// Create a [`Version`] from `major`.`minor`.`0`.
    #[inline]
    pub const fn release(major: usize, minor: usize) -> Self {
        Self::new(major, minor, 0)
    }

    /// Create a [`Version`] from [`SQLITE_VERSION_NUMBER`][versions].
    ///
    /// [versions]: https://sqlite.org/c3ref/c_scm_branch.html
    pub const fn from_number(num: c_int) -> Self {
        let num = num as usize;
        let (major, num) = (num / Self::MAJOR_MAGNITUDE, num % Self::MAJOR_MAGNITUDE);
        let (minor, patch) = (num / Self::MINOR_MAGNITUDE, num % Self::MINOR_MAGNITUDE);

        Self::new(major, minor, patch)
    }

    /// Convert the version to a SQLite version number.
    pub const fn to_number(&self) -> c_int {
        (self.major * Self::MAJOR_MAGNITUDE + self.minor * Self::MINOR_MAGNITUDE + self.patch)
            as c_int
    }

    /// Parse a version string (`major.minor[.patch]`) as a constant expression.
    ///
    /// Patch defaults to `0` when omitted.
    pub const fn parse(s: &str) -> Result<Self, ParseVersionError> {
        let b = s.as_bytes();

        let dot1 = match find_dot(b, 0) {
            Some(i) => i,
            None => return Err(ParseVersionError),
        };
        let major = match parse_uint(b, 0, dot1) {
            Ok(n) => n,
            Err(e) => return Err(e),
        };

        match find_dot(b, dot1 + 1) {
            Some(dot2) => {
                let minor = match parse_uint(b, dot1 + 1, dot2) {
                    Ok(n) => n,
                    Err(e) => return Err(e),
                };
                let patch = match parse_uint(b, dot2 + 1, b.len()) {
                    Ok(n) => n,
                    Err(e) => return Err(e),
                };
                Ok(Self::new(major, minor, patch))
            }
            None => {
                let minor = match parse_uint(b, dot1 + 1, b.len()) {
                    Ok(n) => n,
                    Err(e) => return Err(e),
                };
                Ok(Self::new(major, minor, 0))
            }
        }
    }

    pub(crate) const fn declare(s: &str) -> Self {
        match Self::parse(s) {
            Ok(s) => s,
            Err(_) => panic!("invalid SQLite version"),
        }
    }
}

const fn find_dot(bytes: &[u8], start: usize) -> Option<usize> {
    let mut i = start;
    while i < bytes.len() {
        if bytes[i] == b'.' {
            return Some(i);
        }
        i += 1;
    }
    None
}

const fn parse_uint(bytes: &[u8], start: usize, end: usize) -> Result<usize, ParseVersionError> {
    if start >= end {
        return Err(ParseVersionError);
    }
    let mut result: usize = 0;
    let mut i = start;
    while i < end {
        let b = bytes[i];
        if b < b'0' || b > b'9' {
            return Err(ParseVersionError);
        }
        result = result * 10 + (b - b'0') as usize;
        i += 1;
    }
    Ok(result)
}

macro_rules! from {
    ($t:ty) => {
        impl From<($t, $t, $t)> for Version {
            fn from((major, minor, patch): ($t, $t, $t)) -> Self {
                Self::new(major as usize, minor as usize, patch as usize)
            }
        }

        impl From<[$t; 3]> for Version {
            fn from([major, minor, patch]: [$t; 3]) -> Self {
                Self::new(major as usize, minor as usize, patch as usize)
            }
        }

        impl From<Version> for ($t, $t, $t) {
            fn from(version: Version) -> Self {
                (version.major as $t, version.minor as $t, version.patch as $t)
            }
        }

        impl From<Version> for [$t; 3] {
            fn from(version: Version) -> Self {
                [version.major as $t, version.minor as $t, version.patch as $t]
            }
        }
    };

    ($th:ty, $($tt:ty),*) => {
        from!($th);
        from!($($tt),*);
    }
}

from!(i8, u8, i16, u16, i32, u32, i64, u64, isize, usize);

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// The `Err` returned when a [`Version`] cannot be [parsed](FromStr).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParseVersionError;

impl Error for ParseVersionError {}

impl fmt::Display for ParseVersionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid SQLite version")
    }
}

impl FromStr for Version {
    type Err = ParseVersionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let version = Version::new(3, 50, 4);
        assert_eq!(version.major, 3);
        assert_eq!(version.minor, 50);
        assert_eq!(version.patch, 4);
    }

    #[test]
    fn test_from_number() {
        let version = Version::from_number(3050004);
        assert_eq!(version, Version::new(3, 50, 4));

        let version = Version::from_number(3046000);
        assert_eq!(version, Version::new(3, 46, 0));

        let version = Version::from_number(3000001);
        assert_eq!(version, Version::new(3, 0, 1));
    }

    #[test]
    fn test_to_number() {
        assert_eq!(Version::new(3, 50, 4).to_number(), 3050004);
        assert_eq!(Version::new(3, 46, 0).to_number(), 3046000);
        assert_eq!(Version::new(3, 0, 1).to_number(), 3000001);
    }

    #[test]
    fn test_roundtrip() {
        let version = Version::new(3, 50, 4);
        assert_eq!(Version::from_number(version.to_number()), version);

        let num = 3046000;
        assert_eq!(Version::from_number(num).to_number(), num);
    }

    #[test]
    fn test_from_str() {
        assert_eq!("3.50.4".parse::<Version>().unwrap(), Version::new(3, 50, 4));
        assert_eq!("3.46.0".parse::<Version>().unwrap(), Version::new(3, 46, 0));
        assert_eq!("0.0.1".parse::<Version>().unwrap(), Version::new(0, 0, 1));
        // patch is optional; omitted patch defaults to 0
        assert_eq!("3.50".parse::<Version>().unwrap(), Version::new(3, 50, 0));
        assert_eq!("3.46".parse::<Version>().unwrap(), Version::new(3, 46, 0));
    }

    #[test]
    fn test_from_str_errors() {
        assert!("3.50.4.1".parse::<Version>().is_err());
        assert!("3".parse::<Version>().is_err());
        assert!("".parse::<Version>().is_err());
        assert!("a.b.c".parse::<Version>().is_err());
        assert!("3.50.".parse::<Version>().is_err());
        assert!(".3.50.4".parse::<Version>().is_err());
    }

    #[test]
    fn test_parse_const() {
        const V: Version = match Version::parse("3.50.4") {
            Ok(v) => v,
            Err(_) => panic!("parse failed"),
        };
        assert_eq!(V, Version::new(3, 50, 4));

        const V2: Version = match Version::parse("3.50") {
            Ok(v) => v,
            Err(_) => panic!("parse failed"),
        };
        assert_eq!(V2, Version::new(3, 50, 0));
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_display() {
        #[cfg(feature = "alloc")]
        use alloc::string::ToString;

        assert_eq!(Version::new(3, 50, 4).to_string(), "3.50.4");
        assert_eq!(Version::new(3, 46, 0).to_string(), "3.46.0");
        assert_eq!(Version::new(0, 0, 1).to_string(), "0.0.1");
    }

    #[test]
    fn test_from_tuple() {
        let version: Version = (3, 50, 4).into();
        assert_eq!(version, Version::new(3, 50, 4));
    }

    #[test]
    fn test_from_array() {
        let version: Version = [3, 50, 4].into();
        assert_eq!(version, Version::new(3, 50, 4));
    }

    #[test]
    fn test_ordering() {
        assert!(Version::new(3, 50, 4) < Version::new(3, 51, 0));
        assert!(Version::new(3, 50, 4) < Version::new(4, 0, 0));
        assert!(Version::new(3, 50, 3) < Version::new(3, 50, 4));
        assert!(Version::new(3, 50, 4) == Version::new(3, 50, 4));
        assert!(Version::new(3, 51, 0) > Version::new(3, 50, 4));
    }

    #[test]
    fn test_copy() {
        let mut v1 = Version::new(3, 50, 4);
        let v2 = v1;
        assert_eq!(v1, v2);

        v1.minor = 51;
        v1.patch = 4;
        assert_ne!(v1, v2);
    }
}
