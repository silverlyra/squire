#![cfg_attr(
    feature = "build",
    doc = "Configuration value types (e.g., [`Threading`](crate::build::Threading) has"
)]
#![cfg_attr(
    feature = "build",
    doc = "values like [`Serialized`](Threading::Serialized) instead of `2`)."
)]
#![cfg_attr(not(feature = "build"), doc = "Configuration value types.")]

mod sql;
mod synchronous;
mod temp;
mod threading;
mod vacuum;

pub use sql::DoubleQuotedStrings;
pub use synchronous::Synchronous;
pub use temp::TemporaryStorage;
pub use threading::Threading;
pub use vacuum::AutomaticVacuum;
