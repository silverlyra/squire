#[cfg(any(feature = "bindgen", feature = "bundled"))]
use std::env;
#[cfg(feature = "bundled")]
use std::path::Path;
#[cfg(any(feature = "bindgen", feature = "bundled"))]
use std::path::PathBuf;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result {
    #[cfg(any(feature = "bindgen", feature = "bundled"))]
    let dest = out_path();

    #[cfg(all(not(feature = "bindgen"), feature = "bundled"))]
    build_bundled_sqlite(&dest)?;

    #[cfg(all(feature = "bindgen", feature = "bundled"))]
    let header_path = {
        let build = build_bundled_sqlite(&dest)?;
        build.header()
    };
    #[cfg(all(feature = "bindgen", not(feature = "bundled")))]
    let header_path = PathBuf::from("wrapper.h");

    #[cfg(feature = "bindgen")]
    generate_bindings(&header_path, &dest.join("bindings.rs"))?;

    {
        #[cfg(feature = "static")]
        let linkage = "static";
        #[cfg(not(feature = "static"))]
        let linkage = "dylib";
        println!("cargo:rustc-link-lib={linkage}=sqlite3");
    }

    // Detect features and emit metadata
    #[cfg(feature = "bundled")]
    let library = features::Library::new(
        features::Version::new(3, 51, 0), // Update this when bundled version changes
        threading_mode(),
        bundled_features(),
    );
    #[cfg(not(feature = "bundled"))]
    let library = {
        use features::build;
        let probe = build::Library::probe(build::Build::default());
        features::Library::probe(&probe)
    };

    library.emit_cargo_metadata();
    library.emit_cfg();

    Ok(())
}

#[cfg(feature = "bundled")]
fn threading_mode() -> features::Threading {
    #[cfg(feature = "serialized")]
    return features::Threading::Serialized;
    #[cfg(all(feature = "multi-thread", not(feature = "serialized")))]
    return features::Threading::MultiThread;
    #[cfg(not(feature = "multi-thread"))]
    return features::Threading::SingleThread;
}

#[cfg(feature = "bundled")]
fn bundled_features() -> impl Iterator<Item = features::FeatureKey> {
    use features::FeatureKey::*;

    let all = [(Json, cfg!(feature = "json")), (PrepareQuiet, true)];
    all.into_iter()
        .filter_map(|(key, enabled)| if enabled { Some(key) } else { None })
}

#[cfg(any(feature = "bindgen", feature = "bundled"))]
fn out_path() -> PathBuf {
    env::var_os("OUT_DIR")
        .expect("cargo did not set $OUT_DIR")
        .into()
}

#[cfg(feature = "bundled")]
fn build_bundled_sqlite(dest: &Path) -> Result<sqlite::Build> {
    let location = sqlite::Location::new(dest);

    for source in location.sources() {
        println!("cargo:rerun-if-changed={}", source.display());
    }
    println!("cargo:rustc-link-search={}", location.dest().display());

    let mut config = sqlite::Config::default();

    #[cfg(feature = "serialized")]
    config.set(sqlite::Setting::Threading(sqlite::Threading::Serialized));
    #[cfg(all(feature = "multi-thread", not(feature = "serialized")))]
    config.set(sqlite::Setting::Threading(sqlite::Threading::MultiThread));
    #[cfg(not(feature = "multi-thread"))]
    config.set(sqlite::Setting::Threading(sqlite::Threading::SingleThread));

    #[cfg(feature = "armor")]
    config.set(sqlite::Setting::EnableApiArmor(true));
    #[cfg(feature = "authorization")]
    config.set(sqlite::Setting::EnableAuthorization(true));
    #[cfg(feature = "auto-vacuum")]
    config.set(sqlite::Setting::DefaultAutomaticVacuum(true));
    #[cfg(feature = "blob-io")]
    config.set(sqlite::Setting::EnableBlobIo(true));
    #[cfg(feature = "column-metadata")]
    config.set(sqlite::Setting::EnableColumnMetadata(true));
    #[cfg(feature = "decltype")]
    config.set(sqlite::Setting::EnableColumnDeclaredType(true));
    #[cfg(feature = "extensions")]
    config.set(sqlite::Setting::EnableLoadExtension(true));
    #[cfg(feature = "fts3")]
    config.set(sqlite::Setting::EnableFts3(true));
    #[cfg(feature = "fts5")]
    config.set(sqlite::Setting::EnableFts5(true));
    #[cfg(feature = "geopoly")]
    config.set(sqlite::Setting::EnableGeopoly(true));
    #[cfg(feature = "json")]
    config.set(sqlite::Setting::EnableJson(true));
    #[cfg(feature = "memory-status")]
    config.set(sqlite::Setting::DefaultMemoryStatus(true));
    #[cfg(feature = "normalize-sql")]
    config.set(sqlite::Setting::EnableNormalizeSql(true));
    #[cfg(feature = "page-vtab")]
    config.set(sqlite::Setting::EnableDatabasePagesVirtualTable(true));
    #[cfg(feature = "preupdate-hook")]
    config.set(sqlite::Setting::EnablePreUpdateHook(true));
    #[cfg(feature = "progress-callback")]
    config.set(sqlite::Setting::EnableProgressCallback(true));
    #[cfg(feature = "rtree")]
    config.set(sqlite::Setting::EnableRtree(true));
    #[cfg(feature = "serialize")]
    config.set(sqlite::Setting::EnableSerialize(true));
    #[cfg(feature = "session")]
    config.set(sqlite::Setting::EnableSession(true));
    #[cfg(feature = "shared-cache")]
    config.set(sqlite::Setting::EnableSharedCache(true));
    #[cfg(feature = "snapshot")]
    config.set(sqlite::Setting::EnableSnapshot(true));
    #[cfg(feature = "soundex")]
    config.set(sqlite::Setting::EnableSoundex(true));
    #[cfg(feature = "stat-vtab")]
    config.set(sqlite::Setting::EnableDatabaseStatisticsVirtualTable(true));
    #[cfg(feature = "stat4")]
    config.set(sqlite::Setting::EnableStat4(true));
    #[cfg(feature = "trace")]
    config.set(sqlite::Setting::EnableTrace(true));
    #[cfg(feature = "vtab")]
    config.set(sqlite::Setting::EnableVirtualTables(true));
    #[cfg(feature = "wal")]
    config.set(sqlite::Setting::EnableWriteAheadLog(true));

    #[cfg(not(feature = "json"))]
    config.set(sqlite::Setting::EnableJson(false));

    let build = sqlite::build(location, config);

    Ok(build)
}

