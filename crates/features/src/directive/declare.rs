macro_rules! directive_key_pat {
    ($i:ident) => {
        Self::$i
    };
    ($i:ident ($v:ty)) => {
        Self::$i(_)
    };
}

macro_rules! directive_fmt_pat {
    ($i:ident) => {
        Self::$i
    };
    ($i:ident ($v:ty, $c:ident)) => {
        Self::$i($c)
    };
}

macro_rules! directive_fmt_body {
    ($e:ident, $f:ident) => {
        $f.write_str(stringify!($e))
    };
    ($e:ident, $f:ident, $v:ty, $c:ident) => {{
        $f.write_str(stringify!($e))?;
        $f.write_str("=")?;
        <$v as DirectiveValue>::write_value($c, $f)
    }};
}

macro_rules! directive_parse_body {
    ($i:ident, $value:ident) => {
        match $value {
            Some(_) => Err(ParseDirectiveError::UnexpectedValue(DirectiveKey::$i)),
            None => Ok(Self::$i),
        }
    };
    ($i:ident, $value:ident, $v:ty) => {
        match $value {
            Some(v) => <$v as DirectiveValue>::parse_value(DirectiveKey::$i, v).map(Self::$i),
            None => Err(ParseDirectiveError::MissingValue(DirectiveKey::$i)),
        }
    };
}

macro_rules! directives {
    { $( $e:ident => $i:ident $(( $v:ty ))? ),+ $(,)? } => {
        #[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
        pub enum Directive {
            $(
                #[doc = concat!("`SQLITE_", stringify!($e), "`")]
                $i $(( $v ))?,
            )+
        }

        impl Directive {
            /// The [key](DirectiveKey) (compile-time option name) of this
            /// [`Directive`].
            pub const fn key(&self) -> DirectiveKey {
                match *self {
                    $(
                        directive_key_pat!($i $(($v))?) => DirectiveKey::$i,
                    )+
                }
            }
        }

        impl core::fmt::Display for Directive {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                match self {
                    $(
                        directive_fmt_pat!($i $(($v, value))?) => directive_fmt_body!($e, f $(, $v, value)?),
                    )+
                }
            }
        }

        impl core::str::FromStr for Directive {
            type Err = ParseDirectiveError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let (name, value) = match s.split_once('=') {
                    Some((n, v)) => (n, Some(v)),
                    None => (s, None),
                };
                let key = DirectiveKey::of(name)
                    .ok_or(ParseDirectiveError::UnknownKey)?;
                match key {
                    $(
                        DirectiveKey::$i => directive_parse_body!($i, value $(, $v)?),
                    )+
                }
            }
        }

        #[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
        pub enum DirectiveKey {
            $(
                #[doc = concat!("`SQLITE_", stringify!($e), "`")]
                $i,
            )+
        }

        impl DirectiveKey {
            const PREFIX: &str = "SQLITE_";

            /// The name of the compile-time option.
            ///
            /// ```rust
            /// # use squire_sqlite3_features::DirectiveKey::*;
            /// assert_eq!(EnableJson1.name(), "SQLITE_ENABLE_JSON1");
            /// assert_eq!(OmitJson.name(), "SQLITE_OMIT_JSON");
            /// ```
            pub const fn name(&self) -> &'static str {
                match *self {
                    $(
                        Self::$i => concat!("SQLITE_", stringify!($e)),
                    )+
                }
            }

            /// The [`name`](Self::name) of the directive, without the `SQLITE_` prefix.
            ///
            /// ```rust
            /// # use squire_sqlite3_features::DirectiveKey::*;
            /// assert_eq!(EnableJson1.base_name(), "ENABLE_JSON1");
            /// assert_eq!(OmitJson.base_name(), "OMIT_JSON");
            /// ```
            pub const fn base_name(&self) -> &'static str {
                match *self {
                    $(
                        Self::$i => stringify!($e),
                    )+
                }
            }

            /// Parses a compile-time option name into a `DirectiveKey`.
            ///
            /// This method accepts names with or without the `SQLITE_` prefix.
            ///
            /// ```rust
            /// # use squire_sqlite3_features::DirectiveKey::{self, *};
            /// assert_eq!(DirectiveKey::of("SQLITE_ENABLE_JSON1"), Some(EnableJson1));
            /// assert_eq!(DirectiveKey::of("ENABLE_JSON1"), Some(EnableJson1));
            /// assert_eq!(DirectiveKey::of("OMIT_JSON"), Some(OmitJson));
            /// assert_eq!(DirectiveKey::of("INVALID_FLAG"), None);
            /// ```
            pub fn of(s: impl AsRef<str>) -> Option<Self> {
                let s = s.as_ref();
                let normalized = s.strip_prefix(Self::PREFIX).unwrap_or(s);

                match normalized {
                    $(
                        stringify!($e) => Some(Self::$i),
                    )+
                    _ => None,
                }
            }
        }
    };
}

pub(super) use {
    directive_fmt_body, directive_fmt_pat, directive_key_pat, directive_parse_body, directives,
};
