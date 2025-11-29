type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result {
    // Read feature detection from the sys crate's build script
    let library = features::Library::from_cargo_metadata()?;

    // Verify threading mode compatibility
    #[cfg(feature = "multi-thread")]
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

#[cfg(feature = "multi-thread")]
fn check_threading_mode(library: &features::Library) {
    use features::Threading;

    let actual = library.threading();

    if actual == Threading::SingleThread {
        panic!(
            "multi-thread feature enabled, but SQLite was built with \
                SQLITE_THREADSAFE=0, and can only be used single-threaded"
        );
    }
}
