# squire-sqlite3-src

This crate [bundles][] the [SQLite][] source code for [Squire][]. When Squire’s
`bundled` feature flag is enabled, SQLite is built from the `sqlite3.c` file
included in this crate, and linked into the [`squire-sqlite3-sys`][sys] crate.

Users of Squire don’t need to be aware of or interact with this crate. Cargo
will automatically download and build it when you enable Squire’s `bundled`
feature.

[bundles]: ./vendor/README.md
[SQLite]: https://sqlite.org/
[Squire]: https://github.com/silverlyra/squire
[sys]: https://lib.rs/squire-sqlite3-sys
