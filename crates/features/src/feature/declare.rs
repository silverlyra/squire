macro_rules! features {
    { $($i:ident $(@ $v:literal)? $(^ $d:ident)* $(+ $e:ident)? $(- $o:ident)? $(? $m:ident)?),+ $(,)? } => {
        $(
            #[derive(Debug)]
            pub struct $i;

            $(
                impl $i {
                    #[doc = concat!("The SQLite release where [`", stringify!($i), "`] became available (", $v, ").")]
                    pub const AVAILABLE: $crate::version::Version = $crate::version::Version::declare($v);
                }
            )?
        )+

        $(
            impl $crate::feature::Feature for $i {
                fn is_available(&self, library: &$crate::info::Library) -> bool {
                    $(
                       if library.version() < features!(@version $v) {
                           return false;
                       }
                    )?

                    $(
                        if !$d.is_available(library) {
                            return false;
                        }
                    )*

                    $(
                        if library.directive($crate::directive::DirectiveKey::$e).is_none() {
                            return false;
                        }
                    )?

                    $(
                        if library.directive($crate::directive::DirectiveKey::$o).is_some() {
                            return false;
                        }
                    )?

                    features!(@matcher self, library, $($m)?)
                }

                fn key(&self) -> FeatureKey {
                    FeatureKey::$i
                }

                #[cfg(feature = "alloc")]
                #[allow(unused_variables)]
                fn apply(
                    &self,
                    enabled: bool,
                    directives: &mut DirectiveMap,
                    version: Version,
                    config: &Configuration,
                ) {
                    $(
                        if version < features!(@version $v) {
                           return;
                        }
                    )?

                    $(
                        if !config.is_enabled(FeatureKey::$d) {
                            return;
                        }
                    )*

                    $(
                        if enabled {
                            directives.insert($crate::directive::Directive::$e);
                        } else {
                            directives.remove($crate::directive::DirectiveKey::$e);
                        }
                    )?

                    $(
                        if !enabled {
                            directives.insert($crate::directive::Directive::$o);
                        } else {
                            directives.remove($crate::directive::DirectiveKey::$o);
                        }
                    )?

                    features!(@apply self, enabled, directives, version, $($m)?);
                }
            }
        )+

        /// Identifies a recognized SQLite [`Feature`].
        #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Copy, Clone, Debug)]
        #[repr(u32)]
        pub enum FeatureKey {
            $(
                $i,
            )+
        }

        impl FeatureKey {
            pub const COUNT: usize = (0 $(+ features!(@tally $i))+);

            const NAMES: [&'static str; Self::COUNT] = [
                $(
                    stringify!($i),
                )+
            ];

            pub fn all() -> impl Iterator<Item = Self> {
                (0..Self::COUNT as u32).map(Self::from_index)
            }

            pub fn available(library: &$crate::info::Library) -> impl Iterator<Item = Self> {
                Self::all().filter(|key| key.is_available(library))
            }

            pub const fn name(&self) -> &'static str {
                match *self {
                    $(
                        Self::$i => stringify!($i),
                    )+
                }
            }

            #[inline(always)]
            const fn index(&self) -> u32 {
                *self as u32
            }

            #[inline(always)]
            const fn offset(&self) -> usize {
                self.index() as usize
            }

            #[inline(always)]
            const fn from_index(i: u32) -> Self {
                unsafe { core::mem::transmute(i) }
            }
        }

        impl $crate::feature::Feature for FeatureKey {
            #[inline(always)]
            fn is_available(&self, library: &$crate::info::Library) -> bool {
                match *self {
                    $(
                        Self::$i => $i.is_available(&library),
                    )+
                }
            }

            fn key(&self) -> FeatureKey {
                *self
            }

            #[cfg(feature = "alloc")]
            fn apply(
                &self,
                enabled: bool,
                directives: &mut DirectiveMap,
                version: Version,
                config: &Configuration,
            ) {
                match *self {
                    $(
                        Self::$i => Feature::apply(&$i, enabled, directives, version, config),
                    )+
                }
            }
        }

        impl core::str::FromStr for FeatureKey {
            type Err = UnknownFeatureError;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                match Self::NAMES.binary_search(&value) {
                    Ok(i) => Ok(Self::from_index(i as u32)),
                    Err(_) => Err(UnknownFeatureError(value.to_owned()))
                }
            }
        }
    };

    (@matcher $s:ident, $l:ident, $m:ident) => {
        $s.$m($l)
    };

    (@matcher $_s:ident, $l:ident, ) => {
        true
    };

    (@apply $s:ident, $e:ident, $d:ident, $v:ident, $m:ident) => {
        $s.apply($e, $d, $v)
    };

    (@apply $s:ident, $e:ident, $d:ident, $v:ident, ) => {};

    (@version $v:literal) => {
        Self::AVAILABLE
    };

    (@tally $x:ident) => { 1 };
}

pub(super) use features;
