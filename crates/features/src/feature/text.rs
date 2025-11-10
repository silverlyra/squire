use super::{Feature, FeatureKey};
use crate::probe::{Flag, Probe};

/// Detects whether the [LIKE operator][like] is case-sensitive.
///
/// [like]: https://sqlite.org/lang_expr.html#like
#[derive(Debug)]
#[doc(alias = "SQLITE_CASE_SENSITIVE_LIKE")]
pub struct CaseSensitiveLike;

#[cfg_attr(feature = "inherent", inherent::inherent)]
impl Feature for CaseSensitiveLike {
    fn is_supported<P: Probe>(&self, probe: &P) -> bool {
        probe.is_set(Flag::EnableCaseSensitiveLike)
    }

    fn key(&self) -> FeatureKey {
        FeatureKey::CaseSensitiveLike
    }
}

/// Detects support for the [SOUNDEX()][soundex] SQL function.
///
/// [soundex]: https://sqlite.org/lang_corefunc.html#soundex
#[derive(Debug)]
#[doc(alias = "SQLITE_SOUNDEX")]
pub struct Soundex;

#[cfg_attr(feature = "inherent", inherent::inherent)]
impl Feature for Soundex {
    fn is_supported<P: Probe>(&self, probe: &P) -> bool {
        probe.is_set(Flag::EnableSoundex)
    }

    fn key(&self) -> FeatureKey {
        FeatureKey::Soundex
    }
}

/// Detects support for functions which accept or return UTF-16 text
/// (instead of UTF-8).
#[derive(Debug)]
#[doc(alias = "SQLITE_OMIT_UTF16")]
pub struct Utf16;

#[cfg_attr(feature = "inherent", inherent::inherent)]
impl Feature for Utf16 {
    fn is_supported<P: Probe>(&self, probe: &P) -> bool {
        !probe.is_set(Flag::OmitUtf16)
    }

    fn key(&self) -> FeatureKey {
        FeatureKey::Utf16
    }
}
