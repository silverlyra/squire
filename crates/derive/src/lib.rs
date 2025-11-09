mod column;
mod common;
mod param;

use column::ColumnsDerive;
use darling::FromDeriveInput;
use param::ParametersDerive;
use proc_macro::TokenStream;

/// Derive macro for implementing the `Parameters` trait.
///
/// # Attributes
///
/// - `#[squire(skip)]` - Skip this field when binding parameters
/// - `#[squire(borrow)]` - Wrap the field in `Borrowed` for zero-copy binding
/// - `#[squire(bind_with = custom_function)]` - Use a custom binding function
/// - `#[squire(index = 1)]` - Use a specific parameter index
/// - `#[squire(rename = other_name)]` - Use a different field name for binding
#[proc_macro_derive(Parameters, attributes(squire))]
pub fn derive_parameters(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    match ParametersDerive::from_derive_input(&input) {
        Ok(params) => match params.derive() {
            Ok(tokens) => tokens.into(),
            Err(err) => err.write_errors().into(),
        },
        Err(err) => err.write_errors().into(),
    }
}

/// Derive macro for implementing the `Columns` trait.
///
/// # Attributes
///
/// - `#[squire(skip)]` - Skip this field when fetching columns
/// - `#[squire(borrow)]` - Fetch the field as `Borrowed` for zero-copy access
/// - `#[squire(fetch_with = custom_function)]` - Use a custom fetch function
/// - `#[squire(index = 0)]` - Use a specific column index
/// - `#[squire(rename = other_name)]` - Use a different field name for column lookup
/// - `#[squire(result)]` - Unwrap a Result returned by the fetch expression
#[proc_macro_derive(Columns, attributes(squire))]
pub fn derive_columns(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    match ColumnsDerive::from_derive_input(&input) {
        Ok(columns) => match columns.derive() {
            Ok(tokens) => tokens.into(),
            Err(err) => err.write_errors().into(),
        },
        Err(err) => err.write_errors().into(),
    }
}
