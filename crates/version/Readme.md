# squire-sqlite3-version

[Squire][] is a crate for embedding [SQLite][] in Rust. This `squire-sqlite3-version` crate represents [versions][] of the SQLite library.

Users of Squire don't need to interact with this crate directly, and can treat it as an implementation detail. Its `Version` type is available as [`squire::Version`][].

[Squire]: https://lib.rs/squire
[SQLite]: https://sqlite.org/
[C API]: https://sqlite.org/cintro.html
[versions]: https://sqlite.org/versionnumbers.html
[`squire::Version`]: https://docs.rs/squire/latest/squire/struct.Version.html
