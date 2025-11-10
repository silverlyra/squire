use super::{Feature, FeatureKey};
use crate::probe::{Flag, Probe};

/// Detects support for [BLOB literals][blob] (e.g., `X'0123456789ABCDEF'`).
///
/// [blob]: https://sqlite.org/lang_expr.html#hexint
#[derive(Debug)]
#[doc(alias = "SQLITE_OMIT_BLOB_LITERAL")]
pub struct BlobIo;

#[cfg_attr(feature = "inherent", inherent::inherent)]
impl Feature for BlobIo {
    fn is_supported<P: Probe>(&self, probe: &P) -> bool {
        !probe.is_set(Flag::OmitBlobIo)
    }

    fn key(&self) -> FeatureKey {
        FeatureKey::BlobIo
    }
}

/// Detects support for [LIKE optimization][like] on BLOB columns.
///
/// [like]: https://sqlite.org/optoverview.html#like_opt
#[derive(Debug)]
#[doc(alias = "SQLITE_OMIT_LIKE_OPTIMIZATION")]
pub struct BlobLike;

#[cfg_attr(feature = "inherent", inherent::inherent)]
impl Feature for BlobLike {
    fn is_supported<P: Probe>(&self, probe: &P) -> bool {
        !probe.is_set(Flag::OmitBlobLike)
    }

    fn key(&self) -> FeatureKey {
        FeatureKey::BlobLike
    }
}
