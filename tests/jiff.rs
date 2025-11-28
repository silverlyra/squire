#![cfg(feature = "jiff")]

use std::error::Error;

use jiff::{SignedDuration, Span, SpanFieldwise, Timestamp, Zoned};
use squire::{Connection, Database};

type Result<T = ()> = std::result::Result<T, Box<dyn Error>>;

fn connection() -> Result<Connection> {
    Ok(Connection::open(Database::memory())?)
}

// SignedDuration tests

#[test]
fn signed_duration_round_trip() -> Result {
    let conn = connection()?;

    let duration = SignedDuration::new(3661, 123_456_789); // 1h 1m 1s with full nanos

    let mut stmt = conn.prepare("SELECT ?")?;
    let (fetched,): (SignedDuration,) = stmt.query(duration)?.rows()?.next()?.ok_or("no row")?;

    assert_eq!(duration, fetched);
    Ok(())
}

#[test]
fn signed_duration_negative() -> Result {
    let conn = connection()?;

    let duration = SignedDuration::new(-90, -250_000_000); // -90.25 seconds

    let mut stmt = conn.prepare("SELECT ?")?;
    let (fetched,): (SignedDuration,) = stmt.query(duration)?.rows()?.next()?.ok_or("no row")?;

    assert_eq!(duration, fetched);
    Ok(())
}

#[test]
fn signed_duration_column_representation() -> Result {
    let conn = connection()?;

    let duration = SignedDuration::new(1, 500_000_000); // 1.5 seconds

    let mut stmt = conn.prepare("SELECT ?")?;
    let (value,): (i64,) = stmt.query(duration)?.rows()?.next()?.ok_or("no row")?;

    assert_eq!(value, 1_500_000_000); // nanoseconds
    Ok(())
}

// Timestamp tests

#[test]
fn timestamp_round_trip() -> Result {
    let conn = connection()?;

    // i64 nanoseconds gives exact precision
    let ts = Timestamp::new(1700000000, 123_456_789)?; // 2023-11-14 with full nanos

    let mut stmt = conn.prepare("SELECT ?")?;
    let (fetched,): (Timestamp,) = stmt.query(ts)?.rows()?.next()?.ok_or("no row")?;

    assert_eq!(ts, fetched);
    Ok(())
}

#[test]
fn timestamp_negative() -> Result {
    let conn = connection()?;

    let ts = Timestamp::new(-86400, 0)?; // 1969-12-31

    let mut stmt = conn.prepare("SELECT ?")?;
    let (fetched,): (Timestamp,) = stmt.query(ts)?.rows()?.next()?.ok_or("no row")?;

    assert_eq!(ts, fetched);
    Ok(())
}

#[test]
fn timestamp_column_representation() -> Result {
    let conn = connection()?;

    let ts = Timestamp::new(1000, 500_000_000)?; // 1000.5 seconds since epoch

    let mut stmt = conn.prepare("SELECT ?")?;
    let (value,): (i64,) = stmt.query(ts)?.rows()?.next()?.ok_or("no row")?;

    assert_eq!(value, 1000_500_000_000); // nanoseconds
    Ok(())
}

// Span tests

#[test]
fn span_round_trip() -> Result {
    let conn = connection()?;

    let span = Span::new()
        .years(1)
        .months(2)
        .days(3)
        .hours(4)
        .minutes(5)
        .seconds(6);

    let mut stmt = conn.prepare("SELECT ?")?;
    let (fetched,): (Span,) = stmt.query(span)?.rows()?.next()?.ok_or("no row")?;

    assert_eq!(SpanFieldwise(span), SpanFieldwise(fetched));
    Ok(())
}

#[test]
fn span_column_representation() -> Result {
    let conn = connection()?;

    let span = Span::new().hours(2).minutes(30);

    let mut stmt = conn.prepare("SELECT ?")?;
    let (value,): (String,) = stmt.query(span)?.rows()?.next()?.ok_or("no row")?;

    assert_eq!(value, "PT2H30M");
    Ok(())
}

#[test]
fn span_fetch_error() -> Result {
    let conn = connection()?;

    let mut stmt = conn.prepare("SELECT ?")?;
    let mut rows = stmt.query("not a valid span")?.rows::<(Span,)>()?;
    let result = rows.next();

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.is_squire());
    assert!(err.is_integration());
    assert!(err.as_integration().unwrap().is_jiff());
    assert!(err.as_integration().unwrap().as_jiff().is_some());

    Ok(())
}

// Zoned tests

#[test]
fn zoned_round_trip() -> Result {
    let conn = connection()?;

    let zoned: Zoned = "2024-06-15T14:30:00-04:00[America/New_York]".parse()?;

    let mut stmt = conn.prepare("SELECT ?")?;
    let (fetched,): (Zoned,) = stmt.query(zoned.clone())?.rows()?.next()?.ok_or("no row")?;

    assert_eq!(zoned, fetched);
    Ok(())
}

#[test]
fn zoned_column_representation() -> Result {
    let conn = connection()?;

    let zoned: Zoned = "2024-01-01T00:00:00+00:00[UTC]".parse()?;

    let mut stmt = conn.prepare("SELECT ?")?;
    let (value,): (String,) = stmt.query(zoned)?.rows()?.next()?.ok_or("no row")?;

    assert_eq!(value, "2024-01-01T00:00:00+00:00[UTC]");
    Ok(())
}

#[test]
fn zoned_fetch_error() -> Result {
    let conn = connection()?;

    let mut stmt = conn.prepare("SELECT ?")?;
    let mut rows = stmt
        .query("not a valid zoned datetime")?
        .rows::<(Zoned,)>()?;
    let result = rows.next();

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.is_squire());
    assert!(err.is_integration());
    assert!(err.as_integration().unwrap().is_jiff());
    assert!(err.as_integration().unwrap().as_jiff().is_some());

    Ok(())
}
