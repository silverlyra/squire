type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result {
    // Read feature detection from the sys crate's build script
    let library = features::Library::from_cargo_metadata()?;

    // Emit the same cfg attributes
    library.emit_cfg();

    Ok(())
}
