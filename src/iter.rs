//! [`Iterator`] implementations for [`Rows`].

use crate::{
    column::{ColumnIndexes, Columns},
    error::Result,
    row::Rows,
    statement::{Binding, Execute},
};

/// Map over [`Rows`].
///
/// Returned by [`Rows::map`].
#[derive(Debug)]
pub struct Map<'c, 's, C, F, S = Binding<'c, 's>>
where
    C: ColumnIndexes,
    S: Execute<'c, 's>,
    'c: 's,
{
    pub(crate) rows: Rows<'c, 's, C, S>,
    pub(crate) f: F,
}

impl<'c, 's, 'r, C, F, T, S> Iterator for Map<'c, 's, C, F, S>
where
    C: Columns<'r>,
    F: FnMut(C) -> T,
    T: 's,
    S: Execute<'c, 's>,
    'c: 's,
    's: 'r,
{
    type Item = Result<T>;

    fn next(&mut self) -> Option<Self::Item> {
        // SAFETY: We never hold onto row data across loop iterations
        match unsafe { self.rows.advance() } {
            Ok(Some(item)) => Some(Ok((self.f)(item))),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }
}

/// Map over [`Rows`] and filter the results.
///
/// Returned by [`Rows::filter_map`].
#[derive(Debug)]
pub struct FilterMap<'c, 's, C, F, S = Binding<'c, 's>>
where
    C: ColumnIndexes,
    S: Execute<'c, 's>,
    'c: 's,
{
    pub(crate) rows: Rows<'c, 's, C, S>,
    pub(crate) f: F,
}

impl<'c, 's, 'r, C, F, T, S> Iterator for FilterMap<'c, 's, C, F, S>
where
    C: Columns<'r>,
    F: FnMut(C) -> Option<T>,
    T: 's,
    S: Execute<'c, 's>,
    'c: 's,
    's: 'r,
{
    type Item = Result<T>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // SAFETY: We never hold onto row data across loop iterations
            match unsafe { self.rows.advance() } {
                Ok(Some(item)) => {
                    if let Some(mapped) = (self.f)(item) {
                        return Some(Ok(mapped));
                    }
                }
                Ok(None) => return None,
                Err(e) => return Some(Err(e)),
            }
        }
    }
}
