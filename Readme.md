squire
======

Squire is a [crate][] for embedding [SQLite][] in Rust. It provides a safe, idiomatic Rust interface to the underlying SQLite [C API][].

[crate]: https://lib.rs/squire
[SQLite]: https://sqlite.org/
[C API]: https://sqlite.org/cintro.html

```rust
use squire::{Connection, Database};

let db = Database::memory();
let connection = Connection::open(db)?;

let connection = Connection::builder(Database::memory()).open()?;
```