#[cfg(feature = "bindgen")]
fn generate_bindings(header: &Path, dest: &Path) -> Result {
    let config = bindgen::builder()
        .header(header.to_str().expect("non UTF-8 sqlite3.h path"))
        .default_macro_constant_type(bindgen::MacroTypeVariation::Signed)
        .generate_cstr(true)
        .blocklist_function("sqlite3_(str_)?v[msn]*(print|append)f")
        .blocklist_type("va_list")
        .blocklist_item("^__.*")
        // Block the original sqlite3_destructor_type so we can replace it
        .blocklist_type("sqlite3_destructor_type");

    let config = if cfg!(target_vendor = "apple") {
        config.blocklist_item("^MAC_OS_(X_)?VERSION_.*")
    } else {
        config
    };

    #[cfg(not(feature = "blob-io"))]
    let config = config.blocklist_function("sqlite3_blob_\\w+");
    #[cfg(not(feature = "utf-16"))]
    let config = config.blocklist_function("sqlite3_\\w+16(be|le)?(_v\\d)?");

    let bindings = config.generate()?;

    // Parse the generated bindings using syn
    let bindings_str = bindings.to_string();
    let mut syntax_tree: syn::File = syn::parse_str(&bindings_str)
        .map_err(|e| format!("Failed to parse generated bindings: {}", e))?;

    // Functions that need their destructor parameters replaced and parameter names updated
    let target_functions = [
        // bind functions - 5th parameter (index 4)
        (
            "sqlite3_bind_blob",
            4,
            vec!["pStmt", "parameter", "value", "len", "destructor"],
        ),
        (
            "sqlite3_bind_blob64",
            4,
            vec!["pStmt", "parameter", "value", "len", "destructor"],
        ),
        (
            "sqlite3_bind_text",
            4,
            vec!["pStmt", "parameter", "value", "len", "destructor"],
        ),
        (
            "sqlite3_bind_text16",
            4,
            vec!["pStmt", "parameter", "value", "len", "destructor"],
        ),
        (
            "sqlite3_bind_text64",
            4,
            vec![
                "pStmt",
                "parameter",
                "value",
                "len",
                "destructor",
                "encoding",
            ],
        ),
        (
            "sqlite3_bind_pointer",
            4,
            vec!["pStmt", "parameter", "value", "type_name", "destructor"],
        ),
        // result functions - 4th parameter (index 3)
        (
            "sqlite3_result_blob",
            3,
            vec!["context", "value", "len", "destructor"],
        ),
        (
            "sqlite3_result_blob64",
            3,
            vec!["context", "value", "len", "destructor"],
        ),
        (
            "sqlite3_result_text",
            3,
            vec!["context", "value", "len", "destructor"],
        ),
        (
            "sqlite3_result_text64",
            3,
            vec!["context", "value", "len", "destructor", "encoding"],
        ),
        (
            "sqlite3_result_text16",
            3,
            vec!["context", "value", "len", "destructor"],
        ),
        (
            "sqlite3_result_text16le",
            3,
            vec!["context", "value", "len", "destructor"],
        ),
        (
            "sqlite3_result_text16be",
            3,
            vec!["context", "value", "len", "destructor"],
        ),
        (
            "sqlite3_result_pointer",
            3,
            vec!["context", "value", "type_name", "destructor"],
        ),
    ];

    // Transform the AST to replace destructor parameters
    for item in &mut syntax_tree.items {
        if let syn::Item::ForeignMod(foreign_mod) = item {
            for foreign_item in &mut foreign_mod.items {
                if let syn::ForeignItem::Fn(func) = foreign_item {
                    let func_name = func.sig.ident.to_string();

                    // Check if this function needs parameter substitution and renaming
                    if let Some((_, param_index, param_names)) = target_functions
                        .iter()
                        .find(|(name, _, _)| func_name == *name)
                    {
                        // Replace parameter names and the destructor type
                        for (i, param) in func.sig.inputs.iter_mut().enumerate() {
                            if let syn::FnArg::Typed(pat_type) = param {
                                // Update parameter name if we have one
                                if let Some(param_name) = param_names.get(i) {
                                    let ident =
                                        syn::Ident::new(param_name, proc_macro2::Span::call_site());
                                    pat_type.pat = Box::new(syn::parse_quote! { #ident });
                                }

                                // Replace the destructor parameter type
                                if i == *param_index {
                                    pat_type.ty =
                                        Box::new(syn::parse_quote! { sqlite3_destructor_type });
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Convert the modified AST back to a string using prettyplease for proper formatting
    let mut modified_bindings = prettyplease::unparse(&syntax_tree);

    // Add our custom union type definition at the end
    modified_bindings.push_str(include_str!("src/bindings/destructor.rs"));

    std::fs::write(dest, modified_bindings)?;

    Ok(())
}
