type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result {
    // Read feature detection from the sys crate's build script
    let library = features::Library::from_cargo_metadata()?;

    // Verify threading mode compatibility
    check_threading_mode(&library);

    // Emit cfg attributes for SQLite feature detection
    library.emit_cfg();

    // Emit `nightly` cfg when compiling on a nightly/dev toolchain, so that
    // `--all-features` (including `nightly` and `lang-*` features) can be used
    // on a stable or beta toolchain.
    emit_nightly_cfg();

    Ok(())
}

fn emit_nightly_cfg() {
    println!("cargo::rustc-check-cfg=cfg(nightly)");

    #[cfg(feature = "rustversion")]
    if rustversion::cfg!(nightly) {
        println!("cargo:rustc-cfg=nightly");
    }
}

/// Check that the SQLite library's threading mode is compatible with the
/// requested features.
///
/// The `multi-thread` feature (which `serialized` implies) requires SQLite
/// built with SQLITE_THREADSAFE=1 or 2. If the library is single-threaded,
/// we fail the build.
///
/// Note: We don't require serialized mode even when the `serialized` feature
/// is enabled, because we pass `SQLITE_OPEN_FULLMUTEX` at connection open time
/// to upgrade multi-thread mode to serialized behavior.
fn check_threading_mode(library: &features::Library) {
    use features::Threading;

    #[allow(unused_variables)]
    let actual = library.threading();

    #[cfg(feature = "multi-thread")]
    {
        if actual == Threading::SingleThread {
            panic!(
                "multi-thread feature enabled, but SQLite was built with \
                SQLITE_THREADSAFE=0, and can only be used single-threaded"
            );
        }
    }
}
