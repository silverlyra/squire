macro_rules! directives {
    { $( $o:ident || $l:ident => $i:ident $(( $v:ty ))? ),+ $(,)? } => {
        $(
            directives!(@struct $o || $l => $i $(($v))?);

            impl Directive for $i {
                const NAME: &str = concat!("SQLITE_", stringify!($o));
                const BASE_NAME: &str = Self::NAME.split_at(DirectiveKey::PREFIX.len()).1;
                const KEY: DirectiveKey = DirectiveKey::$i;

                type Value = directives!(@value type $(($v))?);

                #[inline]
                fn value(self) -> Self::Value {
                    directives!(@value for self $i $(($v))?)
                }

                #[inline]
                fn get(map: &DirectiveMap) -> Option<Self> {
                    map.$l
                }

                #[inline]
                fn set(self, map: &mut DirectiveMap) -> Option<Self> {
                    map.$l.replace(self)
                }

                #[inline]
                fn unset(map: &mut DirectiveMap) -> Option<Self> {
                    map.$l.take()
                }
            }

            directives!(@from $i $(($v))?);

            impl fmt::Display for $i {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    directives!(@display for self in f $o $(($v))?)
                }
            }

            impl From<$i> for DirectiveKey {
                #[inline]
                fn from(_: $i) -> Self {
                    Self::$i
                }
            }
        )+

        /// Identifies a [`Directive`].
        #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Copy, Clone, Debug)]
        pub enum DirectiveKey {
            $(
                #[doc = directives!(@doc $o[])]
                #[doc = ""]
                #[doc = directives!(@doc $o as $l)]
                $i,
            )+
        }

        impl DirectiveKey {
            const COUNT: usize = (0 $(+ directives!(@count $i))+);

            const ALL: [Self; Self::COUNT] = [
                $(
                    Self::$i,
                )+
            ];

            const PREFIX: &str = "SQLITE_";

            /// The name of the compile-time option.
            ///
            /// ```rust
            #[doc = concat!("# use ", env!("CARGO_CRATE_NAME"), "::build::DirectiveKey::*;")]
            /// assert_eq!(EnableJson1.name(), "SQLITE_ENABLE_JSON1");
            /// assert_eq!(OmitJson.name(), "SQLITE_OMIT_JSON");
            /// ```
            pub const fn name(self) -> &'static str {
                match self {
                   $(
                       Self::$i => $i::NAME,
                   )+
                }
            }

            /// The [`name`](Self::name) of the directive, without the `SQLITE_` prefix.
            ///
            /// ```rust
            #[doc = concat!("# use ", env!("CARGO_CRATE_NAME"), "::build::DirectiveKey::*;")]
            /// assert_eq!(EnableJson1.base_name(), "ENABLE_JSON1");
            /// assert_eq!(OmitJson.base_name(), "OMIT_JSON");
            /// ```
            pub const fn base_name(self) -> &'static str {
                match self {
                    $(
                        Self::$i => $i::BASE_NAME,
                    )+
                }
            }

            pub fn of(s: impl AsRef<str>) -> Option<Self> {
                let s = s.as_ref();
                let normalized = s.strip_prefix(Self::PREFIX).unwrap_or(s);

                match normalized {
                    $(
                        stringify!($o) => Some(Self::$i),
                    )+
                    _ => None,
                }
            }

            const fn from_index(value: usize) -> Option<Self> {
                if value < Self::COUNT {
                    Some(Self::ALL[value])
                } else {
                    None
                }
            }

            const fn is_set(self, map: &DirectiveMap) -> bool {
                match self {
                   $(
                       Self::$i => map.$l.is_some(),
                   )+
                }
            }

            fn unset(self, map: &mut DirectiveMap) -> bool {
                match self {
                   $(
                       Self::$i => map.$l.take().is_some(),
                   )+
                }
            }

            pub const fn all() -> impl Iterator<Item = Self> {
                DirectiveKeys::new()
            }
        }

        impl core::str::FromStr for DirectiveKey {
            type Err = Error;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                match value {
                    $(
                        concat!("SQLITE_", stringify!($o)) => Ok(Self::$i),
                    )+
                    _ => return Err(Error::Unknown),
                }
            }
        }

        impl fmt::Display for DirectiveKey {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self {
                    $(
                        Self::$i => f.write_str(<$i as Directive>::NAME),
                    )+
                }
            }
        }

        #[derive(PartialEq, Eq, Hash, Copy, Clone, Default, Debug)]
        pub struct DirectiveMap {
            $(
                $l: Option<$i>,
            )+
        }

        impl DirectiveMap {
            /// Set a [`Directive`] on the map by its C macro declaration.
            ///
            /// # Examples
            ///
            /// ```rust
            /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
            #[doc = concat!("use ", env!("CARGO_CRATE_NAME"), "::build;")]
            ///
            /// let mut map = build::DirectiveMap::new();
            ///
            /// map.apply("SQLITE_DEFAULT_MEMSTATUS=0")?;
            /// map.apply("SQLITE_ENABLE_API_ARMOR")?;
            ///
            /// assert_eq!(map.value::<build::DefaultMemoryStatus>(), Some(false));
            /// assert!(map.contains_key(build::DirectiveKey::EnableApiArmor));
            /// # Ok(())
            /// # }
            /// ```
            ///
            /// # Errors
            ///
            /// Returns [`Unknown`](Error::Unknown) if the [`DirectiveKey`] is
            /// not recognized, or [`Value`](Error::Value) for errors with the
            /// directive [value](Directive::Value).
            pub fn apply(&mut self, directive: impl AsRef<str>) -> Result<(), Error> {
                let directive = directive.as_ref();

                let (name, value) = match directive.split_once('=') {
                    Some((n, v)) => (n, Some(v)),
                    None => (directive, None),
                };

                match name {
                    $(
                        concat!("SQLITE_", stringify!($o)) => { <$i as Directive>::parse(value)?.set(self); }
                    )+
                    _ => return Err(Error::Unknown),
                };

                Ok(())
            }
        }

        impl fmt::Display for DirectiveMap {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                $(
                    if let Some($l) = self.$l {
                        writeln!(f, "{}", $l)?;
                    }
                )+

                Ok(())
            }
        }
    };

    (@struct $o:ident || $l:ident => $i:ident) => {
        #[doc = directives!(@doc $o)]
        #[doc = ""]
        #[doc = directives!(@doc $o as $l)]
        #[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
        pub struct $i;
    };

    (@struct $o:ident || $l:ident => $i:ident ($v:ty)) => {
        #[doc = directives!(@doc $o)]
        #[doc = ""]
        #[doc = directives!(@doc $o as $l)]
        #[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
        pub struct $i(pub $v);
    };

    (@doc $o:ident) => {
        concat!("The [`SQLITE_", stringify!($o), "`][] compile-time [option](Directive).")
    };

    (@doc $o:ident[]) => {
        concat!("[`SQLITE_", stringify!($o), "`][]")
    };

    (@doc $o:ident as $l:ident) => {
        concat!("[`SQLITE_", stringify!($o), "`]: https://sqlite.org/compile.html#", stringify!($l))
    };

    (@value type) => {
        ()
    };

    (@value type ($v:ty)) => {
        $v
    };

    (@value for $s:ident $i:ident) => { () };

    (@value for $s:ident $i:ident ($v:ty)) => {{
        let $i(value) = $s;
        value
    }};

    (@from $i:ident) => {
        impl From<()> for $i {
            fn from(_: ()) -> Self {
                Self
            }
        }
    };

    (@from $i:ident ($v:ty)) => {
        impl From<$v> for $i {
            fn from(value: $v) -> Self {
                Self(value)
            }
        }
    };

    (@display for $s:ident in $f:ident $o:ident) => {
        $f.write_str(stringify!($o))
    };

    (@display for $s:ident in $f:ident $o:ident ($v:ty)) => {{
        write!($f, concat!(stringify!($o), "="))?;
        $s.value().write_value($f)
    }};

    (@parse($t:ident) $o:ident => $i:ident) => {
        match value {
            concat!("SQLITE_", stringify!($o)) => Ok(Self::from())
        }
    };

    (@count $i:ident) => { 1 };
}

pub(super) use directives;
