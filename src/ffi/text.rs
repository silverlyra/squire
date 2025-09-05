use super::statement::Statement;
use crate::error::Result;

pub(crate) unsafe trait Text {
    type Owned: Clone;
    type Slice<'a>: Copy + Sized + ToOwned<Owned = Self::Owned> + 'a;

    unsafe fn bind(&self, statement: &Statement) -> Result<()>;
}
