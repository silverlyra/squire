use std::error::Error;

use squire::{Columns, Connection, Database};

type Result<T = ()> = std::result::Result<T, Box<dyn Error>>;

fn setup() -> Result<Connection> {
    let connection = Connection::open(Database::memory())?;

    connection.execute(
        "CREATE TABLE example (id INTEGER PRIMARY KEY AUTOINCREMENT, a TEXT NOT NULL, b INTEGER, c REAL) STRICT;",
        ()
    )?;

    connection.execute(
        "INSERT INTO example (a, b, c) VALUES ('hello 🌎!', 42, 3.14);",
        (),
    )?;

    Ok(connection)
}

#[derive(Columns)]
struct Row {
    a: String,
    b: i64,
    c: f64,
}

#[test]
fn fetch_named_struct() -> Result {
    let connection = setup()?;

    let mut query = connection.prepare("SELECT a, b, c FROM example WHERE id = 1;")?;
    let row: Row = query.query(())?.rows()?.next()?.ok_or("not found")?;

    assert_eq!("hello 🌎!", row.a);
    assert_eq!(42, row.b);
    assert_eq!(3.14, row.c);

    Ok(())
}

#[derive(Columns)]
struct BorrowedRow<'a> {
    #[squire(borrow)]
    a: &'a str,
    b: i64,
    c: f64,
}

#[test]
fn fetch_borrowed_struct() -> Result {
    let connection = setup()?;

    let mut query = connection.prepare("SELECT a, b, c FROM example WHERE id = 1;")?;
    let mut rows = query.query(())?.rows()?;
    let row: BorrowedRow = rows.next()?.ok_or("not found")?;

    assert_eq!("hello 🌎!", row.a);
    assert_eq!(42, row.b);
    assert_eq!(3.14, row.c);

    Ok(())
}

#[derive(Columns)]
struct RowTuple(String, i64, f64);

#[test]
fn fetch_tuple_struct() -> Result {
    let connection = setup()?;

    let mut query = connection.prepare("SELECT a, b, c FROM example WHERE id = 1;")?;
    let row: RowTuple = query.query(())?.rows()?.next()?.ok_or("not found")?;

    assert_eq!("hello 🌎!", row.0);
    assert_eq!(42, row.1);
    assert_eq!(3.14, row.2);

    Ok(())
}

#[derive(Columns)]
#[squire(sequential)]
struct RowSequential {
    a: String,
    b: i64,
    c: f64,
}

#[test]
fn fetch_sequential() -> Result {
    let connection = setup()?;

    let mut query = connection.prepare("SELECT a, b, c FROM example WHERE id = 1;")?;
    let row: RowSequential = query.query(())?.rows()?.next()?.ok_or("not found")?;

    assert_eq!("hello 🌎!", row.a);
    assert_eq!(42, row.b);
    assert_eq!(3.14, row.c);

    Ok(())
}
