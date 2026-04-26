#![cfg_attr(
    all(nightly, feature = "lang-array-assume-init"),
    feature(maybe_uninit_array_assume_init)
)]
#![allow(clippy::approx_constant)]

use std::error::Error;

use squire::{Connection, Memory, Parameters};

type Result<T = ()> = std::result::Result<T, Box<dyn Error>>;

fn setup() -> Result<Connection> {
    let connection = Connection::open(Memory)?;

    connection.execute(
        "CREATE TABLE example (id INTEGER PRIMARY KEY AUTOINCREMENT, a TEXT NOT NULL, b INTEGER, c REAL) STRICT;",
        ()
    )?;

    Ok(connection)
}

#[derive(Parameters)]
struct Row<'a> {
    #[squire(borrow)]
    a: &'a str,
    b: isize,
    c: f64,
}

#[test]
fn round_trip() -> Result {
    let connection = setup()?;

    let mut insert = connection.prepare("INSERT INTO example (a, b, c) VALUES (:a, :b, :c);")?;
    let id = insert.insert(Row {
        a: "hello 🌎!",
        b: 42,
        c: 3.14,
    })?;

    let mut query = connection.prepare("SELECT a, b, c FROM example WHERE id = ?;")?;
    let (a, b, c): (String, i64, f64) = query.query(id)?.rows()?.next()?.ok_or("not found")?;

    assert_eq!("hello 🌎!", a);
    assert_eq!(42, b);
    assert_eq!(3.14, c);

    Ok(())
}
