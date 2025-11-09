use crate::{
    bind::Bind,
    error::Result,
    statement::{Binding, Statement},
    types::BindIndex,
};

pub trait Parameters<'s> {
    type Indexes: Copy + Sized;

    fn resolve<'c>(statement: &Statement<'c>) -> Option<Self::Indexes>;

    fn bind<'c>(self, binding: &mut Binding<'c, 's>, indexes: Self::Indexes) -> Result<()>
    where
        'c: 's;
}

impl<'s, T> Parameters<'s> for T
where
    T: Bind<'s>,
{
    type Indexes = ();

    #[inline(always)]
    fn resolve<'c>(_statement: &Statement<'c>) -> Option<Self::Indexes> {
        Some(())
    }

    fn bind<'c>(self, binding: &mut Binding<'c, 's>, _indexes: Self::Indexes) -> Result<()>
    where
        'c: 's,
    {
        binding.set(BindIndex::INITIAL, self)?;
        Ok(())
    }
}
impl<'s> Parameters<'s> for () {
    type Indexes = ();

    #[inline(always)]
    fn resolve<'c>(_statement: &Statement<'c>) -> Option<Self::Indexes> {
        Some(())
    }

    fn bind<'c>(self, _binding: &mut Binding<'c, 's>, _indexes: Self::Indexes) -> Result<()>
    where
        'c: 's,
    {
        Ok(())
    }
}

/// Implement [`Parameters`] for a tuple type.
macro_rules! tuple {
    ($i:ident: $t:ident) => {
        impl<'s, $t> Parameters<'s> for ($t,)
        where
            $t: Bind<'s>,
        {
            type Indexes = ();

            #[inline(always)]
            fn resolve<'c>(_statement: &Statement<'c>) -> Option<Self::Indexes> {
                Some(())
            }

            fn bind<'c>(self, binding: &mut Binding<'c, 's>, _indexes: Self::Indexes) -> Result<()>
            where
                'c: 's,
            {
                let ($i,) = self;
                binding.set(BindIndex::INITIAL, $i)?;
                Ok(())
            }
        }
    };

    ($ih:ident: $th:ident, $($it:ident: $tt:ident),+) => {
        impl<'s, $th, $($tt),+> Parameters<'s> for ($th, $($tt),+)
        where
            $th: Bind<'s>,
            $($tt: Bind<'s>),+
        {
            type Indexes = ();

            #[inline(always)]
            fn resolve<'c>(_statement: &Statement<'c>) -> Option<Self::Indexes> {
                Some(())
            }

            fn bind<'c>(self, binding: &mut Binding<'c, 's>, _indexes: Self::Indexes) -> Result<()>
            where
                'c: 's,
            {
                let ($ih, $($it),+) = self;

                let index = BindIndex::INITIAL;
                binding.set(index, $ih)?;

                $(
                    let index = index.next();
                    binding.set(index, $it)?;
                )*

                Ok(())
            }
        }
    };
}

tuple!(a: A);
tuple!(a: A, b: B);
tuple!(a: A, b: B, c: C);
tuple!(a: A, b: B, c: C, d: D);
tuple!(a: A, b: B, c: C, d: D, e: E);
tuple!(a: A, b: B, c: C, d: D, e: E, f: F);
tuple!(a: A, b: B, c: C, d: D, e: E, f: F, g: G);
tuple!(a: A, b: B, c: C, d: D, e: E, f: F, g: G, h: H);
tuple!(a: A, b: B, c: C, d: D, e: E, f: F, g: G, h: H, i: I);
tuple!(a: A, b: B, c: C, d: D, e: E, f: F, g: G, h: H, i: I, j: J);
tuple!(a: A, b: B, c: C, d: D, e: E, f: F, g: G, h: H, i: I, j: J, k: K);
tuple!(a: A, b: B, c: C, d: D, e: E, f: F, g: G, h: H, i: I, j: J, k: K, l: L);
tuple!(a: A, b: B, c: C, d: D, e: E, f: F, g: G, h: H, i: I, j: J, k: K, l: L, m: M);
