use core::alloc::{Layout, LayoutError};

use sqlite::SQLITE_UTF8;
#[cfg(sqlite_has_utf8_zt)]
use sqlite::SQLITE_UTF8_ZT;
#[cfg(feature = "utf-16")]
use sqlite::{SQLITE_UTF16, SQLITE_UTF16BE, SQLITE_UTF16LE};

use crate::ffi;

/// Text encodings [recognized][] by SQLite.
///
/// [recognized]: https://sqlite.org/c3ref/c_any.html
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Encoding {
    #[cfg_attr(not(sqlite_has_utf8_zt), doc = "UTF-8")]
    #[cfg_attr(
        sqlite_has_utf8_zt,
        doc = "UTF-8, optionally known to be [null-terminated](ffi::StringRepresentation::NullTerminated)."
    )]
    Utf8(Option<ffi::StringRepresentation>),

    /// UTF-16, in host-native or fixed [byte order](ByteOrder).
    #[cfg(feature = "utf-16")]
    #[cfg_attr(docsrs, doc(cfg(feature = "utf-16")))]
    Utf16(ByteOrder),
}

impl Encoding {
    pub(crate) const MASK: i32 = {
        let mask = SQLITE_UTF8;

        #[cfg(sqlite_has_utf8_zt)]
        let mask = mask | SQLITE_UTF8_ZT;

        #[cfg(feature = "utf-16")]
        let mask = mask | SQLITE_UTF16 | SQLITE_UTF16BE | SQLITE_UTF16LE;

        mask
    };

    pub const fn from_raw(value: i32) -> Option<Self> {
        let encoding = match value & Self::MASK {
            SQLITE_UTF8 => Self::Utf8(None),
            #[cfg(sqlite_has_utf8_zt)]
            SQLITE_UTF8_ZT => Self::Utf8(Some(ffi::StringRepresentation::NullTerminated)),
            #[cfg(feature = "utf-16")]
            SQLITE_UTF16 => Self::Utf16(ByteOrder::Native),
            #[cfg(feature = "utf-16")]
            SQLITE_UTF16BE => Self::Utf16(ByteOrder::BigEndian),
            #[cfg(feature = "utf-16")]
            SQLITE_UTF16LE => Self::Utf16(ByteOrder::LittleEndian),
            _ => return None,
        };

        Some(encoding)
    }

    pub const fn raw(self) -> i32 {
        match self {
            Self::Utf8(None) => SQLITE_UTF8,
            #[cfg(sqlite_has_utf8_zt)]
            Self::Utf8(Some(ffi::StringRepresentation::NullTerminated)) => SQLITE_UTF8_ZT,
            #[cfg(feature = "utf-16")]
            Self::Utf16(ByteOrder::Native) => SQLITE_UTF16,
            #[cfg(feature = "utf-16")]
            Self::Utf16(ByteOrder::BigEndian) => SQLITE_UTF16BE,
            #[cfg(feature = "utf-16")]
            Self::Utf16(ByteOrder::LittleEndian) => SQLITE_UTF16LE,
        }
    }

    /// The [`Layout::array`] of a string in this encoding.
    pub const fn layout(self, n: usize) -> Result<Layout, LayoutError> {
        match self {
            Self::Utf8(None) => Layout::array::<u8>(n),

            #[cfg(sqlite_has_utf8_zt)]
            Self::Utf8(Some(ffi::StringRepresentation::NullTerminated)) => {
                Layout::array::<u8>(n + 1)
            }

            #[cfg(feature = "utf-16")]
            Self::Utf16(_) => Layout::array::<u16>(n),
        }
    }

    #[cfg(feature = "utf-16")]
    #[cfg_attr(docsrs, doc(cfg(feature = "utf-16")))]
    pub const fn byte_order(self) -> Option<ByteOrder> {
        match self {
            Self::Utf8(_) => None,
            Self::Utf16(order) => Some(order),
        }
    }

    #[cfg_attr(not(sqlite_has_utf8_zt), doc = "`None`.")]
    #[cfg_attr(
        sqlite_has_utf8_zt,
        doc = "The [in-memory representation](ffi::StringRepresentation) of the [`Encoding`], if known."
    )]
    pub const fn representation(self) -> Option<ffi::StringRepresentation> {
        match self {
            Self::Utf8(repr) => repr,
            #[cfg(feature = "utf-16")]
            Self::Utf16(_) => None,
        }
    }
}

impl Default for Encoding {
    fn default() -> Self {
        Self::Utf8(None)
    }
}

#[cfg(feature = "utf-16")]
#[cfg_attr(docsrs, doc(cfg(feature = "utf-16")))]
#[derive(PartialEq, Eq, Default, Clone, Copy, Debug)]
pub enum ByteOrder {
    #[default]
    Native,
    BigEndian,
    LittleEndian,
}
