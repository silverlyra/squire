use core::num::NonZero;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
#[repr(transparent)]
pub struct RowId(NonZero<i64>);

impl RowId {
    pub const fn new(value: i64) -> Option<Self> {
        match NonZero::new(value) {
            Some(id) => Some(RowId(id)),
            None => None,
        }
    }

    pub const fn into_inner(self) -> i64 {
        self.0.get()
    }
}
