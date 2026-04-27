squire
======

[![crate][crate-shield]][crate]
[![CI][ci-shield]][build]
[![docs][docs-shield]][docs]
[![Apache 2.0 license][license-shield]][license]

Squire is a [crate][] for embedding [SQLite][] in Rust. It provides a safe, idiomatic [Rust interface][docs] to the underlying SQLite [C API][].

[crate]: https://lib.rs/squire "squire on lib.rs"
[docs]: https://docs.rs/squire "squire on docs.rs"
[SQLite]: https://sqlite.org/ "SQLite home page"
[C API]: https://sqlite.org/cintro.html "introduction to the SQLite C API"
[build]: https://github.com/silverlyra/squire/actions/workflows/ci.yml?query=branch%3Amain "main branch continuous integration (CI) status"
[license]: ./LICENSE "Apache 2.0 license"

> ⚠️ Squire is under active development, without even a `0.0.1` release yet.

```rust
use squire::{Bind, Columns, Connection, Memory};

let connection = Connection::open(Memory)?;

let connection = Connection::builder("./data.sqlite3")
    .read_only()
    .follow_symbolic_links(false)
    .open()?;

let statement = connection.prepare("SELECT id, username, score FROM users WHERE id = ?")?;

let id: squire::RowId = connection.execute("INSERT INTO users VALUES (DEFAULT, ?, ?);", ("boo", 0.69))?;

let user: (i32, String, f64) = statement.bind(id)?.fetch()?;

#[derive(squire::Query)]
#[query = "SELECT * FROM users WHERE id = ?"]
struct GetUser(i64);

#[squire::query]
fn get_user(id: i64) -> Result<User> {
    "SELECT * FROM users WHERE id = :id"
}

db.submit(get_user(101)).await

#[derive(Columns)]
pub struct User {
    pub id: squire::RowId,
    pub username: String,
    pub score: f64,
}

#[derive(Bind)]
#[bind(sequential)]
pub struct InsertUser<'a> {
    pub username: &'a str,
    pub score: f64,
}
```

