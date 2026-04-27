use std::env;
#[cfg(feature = "bundled")]
use std::path::Path;
#[cfg(any(feature = "bindgen", feature = "bundled"))]
use std::path::PathBuf;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result {
    let docs_rs = env::var_os("DOCS_RS").is_some();

    #[cfg(any(feature = "bindgen", feature = "bundled"))]
    let dest = out_path();

    #[cfg(feature = "bundled")]
    let build = build_bundled_sqlite(&dest, docs_rs)?;

    #[cfg(all(feature = "bindgen", feature = "bundled"))]
    let header_path = build.header();
    #[cfg(all(feature = "bindgen", not(feature = "bundled")))]
    let header_path = PathBuf::from("wrapper.h");

    #[cfg(feature = "bindgen")]
    generate_bindings(&header_path, &dest.join("bindings.rs"))?;

    if !docs_rs {
        #[cfg(feature = "static")]
        let linkage = "static";
        #[cfg(not(feature = "static"))]
        let linkage = "dylib";
        println!("cargo:rustc-link-lib={linkage}=sqlite3");
    }

    // Detect features and emit metadata
    #[cfg(feature = "bundled")]
    let metadata = features::Metadata::detect(build.library());
    #[cfg(not(feature = "bundled"))]
    let metadata = features::Metadata::probe(&features::Build::default())?;

    metadata.emit_for_dependents();
    metadata.emit_cfg();

    Ok(())
}

#[cfg(any(feature = "bindgen", feature = "bundled"))]
fn out_path() -> PathBuf {
    env::var_os("OUT_DIR")
        .expect("cargo did not set $OUT_DIR")
        .into()
}

#[cfg(feature = "bundled")]
fn build_bundled_sqlite(dest: &Path, mock: bool) -> Result<sqlite::Build> {
    let location = sqlite::Location::new(dest);

    for source in location.sources() {
        println!("cargo:rerun-if-changed={}", source.display());
    }
    println!("cargo:rustc-link-search={}", location.dest().display());

    let mut configuration = features::Configuration::empty();

    macro_rules! set {
        ($key:ident, $enabled:expr) => {
            configuration.set_enabled(features::FeatureKey::$key, $enabled);
        };
    }

    set!(ApiArmor, cfg!(feature = "armor"));
    set!(AuthorizationCallback, cfg!(feature = "authorization"));
    set!(AutomaticVacuum, cfg!(feature = "auto-vacuum"));
    set!(BlobIo, cfg!(feature = "blob-io"));
    set!(ColumnDeclaredType, cfg!(feature = "decltype"));
    set!(ColumnMetadata, cfg!(feature = "column-metadata"));
    set!(DatabasePageVirtualTable, cfg!(feature = "page-vtab"));
    set!(DatabaseStatisticsVirtualTable, cfg!(feature = "stat-vtab"));
    set!(Fts3, cfg!(feature = "fts3"));
    set!(Fts5, cfg!(feature = "fts5"));
    set!(Geopoly, cfg!(feature = "geopoly"));
    set!(Json, cfg!(feature = "json"));
    set!(LoadExtension, cfg!(feature = "extensions"));
    set!(MemoryStatus, cfg!(feature = "memory-status"));
    set!(NormalizeSql, cfg!(feature = "normalize-sql"));
    set!(Percentile, cfg!(feature = "percentile"));
    set!(PreUpdateHook, cfg!(feature = "preupdate-hook"));
    set!(ProgressCallback, cfg!(feature = "progress-callback"));
    set!(Rtree, cfg!(feature = "rtree"));
    set!(Serialize, cfg!(feature = "serialize"));
    set!(Session, cfg!(feature = "session"));
    set!(SharedCache, cfg!(feature = "shared-cache"));
    set!(Snapshot, cfg!(feature = "snapshot"));
    set!(Soundex, cfg!(feature = "soundex"));
    set!(Stat4, cfg!(feature = "stat4"));
    set!(Trace, cfg!(feature = "trace"));
    set!(VirtualTable, cfg!(feature = "vtab"));

    let mut directives = sqlite::config(Some(&configuration));

    #[cfg(feature = "serialized")]
    let threading = features::directive::Threading::Serialized;
    #[cfg(all(feature = "multi-thread", not(feature = "serialized")))]
    let threading = features::directive::Threading::MultiThread;
    #[cfg(not(feature = "multi-thread"))]
    let threading = features::directive::Threading::SingleThread;
    directives.insert(features::Directive::Threading(threading));

    let build = if !mock {
        sqlite::build(location, directives)
    } else {
        sqlite::Build::new(location, directives)
    };

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
                                    *pat_type.pat = syn::parse_quote! { #ident };
                                }

                                // Replace the destructor parameter type
                                if i == *param_index {
                                    *pat_type.ty = syn::parse_quote! { sqlite3_destructor_type };
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
