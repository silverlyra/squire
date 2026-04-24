use super::FeatureKey;
#[cfg(feature = "alloc")]
use crate::directive::DirectiveMap;
use crate::info::Library;
use crate::version::Version;

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct Configuration([Option<bool>; FeatureKey::COUNT]);

impl Configuration {
    pub const fn empty() -> Self {
        Self([None; FeatureKey::COUNT])
    }

    pub fn detect(library: &Library) -> Self {
        use super::Feature as _;

        let mut config = Self::empty();
        for key in FeatureKey::all() {
            config.0[key.offset()] = Some(key.is_available(library));
        }
        config
    }

    pub fn defined(&self) -> impl Iterator<Item = FeatureKey> {
        FeatureKey::all().filter(|key| self.is_defined(*key))
    }

    pub fn enabled(&self) -> impl Iterator<Item = FeatureKey> {
        FeatureKey::all().filter(|key| self.is_enabled(*key))
    }

    #[cfg(feature = "alloc")]
    #[cfg_attr(docsrs, doc(cfg(any(feature = "std", feature = "alloc"))))]
    pub fn directives(&self, version: Version) -> DirectiveMap {
        let mut directives = DirectiveMap::new();
        self.apply(version, &mut directives);

        directives
    }

    #[cfg(feature = "alloc")]
    #[cfg_attr(docsrs, doc(cfg(any(feature = "std", feature = "alloc"))))]
    pub fn apply(&self, version: Version, directives: &mut DirectiveMap) {
        use super::Feature as _;

        for key in self.defined() {
            key.apply(self.is_enabled(key), directives, version, self);
        }
    }

    #[inline]
    pub fn is_defined(&self, key: FeatureKey) -> bool {
        self.0[key.offset()].is_some()
    }

    #[inline]
    pub fn is_enabled(&self, key: FeatureKey) -> bool {
        self.0[key.offset()] == Some(true)
    }

    #[inline]
    pub fn is_disabled(&self, key: FeatureKey) -> bool {
        self.0[key.offset()] == Some(false)
    }

    pub fn set_enabled(&mut self, key: FeatureKey, enabled: bool) {
        self.0[key.offset()] = Some(enabled);
    }
}

impl FromIterator<FeatureKey> for Configuration {
    fn from_iter<T: IntoIterator<Item = FeatureKey>>(iter: T) -> Self {
        let mut config = Self::empty();

        for key in iter {
            config.set_enabled(key, true);
        }

        config
    }
}
