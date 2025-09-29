use crate::{error::Result, statement::Statement, value::Fetch};

pub use crate::ffi::Column;

pub trait Columns<'r>: Sized {
    type Indexes: Copy + Sized;

    fn resolve<'c>(statement: &Statement<'c>) -> Option<Self::Indexes>;

    fn fetch<'c>(statement: &'r mut Statement<'c>, indexes: Self::Indexes) -> Result<Self>
    where
        'c: 'r;
}

impl<'r, T> Columns<'r> for T
where
    T: Fetch<'r>,
{
    type Indexes = ();

    #[inline(always)]
    fn resolve<'c>(_statement: &Statement<'c>) -> Option<Self::Indexes> {
        Some(())
    }

    fn fetch<'c>(statement: &'r mut Statement<'c>, _indexes: Self::Indexes) -> Result<Self>
    where
        'c: 'r,
    {
        <Self as Fetch<'r>>::fetch(statement, Column::INITIAL)
    }
}

/// Implement [`Columns`] for a tuple type.
macro_rules! tuple {
    ($i:ident: $t:ident) => {
        impl<'r, $t> Columns<'r> for ($t,)
        where
            $t: Fetch<'r>,
        {
            type Indexes = ();

            #[inline(always)]
            fn resolve<'c>(_statement: &Statement<'c>) -> Option<Self::Indexes> {
                Some(())
            }

            fn fetch<'c>(statement: &'r mut Statement<'c>, _indexes: Self::Indexes) -> Result<Self>
            where
                'c: 'r,
            {
                let column = Column::INITIAL;
                let $i = <$t as Fetch<'r>>::fetch(statement, column)?;
                Ok(($i,))
            }
        }
    };

    ($ih:ident: $th:ident, $($it:ident: $tt:ident),+) => {
        impl<'r, $th, $($tt),+> Columns<'r> for ($th, $($tt),+)
        where
            $th: Fetch<'r>,
            $($tt: Fetch<'r>),+
        {
            type Indexes = ();

            #[inline(always)]
            fn resolve<'c>(_statement: &Statement<'c>) -> Option<Self::Indexes> {
                Some(())
            }

            fn fetch<'c>(statement: &'r mut Statement<'c>, _indexes: Self::Indexes) -> Result<Self>
            where
                'c: 'r,
            {
                let column = Column::INITIAL;
                let $ih = <$th as Fetch<'r>>::fetch(statement, column)?;

                $(
                    let column = column.next();
                    let $it = <$tt as Fetch<'r>>::fetch(statement, column)?;
                )*

                Ok(($ih, $($it),+))
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
