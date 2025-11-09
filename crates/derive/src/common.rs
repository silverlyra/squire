use std::{fmt::Debug, num::NonZero};

use darling::{FromMeta, ast, util::Flag};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Expr, Generics, Ident, Path};

/// Trait for types that can be used as sequential parameter/column indices.
pub trait SequentialIndex: Copy + Debug + FromMeta {
    fn from_field_index(index: usize) -> Self;
}

impl SequentialIndex for NonZero<i32> {
    fn from_field_index(index: usize) -> Self {
        // Parameters are 1-indexed in SQLite
        unsafe { NonZero::new_unchecked((index as i32) + 1) }
    }
}

impl SequentialIndex for i32 {
    fn from_field_index(index: usize) -> Self {
        // Columns are 0-indexed in SQLite
        index as i32
    }
}

/// How a field is identified - either by name or sequential index.
#[derive(Clone, Debug)]
pub enum FieldIdentity<I>
where
    I: SequentialIndex,
{
    Named(String),
    Sequential(I),
}

impl<I: SequentialIndex> FieldIdentity<I> {
    /// Determine field identity from field metadata.
    pub fn from_field(
        ident: &Option<Ident>,
        field_index: usize,
        rename: Option<&Ident>,
        explicit_index: Option<I>,
        sequential: bool,
    ) -> Self {
        match (sequential, explicit_index, rename, ident) {
            // Explicit index always wins
            (_, Some(index), _, _) => FieldIdentity::Sequential(index),
            // Named mode with rename
            (false, None, Some(name), _) => FieldIdentity::Named(name.to_string()),
            // Named mode with field ident
            (false, None, None, Some(ident)) => FieldIdentity::Named(ident.to_string()),
            // Sequential mode - compute from field index
            (true, None, _, _) => FieldIdentity::Sequential(I::from_field_index(field_index)),
            // Named mode but no name available (tuple struct) - fall back to sequential
            (false, None, None, None) => {
                FieldIdentity::Sequential(I::from_field_index(field_index))
            }
        }
    }

    /// Extract the name if this is a named identity.
    pub fn name(&self) -> Option<&str> {
        match self {
            FieldIdentity::Named(name) => Some(name),
            FieldIdentity::Sequential(_) => None,
        }
    }
}

/// Custom function wrapper for `bind_with` and `fetch_with` attributes.
#[derive(Debug, Clone)]
pub enum With {
    /// Simple path: `my_function`
    Path(Path),
    /// String reference: `"my_function"`
    Name(String),
    /// Complex expression with placeholders: `some::func(_, some::Enum::Value)`
    Expression(Expr),
}

