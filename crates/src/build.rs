use std::{env, fmt, fs, path::PathBuf};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result {
    let crate_path = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let crate_version = crate_version();

    let sqlite_path = crate_path.join("sqlite");
    let version_path = sqlite_path.join("VERSION");
    let mut sqlite_version = fs::read_to_string(version_path)?;
    sqlite_version.truncate(sqlite_version.len() - 1);

    if crate_version == sqlite_version {
        println!("cargo::rustc-env=SQUIRE_SQLITE_VERSION={sqlite_version}");
        println!("cargo::rerun-if-changed=sqlite/VERSION");

        Ok(())
    } else {
        Err(VersionMismatchError {
            sqlite: sqlite_path,
            expected: crate_version,
            actual: sqlite_version,
        }
        .into())
    }
}

fn crate_version() -> String {
    let major = env::var("CARGO_PKG_VERSION_MAJOR").unwrap();
    let minor = env::var("CARGO_PKG_VERSION_MINOR").unwrap();
    let patch = env::var("CARGO_PKG_VERSION_PATCH").unwrap();

    format!("{major}.{minor}.{patch}")
}

#[derive(Clone, Debug)]
struct VersionMismatchError {
    sqlite: PathBuf,
    expected: String,
    actual: String,
}

impl fmt::Display for VersionMismatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let VersionMismatchError {
            sqlite,
            expected,
            actual,
        } = self;
        let sqlite = &sqlite.display();

        write!(
            f,
            "expected SQLite {expected}, but found {actual} at {sqlite}"
        )
    }
}

impl std::error::Error for VersionMismatchError {}
