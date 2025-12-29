use core::alloc::Layout;

use sqlite::SQLITE_UTF8;
#[cfg(feature = "utf-16")]
use sqlite::{SQLITE_UTF16, SQLITE_UTF16BE, SQLITE_UTF16LE};

#[derive(PartialEq, Eq, Default, Clone, Copy, Debug)]
pub enum Encoding {
    #[default]
    Utf8,
    #[cfg(feature = "utf-16")]
    #[cfg_attr(docsrs, doc(cfg(feature = "utf-16")))]
    Utf16(ByteOrder),
}

impl Encoding {
    #[cfg(not(feature = "utf-16"))]
    pub(crate) const MASK: i32 = SQLITE_UTF8;
    #[cfg(feature = "utf-16")]
    pub(crate) const MASK: i32 = SQLITE_UTF8 | SQLITE_UTF16 | SQLITE_UTF16BE | SQLITE_UTF16LE;

    pub const fn from_raw(value: i32) -> Option<Self> {
        let encoding = match value & Self::MASK {
            SQLITE_UTF8 => Self::Utf8,
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
            Self::Utf8 => SQLITE_UTF8,
            #[cfg(feature = "utf-16")]
            Self::Utf16(ByteOrder::Native) => SQLITE_UTF16,
            #[cfg(feature = "utf-16")]
            Self::Utf16(ByteOrder::BigEndian) => SQLITE_UTF16BE,
            #[cfg(feature = "utf-16")]
            Self::Utf16(ByteOrder::LittleEndian) => SQLITE_UTF16LE,
        }
    }

    pub const fn layout(self) -> Layout {
        match self {
            Encoding::Utf8 => Layout::new::<u8>(),
            #[cfg(feature = "utf-16")]
            Self::Utf16(_) => Layout::new::<u16>(),
        }
    }

    #[cfg(feature = "utf-16")]
    #[cfg_attr(docsrs, doc(cfg(feature = "utf-16")))]
    pub const fn byte_order(self) -> Option<ByteOrder> {
        match self {
            Self::Utf16(order) => Some(order),
            Self::Utf8 => None,
        }
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
