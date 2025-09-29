use std::error::Error;

use squire::{Connection, Database};

type Result<T = ()> = std::result::Result<T, Box<dyn Error>>;

fn setup() -> Result<Connection> {
    let connection = Connection::open(Database::memory())?;

    connection.execute(
        "CREATE TABLE example (id INTEGER PRIMARY KEY AUTOINCREMENT, a TEXT NOT NULL, b INTEGER, c REAL) STRICT;",
        ()
    )?;

    Ok(connection)
}

#[test]
fn round_trip() -> Result {
    let connection = setup()?;

    let mut insert = connection.prepare("INSERT INTO example (a, b, c) VALUES (?, ?, ?);")?;
    let id = insert.insert(("hello 🌎!", 42, 3.14))?;

    let mut query = connection.prepare("SELECT a, b, c FROM example WHERE id = ?;")?;
    let (a, b, c): (String, i64, f64) = query.query(id)?.next()?.ok_or("not found")?;

    assert_eq!("hello 🌎!", a);
    assert_eq!(42, b);
    assert_eq!(3.14, c);

    Ok(())
}
