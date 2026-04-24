use alloc::collections::BTreeMap;
use core::{fmt, str::FromStr};

use super::{Directive, DirectiveKey, ParseDirectiveError};

/// An ordered map of [`Directive`]s keyed by their [`DirectiveKey`].
#[derive(PartialEq, Eq, Default, Clone, Debug)]
#[cfg_attr(docsrs, doc(cfg(any(feature = "std", feature = "alloc"))))]
pub struct DirectiveMap(BTreeMap<DirectiveKey, Directive>);

impl DirectiveMap {
    /// Construct an empty [`DirectiveMap`].
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }

    /// The number of directives in the map.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// `true` if the map contains no directives.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Remove all directives from the map.
    pub fn clear(&mut self) {
        self.0.clear();
    }

    /// Returns the [`Directive`] associated with `key`, if any.
    pub fn get(&self, key: DirectiveKey) -> Option<Directive> {
        self.0.get(&key).copied()
    }

    /// `true` if the map contains a [`Directive`] for `key`.
    pub fn contains_key(&self, key: DirectiveKey) -> bool {
        self.0.contains_key(&key)
    }

    /// Insert `directive`, replacing any previous value stored under its
    /// [`key`](Directive::key). Returns the previous value, if any.
    pub fn insert(&mut self, directive: Directive) -> Option<Directive> {
        self.0.insert(directive.key(), directive)
    }

    /// Remove and return the [`Directive`] for `key`, if present.
    pub fn remove(&mut self, key: DirectiveKey) -> Option<Directive> {
        self.0.remove(&key)
    }

    /// Iterate over `(key, directive)` pairs in key order.
    pub fn iter(&self) -> impl Iterator<Item = (DirectiveKey, Directive)> + '_ {
        self.0.iter().map(|(k, v)| (*k, *v))
    }

    /// Iterate over the [`DirectiveKey`]s in order.
    pub fn keys(&self) -> impl Iterator<Item = DirectiveKey> + '_ {
        self.0.keys().copied()
    }

    /// Iterate over the [`Directive`]s in key order.
    pub fn values(&self) -> impl Iterator<Item = Directive> + '_ {
        self.0.values().copied()
    }
}

impl FromIterator<Directive> for DirectiveMap {
    fn from_iter<I: IntoIterator<Item = Directive>>(iter: I) -> Self {
        let mut map = Self::new();
        map.extend(iter);
        map
    }
}

impl Extend<Directive> for DirectiveMap {
    fn extend<I: IntoIterator<Item = Directive>>(&mut self, iter: I) {
        for directive in iter {
            self.insert(directive);
        }
    }
}

impl fmt::Display for DirectiveMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, directive) in self.values().enumerate() {
            if i > 0 {
                f.write_str("\n")?;
            }
            fmt::Display::fmt(&directive, f)?;
        }
        Ok(())
    }
}

impl FromStr for DirectiveMap {
    type Err = ParseDirectiveError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut map = Self::new();
        for line in s.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            match line.parse::<Directive>() {
                Ok(directive) => {
                    map.insert(directive);
                }
                Err(ParseDirectiveError::UnknownKey) => continue,
                Err(err) => return Err(err),
            }
        }
        Ok(map)
    }
}
