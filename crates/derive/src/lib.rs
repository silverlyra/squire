mod param;

use darling::FromDeriveInput;
use param::Parameters;
use proc_macro::TokenStream;

/// Derive macro for implementing the `Parameters` trait.
///
/// # Attributes
///
/// - `#[squire(skip)]` - Skip this field when binding parameters
/// - `#[squire(borrow)]` - Wrap the field in `Static` for zero-copy binding
/// - `#[squire(borrow = "'a + 'b")]` - Wrap with explicit lifetime bounds
/// - `#[squire(owned)]` - Wrap the field in `Owned` to transfer ownership to SQLite
/// - `#[squire(shrunk)]` - Wrap the field in `Shrunk` for memory-efficient ownership transfer
/// - `#[squire(with = custom_function)]` - Use a custom binding function
/// - `#[squire(index = 1)]` - Use a specific parameter index
/// - `#[squire(rename = other_name)]` - Use a different field name for binding
#[proc_macro_derive(Parameters, attributes(squire))]
pub fn derive_parameters(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    match Parameters::from_derive_input(&input) {
        Ok(params) => match params.derive() {
            Ok(tokens) => tokens.into(),
            Err(err) => err.write_errors().into(),
        },
        Err(err) => err.write_errors().into(),
    }
}
