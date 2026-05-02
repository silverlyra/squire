# squire-sqlite3-config

[Squire][] is a crate for embedding [SQLite][] in Rust. This `squire-sqlite3-config` crate represents SQLite’s [compile-time options][], [library settings][], and [connection settings][].

Users of Squire don't need to interact with this crate directly, and can treat it as an implementation detail.

[Squire]: https://lib.rs/squire
[SQLite]: https://sqlite.org/
[C API]: https://sqlite.org/cintro.html
[compile-time options]: https://sqlite.org/compile.html
[library settings]: https://sqlite.org/c3ref/c_dbconfig_defensive.html
[connection settings]: https://sqlite.org/c3ref/c_config_covering_index_scan.html
