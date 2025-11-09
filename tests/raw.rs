use std::error::Error;

use squire::{BindIndex, Connection, Database, RowId, ffi};

type Result<T = ()> = std::result::Result<T, Box<dyn Error>>;

fn setup() -> Result<Connection> {
    let connection = Connection::open(Database::memory())?;

    {
        let (create, _) = ffi::Statement::prepare(
            connection.internal_ref(),
            "CREATE TABLE example (id INTEGER PRIMARY KEY AUTOINCREMENT, a TEXT NOT NULL, b INTEGER, c REAL) STRICT;",
            0,
        )?;
        unsafe { create.execute::<()>() }?;
        create.close()?;
    }

    Ok(connection)
}

#[test]
fn round_trip() -> Result {
    let connection = setup()?;

    let id = {
        let (insert, _) = ffi::Statement::prepare(
            connection.internal_ref(),
            "INSERT INTO example (a, b, c) VALUES (?, ?, ?);",
            0,
        )?;

        let index = BindIndex::INITIAL;
        unsafe { insert.bind(index, "hello 🌎!") }?;

        let index = index.next();
        unsafe { insert.bind(index, 12) }?;

        let index = index.next();
        unsafe { insert.bind(index, 3.14) }?;

        let id: Option<RowId> = unsafe { insert.execute() }?;
        insert.close()?;

        id.expect("inserted row").into_inner()
    };

    {
        let (select, _) = ffi::Statement::prepare(
            connection.internal_ref(),
            "SELECT a, b, c FROM example WHERE id = ?;",
            0,
        )?;

        let index = BindIndex::new(1)?;
        unsafe { select.bind(index, id) }?;

        let row = unsafe { select.row() }?;
        assert!(row, "expected a row");

        let a: ffi::Bytes<'_, str> = unsafe { select.fetch(ffi::ColumnIndex::new(0)) };
        let b: i32 = unsafe { select.fetch(ffi::ColumnIndex::new(1)) };
        let c: f64 = unsafe { select.fetch(ffi::ColumnIndex::new(2)) };

        let a = a.into_inner();

        assert_eq!("hello 🌎!", a);
        assert_eq!(12, b);
        assert_eq!(3.14, c);

        select.close()?;
    }

    Ok(())
}
