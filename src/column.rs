use core::fmt;

use crate::{error::Result, statement::Statement, types::ColumnIndex, value::Fetch};

pub trait ColumnIndexes {
    type Indexes: Copy + fmt::Debug + Sized;

    fn resolve<'c>(statement: &Statement<'c>) -> Option<Self::Indexes>;
}

pub trait Columns<'r>: ColumnIndexes + Sized {
    fn fetch<'c>(statement: &'r Statement<'c>, indexes: Self::Indexes) -> Result<Self>
    where
        'c: 'r;
}

impl<'r, T> ColumnIndexes for T
where
    T: Fetch<'r>,
{
    type Indexes = ();

    #[inline(always)]
    fn resolve<'c>(_statement: &Statement<'c>) -> Option<Self::Indexes> {
        Some(())
    }
}

impl<'r, T> Columns<'r> for T
where
    T: Fetch<'r>,
{
    fn fetch<'c>(statement: &'r Statement<'c>, _indexes: Self::Indexes) -> Result<Self>
    where
        'c: 'r,
    {
        <T as Fetch<'r>>::fetch(statement, ColumnIndex::INITIAL)
    }
}

/// Implement [`Columns`] for a tuple type.
macro_rules! tuple {
    ($i:ident: $t:ident) => {
        impl<$t> ColumnIndexes for ($t,) {
            type Indexes = ();

            #[inline(always)]
            fn resolve<'c>(_statement: &Statement<'c>) -> Option<Self::Indexes> {
                Some(())
            }
        }

        impl<'r, $t> Columns<'r> for ($t,)
        where
            $t: Fetch<'r>,
        {
            fn fetch<'c>(statement: &'r Statement<'c>, _indexes: Self::Indexes) -> Result<Self>
            where
                'c: 'r,
            {
                let column = ColumnIndex::INITIAL;
                let $i = <$t as Fetch<'r>>::fetch(statement, column)?;
                Ok(($i,))
            }
        }
    };

    ($ih:ident: $th:ident, $($it:ident: $tt:ident),+) => {
        impl<$th, $($tt),+> ColumnIndexes for ($th, $($tt),+)
        {
            type Indexes = ();

            #[inline(always)]
            fn resolve<'c>(_statement: &Statement<'c>) -> Option<Self::Indexes> {
                Some(())
            }
        }

        impl<'r, $th, $($tt),+> Columns<'r> for ($th, $($tt),+)
        where
            $th: Fetch<'r>,
            $($tt: Fetch<'r>),+
        {
            fn fetch<'c>(statement: &'r Statement<'c>, _indexes: Self::Indexes) -> Result<Self>
            where
                'c: 'r,
            {
                let column = ColumnIndex::INITIAL;
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
