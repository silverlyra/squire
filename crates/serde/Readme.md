# squire-sqlite3-features

[Squire][] is a crate for embedding [SQLite][] in Rust. This `squire-serde` crate integrates Squire with [Serde][], the ubiquitous serialization/deserialization crate.

Include the `serde` feature on your `squire` dependency to include Serde integration. (Users of Squire don't need to interact with this crate directly, and can treat it as an implementation detail.)

[Squire]: https://lib.rs/squire
[SQLite]: https://sqlite.org/
[Serde]: https://serde.rs/
