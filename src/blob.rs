/// A request for SQLite to allocate a blob of a certain size.
///
/// When a `Reservation` is [used](crate::Bind) as a prepared
/// [statement](crate::Statement) parameter, SQLite will create a `BLOB` of the
/// [requested length](Reservation::len()) and set every byte in the blob to `\0`.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
#[repr(transparent)]
pub struct Reservation(isize);

#[allow(clippy::len_without_is_empty)]
impl Reservation {
    /// Create a [`Reservation`].
    pub const fn new(bytes: isize) -> Self {
        Self(bytes)
    }

    /// The number of bytes to reserve, as a [`usize`].
    pub const fn len(&self) -> usize {
        self.0 as usize
    }

    /// The number of bytes to reserve, as an [`isize`].
    pub const fn into_inner(self) -> isize {
        self.0
    }
}

impl<T> From<T> for Reservation
where
    isize: From<T>,
{
    fn from(value: T) -> Self {
        Self::new(isize::from(value))
    }
}
