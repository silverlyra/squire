# squire-sqlite3-sys

[Squire][] is a crate for embedding [SQLite][] in Rust. This `squire-sqlite3-sys` crate links SQLite into the application, exposing the [C API][] of SQLite to Rust.

Users of Squire don't need to interact with this crate directly, and can treat it as an implementation detail.

[Squire]: https://lib.rs/squire
[SQLite]: https://sqlite.org/
[C API]: https://sqlite.org/cintro.html

## External Users

By default, `squire-sqlite3-sys` ships a [predefined set][default] of `extern "C"` declarations for SQLite. This includes only the API’s actually used by Squire.

To instead generate complete bindings based on the installed or bundled `sqlite3.h` header, activate the [`bindgen`][bindgen] feature.

[default]: https://github.com/silverlyra/squire/blob/main/crates/sys/src/bindings/default/mod.rs
[bindgen]: https://lib.rs/bindgen