impl With {
    /// Wrap the given expression with this custom function.
    ///
    /// For bind_with: wraps the field value expression
    /// For fetch_with: wraps the column fetch expression
    pub fn wrap(&self, value_expr: &Expr) -> Expr {
        match self {
            With::Path(path) => {
                syn::parse_quote!(#path(#value_expr))
            }
            With::Name(func_name) => {
                let path: Path =
                    syn::parse_str(func_name).unwrap_or_else(|_| syn::parse_quote!(#func_name));
                syn::parse_quote!(#path(#value_expr))
            }
            With::Expression(expr) => Self::replace_placeholders(expr, value_expr),
        }
    }

    fn replace_placeholders(expr: &Expr, replacement: &Expr) -> Expr {
        use syn::*;
        match expr {
            // Replace underscore with the actual value
            Expr::Path(path) if path.path.is_ident("_") => replacement.clone(),

            // Recursively process function calls
            Expr::Call(call) => {
                let func = Self::replace_placeholders(&call.func, replacement);
                let args = call
                    .args
                    .iter()
                    .map(|arg| Self::replace_placeholders(arg, replacement))
                    .collect();
                Expr::Call(ExprCall {
                    attrs: call.attrs.clone(),
                    func: Box::new(func),
                    paren_token: call.paren_token,
                    args,
                })
            }

            // Recursively process method calls
            Expr::MethodCall(method) => {
                let receiver = Self::replace_placeholders(&method.receiver, replacement);
                let args = method
                    .args
                    .iter()
                    .map(|arg| Self::replace_placeholders(arg, replacement))
                    .collect();
                Expr::MethodCall(ExprMethodCall {
                    attrs: method.attrs.clone(),
                    receiver: Box::new(receiver),
                    dot_token: method.dot_token,
                    method: method.method.clone(),
                    turbofish: method.turbofish.clone(),
                    paren_token: method.paren_token,
                    args,
                })
            }

            // For other expressions, return as-is
            _ => expr.clone(),
        }
    }
}

impl FromMeta for With {
    fn from_meta(item: &syn::Meta) -> darling::Result<Self> {
        match item {
            syn::Meta::NameValue(meta) => match &meta.value {
                // Handle string literals: bind_with = "my_function"
                syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(lit_str),
                    ..
                }) => Ok(With::Name(lit_str.value())),

                // Handle direct paths: bind_with = my_function
                syn::Expr::Path(path) => Ok(With::Path(path.path.clone())),

                // Handle complex expressions: bind_with = some::func(_, Enum::Value)
                expr => Ok(With::Expression(expr.clone())),
            },
            _ => Err(darling::Error::custom("Invalid function attribute format")),
        }
    }
}

/// Generate impl generics with a lifetime parameter.
///
/// For types without generics: `<'lifetime>`
/// For types with generics: `<'lifetime, existing_params>`
///
/// Note: Uses verbose lifetime names (e.g., `'__squire_statement`) to avoid conflicts with
/// user-defined lifetimes, since proc macros don't have hygiene for lifetimes.
pub fn impl_generics_with_lifetime(generics: &Generics, lifetime_name: &str) -> TokenStream {
    let lifetime = syn::Lifetime::new(lifetime_name, Span::call_site());

    if generics.params.is_empty() {
        quote! { <#lifetime> }
    } else {
        let params = &generics.params;
        quote! { <#lifetime, #params> }
    }
}

/// Generate [`MaybeUninit`](core::mem::MaybeUninit) array finalization code.
pub fn assume_array_init(elem_type: TokenStream) -> TokenStream {
    if cfg!(feature = "lang-array-assume-init") {
        quote! {
            unsafe {
                Some(::core::mem::MaybeUninit::array_assume_init(indexes))
            }
        }
    } else {
        quote! {
            Some(indexes.map(|i| unsafe { ::core::mem::MaybeUninit::<#elem_type>::assume_init(i) }))
        }
    }
}

/// Binding mode for parameters/columns - determines whether to use named or sequential indexing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingMode {
    /// Use named bind parameters / match columns by name
    Named,
    /// Use sequential bind parameters / match columns by index
    Sequential,
}

impl BindingMode {
    /// Determine binding mode from flags and struct style.
    pub fn from_flags_and_style(
        named: &Flag,
        sequential: &Flag,
        style: ast::Style,
    ) -> darling::Result<Self> {
        if named.is_present() && sequential.is_present() {
            return Err(
                darling::Error::custom("named and sequential are mutually exclusive")
                    .with_span(&sequential.span()),
            );
        }

        if sequential.is_present() {
            Ok(BindingMode::Sequential)
        } else if named.is_present() || style == ast::Style::Struct {
            // Explicit #[squire(named)] or struct with named fields
            Ok(BindingMode::Named)
        } else {
            // Tuple structs default to sequential
            Ok(BindingMode::Sequential)
        }
    }

    /// Check if this mode uses named binding.
    pub fn is_named(self) -> bool {
        matches!(self, BindingMode::Named)
    }
}

/// Process fields through a builder function, collecting errors.
///
/// This is a common pattern in both derives: iterate over fields, call a build function,
/// and accumulate any errors that occur.
pub fn process_fields<F, T>(
    fields: &[F],
    mut builder: impl FnMut(usize, &F) -> darling::Result<T>,
) -> darling::Result<Vec<T>> {
    let mut errors = darling::Error::accumulator();
    let results: Vec<_> = fields
        .iter()
        .enumerate()
        .filter_map(|(i, field)| errors.handle_in(|| builder(i, field)))
        .collect();
    errors.finish()?;
    Ok(results)
}
