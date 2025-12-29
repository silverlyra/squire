use super::text::Encoding;

#[cfg_attr(docsrs, doc(cfg(feature = "functions")))]
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[repr(transparent)]
pub struct FunctionOptions(i32);

impl FunctionOptions {
    const DETERMINISTIC: i32 = sqlite::SQLITE_DETERMINISTIC;
    #[cfg(sqlite_has_direct_only_function_option)]
    const DIRECT_ONLY: i32 = sqlite::SQLITE_DIRECTONLY;
    #[cfg(sqlite_has_subtype_function_option)]
    const ARGUMENT_SUBTYPE: i32 = sqlite::SQLITE_SUBTYPE;
    #[cfg(sqlite_has_innocuous_function_option)]
    const INNOCUOUS: i32 = sqlite::SQLITE_INNOCUOUS;
    #[cfg(sqlite_has_strict_subtypes)]
    const RESULT_SUBTYPE: i32 = sqlite::SQLITE_RESULT_SUBTYPE;
    #[cfg(sqlite_has_aggregate_self_ordered)]
    const AGGREGATE_SELF_ORDERED_1: i32 = sqlite::SQLITE_SELFORDER1;

    pub const fn new(encoding: Encoding) -> Self {
        Self(encoding.raw())
    }

    pub const fn from_raw(options: i32) -> Self {
        Self(options)
    }

    pub const fn raw(self) -> i32 {
        self.0
    }

    #[doc(alias = "SQLITE_DETERMINISTIC")]
    pub const fn deterministic(self, value: bool) -> Self {
        self.set(Self::DETERMINISTIC, value)
    }

    #[cfg(sqlite_has_direct_only_function_option)]
    #[doc(alias = "SQLITE_DIRECTONLY")]
    pub const fn direct_only(self, value: bool) -> Self {
        self.set(Self::DIRECT_ONLY, value)
    }

    #[cfg(sqlite_has_innocuous_function_option)]
    #[doc(alias = "SQLITE_INNOCUOUS")]
    pub const fn innocuous(self, value: bool) -> Self {
        self.set(Self::INNOCUOUS, value)
    }

    #[cfg(sqlite_has_subtype_function_option)]
    #[doc(alias = "SQLITE_SUBTYPE")]
    pub const fn argument_subtypes(self, value: bool) -> Self {
        self.set(Self::ARGUMENT_SUBTYPE, value)
    }

    #[cfg(sqlite_has_strict_subtypes)]
    #[doc(alias = "SQLITE_RESULT_SUBTYPE")]
    pub const fn result_subtype(self, value: bool) -> Self {
        self.set(Self::RESULT_SUBTYPE, value)
    }

    #[cfg(all(sqlite_has_subtype_function_option, sqlite_has_strict_subtypes))]
    pub const fn subtypes(self, value: bool) -> Self {
        self.argument_subtypes(value).result_subtype(value)
    }

    #[cfg(sqlite_has_aggregate_self_ordered)]
    pub const fn aggregate_supports_order_by(self, value: bool) -> Self {
        self.set(Self::AGGREGATE_SELF_ORDERED_1, value)
    }

    const fn set(self, bit: i32, value: bool) -> Self {
        if value {
            Self(self.0 | bit)
        } else {
            Self(self.0 & !bit)
        }
    }
}

impl Default for FunctionOptions {
    fn default() -> Self {
        Self::new(Encoding::default())
    }
}
