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

let statement = connection.prepare("SELECT id, username, score FROM users WHERE id = ?")?;

let id: squire::RowId = connection.execute("INSERT INTO users VALUES (DEFAULT, ?, ?);", ("boo", 0.69))?;

let user: (i32, String, f64) = statement.bind(101)?.fetch()?;

#[derive(squire::Statement)]
struct GetUser(i64);

#[squire::query]
fn get_user(id: i64) -> Result<User> {
    "SELECT * FROM users WHERE id = :id"
}

#[derive(squire::Row)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub score: f64,
}

#[derive(squire::Bind)]
#[bind(sequential)]
pub struct InsertUser<'a> {
    pub username: &'a str,
    pub score: f64,
}
```
