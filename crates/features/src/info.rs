use crate::directive::{Directive, DirectiveKey, DirectiveMap};
use crate::probe::Probe;
use crate::version::Version;

/// Describes a SQLite library, with known [version](Version) and
/// [compile-time options](Directive).
#[derive(PartialEq, Eq, Clone, Debug)]
#[cfg_attr(docsrs, doc(cfg(any(feature = "std", feature = "alloc"))))]
pub struct Library {
    version: Version,
    directives: DirectiveMap,
}

impl Library {
    pub const fn new(version: Version, directives: DirectiveMap) -> Self {
        Self {
            version,
            directives,
        }
    }

    pub fn probe<P: Probe>(probe: &P) -> Result<Self, P::Error> {
        probe.probe()
    }

    pub const fn version(&self) -> Version {
        self.version
    }

    pub const fn directives(&self) -> &DirectiveMap {
        &self.directives
    }

    pub fn directive(&self, key: DirectiveKey) -> Option<Directive> {
        self.directives.get(key)
    }

    pub fn has_directive(&self, key: DirectiveKey) -> bool {
        self.directives.contains_key(key)
    }

    pub fn into_inner(self) -> (Version, DirectiveMap) {
        (self.version, self.directives)
    }
}

impl<T> From<(Version, T)> for Library
where
    T: Iterator<Item = Directive>,
{
    fn from(value: (Version, T)) -> Self {
        let (version, directives) = value;
        Self::new(version, directives.collect())
    }
}
