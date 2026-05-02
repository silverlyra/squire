use super::{Directive, DirectiveKey, DirectiveMap, Error};

impl DirectiveMap {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn contains_key(&self, key: DirectiveKey) -> bool {
        key.is_set(self)
    }

    pub fn keys(&self) -> impl Iterator<Item = DirectiveKey> {
        DirectiveKey::all().filter(|key| key.is_set(self))
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.keys().count()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.keys().next().is_none()
    }

    #[must_use]
    pub fn get<D: Directive>(&self) -> Option<D> {
        D::get(self)
    }

    #[must_use]
    pub fn value<D: Directive>(&self) -> Option<D::Value> {
        D::get(self).map(Directive::value)
    }

    pub fn insert<D: Directive>(&mut self, value: D) -> Option<D> {
        value.set(self)
    }

    pub fn remove<D: Directive>(&mut self) -> Option<D> {
        D::unset(self)
    }

    pub fn remove_key(&mut self, key: DirectiveKey) -> bool {
        key.unset(self)
    }
}

impl core::str::FromStr for DirectiveMap {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut map = Self::default();

        for line in s.lines() {
            if line.is_empty() {
                continue;
            }

            map.apply(line)?;
        }

        Ok(map)
    }
}

#[derive(Debug)]
pub(super) struct DirectiveKeys {
    current: usize,
}

impl DirectiveKeys {
    pub(super) const fn new() -> Self {
        Self { current: 0 }
    }
}

impl Iterator for DirectiveKeys {
    type Item = DirectiveKey;

    fn next(&mut self) -> Option<Self::Item> {
        let key = DirectiveKey::from_index(self.current);

        if key.is_some() {
            self.current += 1;
        }

        key
    }
}
