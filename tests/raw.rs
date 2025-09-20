use std::error::Error;

use squire::{Connection, Database, RowId, ffi};

type Result<T = ()> = std::result::Result<T, Box<dyn Error>>;

fn setup() -> Result<Connection> {
    let connection = Connection::open(Database::memory())?;

    {
        let (mut create, _) = ffi::Statement::prepare(
            connection.internal_ref(),
            "CREATE TABLE example (id INTEGER PRIMARY KEY AUTOINCREMENT, a TEXT NOT NULL, b INTEGER, c REAL) STRICT;",
            0,
        )?;
        create.binding().ready().execute::<()>()?;

        create.close()?;
    }

    Ok(connection)
}

#[test]
fn round_trip() -> Result {
    let connection = setup()?;

    let id = {
        let (mut insert, _) = ffi::Statement::prepare(
            connection.internal_ref(),
            "INSERT INTO example (a, b, c) VALUES (?, ?, ?);",
            0,
        )?;

        let id = {
            let mut binding = insert.binding();

            unsafe {
                let index = ffi::Index::INITIAL;
                binding.set(index, "hello 🌎!")?;

                let index = index.next();
                binding.set(index, 12)?;

                let index = index.next();
                binding.set(index, 3.14)?;
            }

            binding.ready().execute::<Option<RowId>>()?
        };
        id.expect("inserted row").into_inner()
    };

    {
        let (mut select, _) = ffi::Statement::prepare(
            connection.internal_ref(),
            "SELECT a, b, c FROM example WHERE id = ?;",
            0,
        )?;

        {
            let mut binding = select.binding();

            unsafe {
                let index = ffi::Index::new(1)?;
                binding.set(index, id)?;
            }

            let mut exec = binding.ready();

            let row = exec.row()?.expect("a row");

            let a: ffi::Bytes<'_, str> = unsafe { row.fetch(ffi::Column::new(0)) };
            let b: i32 = unsafe { row.fetch(ffi::Column::new(1)) };
            let c: f64 = unsafe { row.fetch(ffi::Column::new(2)) };

            let a = a.into_inner();

            assert_eq!("hello 🌎!", a);
            assert_eq!(12, b);
            assert_eq!(3.14, c);
        }
    }

    Ok(())
}
