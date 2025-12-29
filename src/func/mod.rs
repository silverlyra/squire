use crate::ffi::ContextRef;

pub trait Return<'b> {
    fn apply<'a>(self, context: &mut ContextRef<'a>)
    where
        'b: 'a;
}
