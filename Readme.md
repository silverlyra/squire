squire
======

[![crate][crate-shield]][crate]
[![CI][ci-shield]][build]
[![docs][docs-shield]][docs]
[![Apache 2.0 license][license-shield]][license]

Squire is a [crate][] for embedding [SQLite][] in Rust. It provides a safe, idiomatic [Rust interface][docs] to the underlying SQLite [C API][].

[crate]: https://lib.rs/squire "squire on lib.rs"
[docs]: https://docs.rs/squire "squire on docs.rs"
[SQLite]: https://sqlite.org/
[C API]: https://sqlite.org/cintro.html
[build]: https://github.com/silverlyra/squire/actions/workflows/ci.yml?query=branch%3Amain "main branch continuous integration (CI) status"
[license]: ./LICENSE

> ⚠️ Squire is under active development, without even a `0.0.1` release yet. Not all features described below exist.

```rust
use squire::{Connection, Database};

let db = Database::memory();
let connection = Connection::open(db)?;

let connection = Connection::builder(Database::memory()).open()?;

let statement = connection.prepare("SELECT id, username, score FROM users WHERE id = ?")?;

let id: squire::RowId = connection.execute("INSERT INTO users VALUES (DEFAULT, ?, ?);", ("boo", 0.69))?;

let user: (i32, String, f64) = statement.bind(101)?.fetch()?;

#[derive(squire::Query)]
#[query = "SELECT * FROM users WHERE id = ?"]
struct GetUser(i64);

#[squire::query]
fn get_user(id: i64) -> Result<User> {
    "SELECT * FROM users WHERE id = :id"
}

#[derive(squire::Table)]
#[squire(table = users)]
pub struct User {
    pub id: squire::RowId,
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

[crate-shield]: https://img.shields.io/crates/v/squire.svg?label=%20&logo=data:image/svg%2bxml;base64,PHN2ZyBoZWlnaHQ9IjE2IiB3aWR0aD0iMTYiIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyI+PGcgZmlsbD0ibm9uZSIgc3Ryb2tlPSIjMDAwIj48cGF0aCBkPSJNOS44NCAxMS43OGw0LjUyMy0xLjk3NEwxMi44IDYuMjI0IDguMjc4IDguMTk3ek03LjcwOCA2LjkxN2w0LjYzOC0yLjAyMy0xLjU2My0zLjU4My00LjYzOCAyLjAyNHoiIHN0cm9rZS13aWR0aD0iLjYyNyIvPjxwYXRoIGQ9Ik0zLjcwNiAxLjg2NGM0Ljc3IDEwLjk3IDUuMzM5IDEyLjA5NyA1LjMzOSAxMi4wOTdsNS42ODctMi41NTkiLz48cGF0aCBkPSJNNS41NDYgMTQuNTRhMS4yNjcgMS4yNjcgMCAxIDEgMi41MDctLjM2MyAxLjI2NyAxLjI2NyAwIDAgMS0yLjUwNy4zNjR6IiBzdHJva2Utd2lkdGg9Ii43MzMiLz48cGF0aCBkPSJNNC4wMzcgMS4yNjZDMS45MjIgMi40MS44NjkgMS4zMjEgMS44NTUgMi45NCIgc3Ryb2tlLXdpZHRoPSIuODgiLz48cGF0aCBkPSJNNy42ODggOC4zNTlsNS41Ny0yLjQxNiIgc3Ryb2tlLXdpZHRoPSIuNjQzIi8+PHBhdGggZD0iTTUuNTU2IDMuNDk3TDExLjMyLjk4NCIgc3Ryb2tlLXdpZHRoPSIuNjY2Ii8+PC9nPjxwYXRoIGQ9Ik0xMS45NTkgOS40NDZjLS4wNjMtLjE1OC0uMTc0LS4xNzItLjcwNy0uMTE0LS40ODcuMDU3LS44NzItLjAyNy0xLjAzOS0uNDUtLjE2OS0uNDI2LjA3My0uODkuNzYzLTEuMTYyLjQwOC0uMTYyLjc5Ny0uMTc0IDEuMDU2LS4xMDRsLS4xMjEuNTQyYTEuMzIgMS4zMiAwIDAgMC0uNzE3LjA3NGMtLjIzMS4wOTEtLjMxNy4xOTQtLjI2OC4zMTYuMDQ4LjEyMy4xNjMuMTM1LjY5LjA4NC41MTktLjA1My44ODYtLjAxNSAxLjA3My40NTcuMjEuNTMxLS4yMSAxLjAwNy0uODI4IDEuMjUyLS40NjcuMTg1LS44OC4xOS0xLjIxNi4wNzJsLjIwNS0uNTQ0Yy4yNDQuMDcyLjUyLjA2OC44MDItLjA0NC4yMzYtLjA5My4zNjYtLjIyOS4zMDctLjM3OXpNMTAuNzM4IDUuMTZsLTEuOTEuOC0uMjE1LS41MTMuNTE4LS4yMTgtLjYxOC0xLjQ3Ni0uNTE5LjIxOC0uMjE1LS41MTQgMS4xODgtLjQ5OC4yNTcuNDRjLS4wMjctLjM4MS4xNTQtLjcwMi40Ni0uODNhMS4xNyAxLjE3IDAgMCAxIC4zMy0uMDg4bC4xODQuNzgyYTEuMDIzIDEuMDIzIDAgMCAwLS4zMDQuMDgzYy0uMzExLjEzLS4zOC40NTQtLjMyLjgyMmwuMzEyLjc0NS42MzYtLjI2N3ptLS44OTcgNi43MjZsNC41MjItMS45NzNMMTIuOCA2LjMzIDguMjc4IDguMzA0ek03LjcwNyA3LjAyNUwxMi4zNDUgNWwtMS41NjMtMy41OC00LjYzOCAyLjAyM3oiLz48L3N2Zz4=
[ci-shield]: https://img.shields.io/github/actions/workflow/status/silverlyra/squire/ci.yml?label=%20&logo=github-actions&logoColor=101010
[docs-shield]: https://img.shields.io/docsrs/squire?label=%20&logo=data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAACAAAAAgCAQAAADZc7J/AAAAIGNIUk0AAHomAACAhAAA+gAAAIDoAAB1MAAA6mAAADqYAAAXcJy6UTwAAAACYktHRAD/h4/MvwAAAAd0SU1FB+oEGRUaKDzvHysAAAAldEVYdGRhdGU6Y3JlYXRlADIwMjYtMDQtMjVUMjE6MjY6NDArMDA6MDDUoKYVAAAAJXRFWHRkYXRlOm1vZGlmeQAyMDI2LTA0LTI1VDIxOjI2OjQwKzAwOjAwpf0eqQAAACh0RVh0ZGF0ZTp0aW1lc3RhbXAAMjAyNi0wNC0yNVQyMToyNjo0MCswMDowMPLoP3YAAAFsSURBVEjH7ZTddZtAEIU/dFzAdmA6CCXgCqIO7FQQdRBSgXAFSiqIXYHUgegAOmA7+PIgBMgGDsfHb8mdB9iFO3N3fhb+4/Ng6t7aJZz9YfqWl3T0jCMh0hBnAgQygIaHpJmK3mphkAVLLVVrw3sHB90tkq9WqhZv6UHrVXQMttqONWyADF5Xpjryu0/H4KCB712IG2WdtexHuw1AeuMgafhGRcMMArtxyIVOOHpz3gGtab97UM0H1t28w5/9uU+9vEAOMTmtUjBle9Vy5RHel3A/0UiblfUD4qXvU9o+ObWPyVgB+WjJoXu+8tJnoCYAkQqAjBBXVWGoQanqsVud1Y2PdpLImcGuf7tEvtYkhXhHEUM1STtdZfKl3/tFxn1X4ECACtvlQTqq7eSXrWq5oUrHrT2JMPnHV4AXfBqSMqdAi4nrRa377J/NFx2MpwExv1yfTxcHwT9+BAUMl+qWLffru5LI88xI/YP4C2X76vn+mE55AAAAAElFTkSuQmCC
[license-shield]: https://img.shields.io/crates/l/squire.svg?label=%20
