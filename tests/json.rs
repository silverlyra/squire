#![cfg_attr(
    all(nightly, feature = "lang-array-assume-init"),
    feature(maybe_uninit_array_assume_init)
)]

use std::{collections::HashMap, error::Error};

use serde::{Deserialize, Serialize};
use squire::{Columns, Connection, Database, Parameters};

type Result<T = ()> = std::result::Result<T, Box<dyn Error>>;

fn setup() -> Result<Connection> {
    let connection = Connection::open(Database::memory())?;

    connection.execute(
        "CREATE TABLE records (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            metadata BLOB NOT NULL,
            settings BLOB NOT NULL
        ) STRICT;",
        (),
    )?;

    Ok(connection)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Settings {
    theme: String,
    language: String,
    notifications: bool,
}

#[derive(Parameters)]
struct InsertRecord<'a> {
    #[squire(json)]
    metadata: &'a HashMap<String, String>,
    #[squire(json)]
    settings: &'a Settings,
}

#[derive(Columns)]
struct Record {
    #[squire(json)]
    metadata: HashMap<String, String>,
    #[squire(json)]
    settings: Settings,
}

#[test]
fn json_round_trip() -> Result {
    let connection = setup()?;

    let mut metadata = HashMap::new();
    metadata.insert("author".to_string(), "Alice".to_string());
    metadata.insert("version".to_string(), "1.0".to_string());

    let settings = Settings {
        theme: "dark".to_string(),
        language: "en".to_string(),
        notifications: true,
    };

    // Insert with JSON serialization
    let mut insert = connection
        .prepare("INSERT INTO records (metadata, settings) VALUES (:metadata, :settings);")?;
    let id = insert.insert(InsertRecord {
        metadata: &metadata,
        settings: &settings,
    })?;

    // Fetch with JSON deserialization
    let mut query = connection.prepare("SELECT metadata, settings FROM records WHERE id = ?;")?;
    let record: Record = query.query(id)?.rows()?.next()?.ok_or("not found")?;

    assert_eq!(metadata, record.metadata);
    assert_eq!(settings, record.settings);

    Ok(())
}

#[cfg(feature = "jsonb")]
mod jsonb_tests {
    use super::*;

    #[derive(Parameters)]
    struct InsertRecordJsonb<'a> {
        #[squire(jsonb)]
        metadata: &'a HashMap<String, String>,
        #[squire(jsonb)]
        settings: &'a Settings,
    }

    #[derive(Columns)]
    struct RecordJsonb {
        #[squire(jsonb)]
        metadata: HashMap<String, String>,
        #[squire(jsonb)]
        settings: Settings,
    }

    #[test]
    fn jsonb_round_trip() -> Result {
        let connection = setup()?;

        let mut metadata = HashMap::new();
        metadata.insert("author".to_string(), "Bob".to_string());
        metadata.insert("version".to_string(), "2.0".to_string());

        let settings = Settings {
            theme: "light".to_string(),
            language: "es".to_string(),
            notifications: false,
        };

        // Insert with JSONB serialization
        let mut insert = connection
            .prepare("INSERT INTO records (metadata, settings) VALUES (:metadata, :settings);")?;
        let id = insert.insert(InsertRecordJsonb {
            metadata: &metadata,
            settings: &settings,
        })?;

        // Fetch with JSONB deserialization
        let mut query =
            connection.prepare("SELECT metadata, settings FROM records WHERE id = ?;")?;
        let record: RecordJsonb = query.query(id)?.rows()?.next()?.ok_or("not found")?;

        assert_eq!(metadata, record.metadata);
        assert_eq!(settings, record.settings);

        Ok(())
    }
}

#[test]
fn json_nested_structure() -> Result {
    let connection = setup()?;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct NestedData {
        items: Vec<String>,
        count: usize,
        nested: HashMap<String, Vec<i32>>,
    }

    #[derive(Parameters)]
    struct InsertNested<'a> {
        #[squire(json)]
        metadata: &'a NestedData,
        #[squire(json)]
        settings: &'a Settings,
    }

    #[derive(Columns)]
    struct RecordNested {
        #[squire(json)]
        metadata: NestedData,
    }

    let nested_data = NestedData {
        items: vec!["foo".to_string(), "bar".to_string(), "baz".to_string()],
        count: 3,
        nested: {
            let mut map = HashMap::new();
            map.insert("a".to_string(), vec![1, 2, 3]);
            map.insert("b".to_string(), vec![4, 5, 6]);
            map
        },
    };

    let settings = Settings {
        theme: "auto".to_string(),
        language: "fr".to_string(),
        notifications: true,
    };

    let mut insert = connection
        .prepare("INSERT INTO records (metadata, settings) VALUES (:metadata, :settings);")?;
    let id = insert.insert(InsertNested {
        metadata: &nested_data,
        settings: &settings,
    })?;

    let mut query = connection.prepare("SELECT metadata FROM records WHERE id = ?;")?;
    let record: RecordNested = query.query(id)?.rows()?.next()?.ok_or("not found")?;

    assert_eq!(nested_data, record.metadata);

    Ok(())
}