[crate-shield]: https://img.shields.io/crates/v/squire.svg?label=%20&logo=data:image/svg%2bxml;base64,PHN2ZyBoZWlnaHQ9IjE2IiB3aWR0aD0iMTYiIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyI+PGcgZmlsbD0ibm9uZSIgc3Ryb2tlPSIjRUVFIiA+PHBhdGggc3Ryb2tlPSIjRUVFIiBkPSJNMy43MDYgMS44NjRjNC43NyAxMC45NyA1LjMzOSAxMi4wOTcgNS4zMzkgMTIuMDk3bDUuNjg3LTIuNTU5IiAvPjxwYXRoIGQ9Ik01LjU0NiAxNC41NGExLjI2NyAxLjI2NyAwIDEgMSAyLjUwNy0uMzYzIDEuMjY3IDEuMjY3IDAgMCAxLTIuNTA3LjM2NHoiIHN0cm9rZS13aWR0aD0iLjczMyIgLz48cGF0aCBkPSJNNC4wMzcgMS4yNjZDMS45MjIgMi40MS44NjkgMS4zMjEgMS44NTUgMi45NCIgc3Ryb2tlLXdpZHRoPSIuODgiIC8+PHBhdGggZD0iTTcuNjg4IDguMzU5bDUuNTctMi40MTYiIHN0cm9rZS13aWR0aD0iLjY0MyIgLz48cGF0aCBkPSJNNS41NTYgMy40OTdMMTEuMzIuOTg0IiBzdHJva2Utd2lkdGg9Ii42NjYiIC8+PC9nPjxwYXRoIGZpbGw9IiNFRUUiIGZpbGwtcnVsZT0iZXZlbm9kZCIgZD0iTTExLjk1OSA5LjQ0NmMtLjA2My0uMTU4LS4xNzQtLjE3Mi0uNzA3LS4xMTQtLjQ4Ny4wNTctLjg3Mi0uMDI3LTEuMDM5LS40NS0uMTY5LS40MjYuMDczLS44OS43NjMtMS4xNjIuNDA4LS4xNjIuNzk3LS4xNzQgMS4wNTYtLjEwNGwtLjEyMS41NDJhMS4zMiAxLjMyIDAgMCAwLS43MTcuMDc0Yy0uMjMxLjA5MS0uMzE3LjE5NC0uMjY4LjMxNi4wNDguMTIzLjE2My4xMzUuNjkuMDg0LjUxOS0uMDUzLjg4Ni0uMDE1IDEuMDczLjQ1Ny4yMS41MzEtLjIxIDEuMDA3LS44MjggMS4yNTItLjQ2Ny4xODUtLjg4LjE5LTEuMjE2LjA3MmwuMjA1LS41NDRjLjI0NC4wNzIuNTIuMDY4LjgwMi0uMDQ0LjIzNi0uMDkzLjM2Ni0uMjI5LjMwNy0uMzc5ek0xMC43MzggNS4xNmwtMS45MS44LS4yMTUtLjUxMy41MTgtLjIxOC0uNjE4LTEuNDc2LS41MTkuMjE4LS4yMTUtLjUxNCAxLjE4OC0uNDk4LjI1Ny40NGMtLjAyNy0uMzgxLjE1NC0uNzAyLjQ2LS44M2ExLjE3IDEuMTcgMCAwIDEgLjMzLS4wODhsLjE4NC43ODJhMS4wMjMgMS4wMjMgMCAwIDAtLjMwNC4wODNjLS4zMTEuMTMtLjM4LjQ1NC0uMzIuODIybC4zMTIuNzQ1LjYzNi0uMjY3em0tLjg5NyA2LjcyNmw0LjUyMi0xLjk3M0wxMi44IDYuMzMgOC4yNzggOC4zMDR6TTcuNzA3IDcuMDI1TDEyLjM0NSA1bC0xLjU2My0zLjU4LTQuNjM4IDIuMDIzeiIgLz48L3N2Zz4=
[ci-shield]: https://img.shields.io/github/actions/workflow/status/silverlyra/squire/ci.yml?label=%20&logo=github&logoColor=EEEEEE
[docs-shield]: https://img.shields.io/docsrs/squire?label=%20&logo=data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAACAAAAAgCAQAAADZc7J/AAAAIGNIUk0AAHomAACAhAAA+gAAAIDoAAB1MAAA6mAAADqYAAAXcJy6UTwAAAACYktHRAD/h4/MvwAAAAd0SU1FB+oEGRUaKDzvHysAAAAldEVYdGRhdGU6Y3JlYXRlADIwMjYtMDQtMjVUMjE6MjY6NDArMDA6MDDUoKYVAAAAJXRFWHRkYXRlOm1vZGlmeQAyMDI2LTA0LTI1VDIxOjI2OjQwKzAwOjAwpf0eqQAAACh0RVh0ZGF0ZTp0aW1lc3RhbXAAMjAyNi0wNC0yNVQyMToyNjo0MCswMDowMPLoP3YAAAFsSURBVEjH7ZTddZtAEIU/dFzAdmA6CCXgCqIO7FQQdRBSgXAFSiqIXYHUgegAOmA7+PIgBMgGDsfHb8mdB9iFO3N3fhb+4/Ng6t7aJZz9YfqWl3T0jCMh0hBnAgQygIaHpJmK3mphkAVLLVVrw3sHB90tkq9WqhZv6UHrVXQMttqONWyADF5Xpjryu0/H4KCB712IG2WdtexHuw1AeuMgafhGRcMMArtxyIVOOHpz3gGtab97UM0H1t28w5/9uU+9vEAOMTmtUjBle9Vy5RHel3A/0UiblfUD4qXvU9o+ObWPyVgB+WjJoXu+8tJnoCYAkQqAjBBXVWGoQanqsVud1Y2PdpLImcGuf7tEvtYkhXhHEUM1STtdZfKl3/tFxn1X4ECACtvlQTqq7eSXrWq5oUrHrT2JMPnHV4AXfBqSMqdAi4nrRa377J/NFx2MpwExv1yfTxcHwT9+BAUMl+qWLffru5LI88xI/YP4C2X76vn+mE55AAAAAElFTkSuQmCC
[license-shield]: https://img.shields.io/crates/l/squire.svg?label=%20
