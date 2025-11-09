use std::{
    collections::{BTreeMap, BTreeSet},
    num::NonZero,
};

use darling::{FromDeriveInput, FromField, FromMeta, Result, ast, util::Flag};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Generics, Ident, Path, Type, parse_quote};

#[derive(FromDeriveInput, Debug)]
#[darling(
    attributes(squire),
    supports(struct_named, struct_newtype, struct_tuple)
)]
pub struct Parameters {
    ident: Ident,
    generics: Generics,
    data: ast::Data<(), Field>,

    named: Flag,
    sequential: Flag,
}

impl Parameters {
    pub fn derive(self) -> Result<TokenStream> {
        let ident = self.ident;

        let (_, ty_generics, where_clause) = self.generics.split_for_impl();

        // Create impl_generics with our 'statement lifetime
        let impl_generics = if self.generics.params.is_empty() {
            quote! { <'statement> }
        } else {
            let params = &self.generics.params;
            quote! { <'statement, #params> }
        };

        let fields = match self.data {
            ast::Data::Struct(contents) => match contents.style {
                ast::Style::Struct | ast::Style::Tuple => contents.fields,
                ast::Style::Unit => return Err(darling::Error::unsupported_shape("unit struct")),
            },
            ast::Data::Enum(_) => return Err(darling::Error::unsupported_shape("enum")),
        };

        let fields: Vec<_> = fields
            .into_iter()
            .filter(|field| !field.skip.is_present())
            .collect();

        let mut errors = darling::Error::accumulator();

        if self.named.is_present() && self.sequential.is_present() {
            errors.push(
                darling::Error::custom("named and sequential are mutually exclusive")
                    .with_span(&self.sequential.span()),
            );
        }

        let bind_exprs: Vec<_> = fields
            .iter()
            .enumerate()
            .filter_map(|(i, field)| errors.handle_in(|| field.bind_expr(i)))
            .collect();

        let lifetime_bounds: BTreeSet<_> = fields.iter().filter_map(Field::borrow_bound).collect();

        // Build where clause with lifetime bounds
        let mut where_clause = where_clause.cloned();
        if !lifetime_bounds.is_empty() {
            let lifetime_predicates: Vec<syn::WherePredicate> = lifetime_bounds
                .iter()
                .map(|lt| parse_quote!(#lt: 'statement))
                .collect();

            if where_clause.is_none() {
                where_clause = Some(parse_quote!(where));
            }

            if let Some(ref mut where_clause) = where_clause {
                where_clause.predicates.extend(lifetime_predicates);
            }
        }

        let names: BTreeMap<String, usize> = fields
            .iter()
            .filter_map(|field| field.name())
            .enumerate()
            .map(|(i, field)| (field, i))
            .collect();

        if self.named.is_present() && names.len() < fields.len() {
            errors.push(
                darling::Error::custom("not all fields have names").with_span(&self.named.span()),
            );
        }

        errors.finish()?;

        let (indexes_type, resolve_indexes) = if !self.sequential.is_present() && !names.is_empty()
        {
            let count = names.len();

            let initializers = names.keys().enumerate().map(|(i, name)| {
                quote! {
                    if let Some(index) = params.index(#name) {
                        indexes[#i].write(index);
                    } else {
                        return None;
                    }
                }
            });

            let finalize = if cfg!(feature = "lang-array-assume-init") {
                quote! {
                    unsafe {
                        Some(::core::mem::MaybeUninit::array_assume_init(indexes))
                    }
                }
            } else {
                quote! {
                    Some(indexes.map(|i| unsafe { ::core::mem::MaybeUninit::assume_init(i) }))
                }
            };

            let initialize = quote! {
                let params = statement.parameters();
                let mut indexes = [::core::mem::MaybeUninit::<squire::Index>::uninit(); #count];

                #(#initializers)*

                #finalize
            };

            (
                quote! { type Indexes = [squire::Index; #count]; },
                initialize,
            )
        } else {
            (quote! { type Indexes = (); }, quote! { Some(()) })
        };

        let bind_statements = bind_exprs.into_iter().enumerate().map(|(i, expr)| {
            let field = &fields[i];
            let index = match field.parameter(i, self.sequential.is_present()) {
                Parameter::Named(name) => {
                    let offset = names.get(&name).unwrap();
                    quote! { indexes[#offset] }
                }
                Parameter::Sequential(index) => {
                    let index = index.get();
                    quote! { unsafe { squire::Index.new_unchecked(#index) } }
                }
            };

            quote! { binding.set(#index, #expr)?; }
        });

        Ok(quote! {
            impl #impl_generics squire::Parameters<'statement> for #ident #ty_generics
            #where_clause
            {
                #indexes_type

                fn resolve<'connection>(statement: &squire::Statement<'connection>) -> Option<Self::Indexes> {
                    #resolve_indexes
                }

                fn bind<'connection>(self, binding: &mut squire::Binding<'connection, 'statement>, indexes: Self::Indexes) -> squire::Result<()>
                where
                    'connection: 'statement,
                {
                    #(#bind_statements)*

                    Ok(())
                }
            }
        })
    }
}

#[derive(Clone, Debug)]
enum Parameter {
    Named(String),
    Sequential(NonZero<i32>),
}

#[derive(FromField, Debug)]
#[darling(attributes(squire))]
struct Field {
    ident: Option<Ident>,
    ty: Type,

    borrow: Flag,
    index: Option<NonZero<i32>>,
    rename: Option<Ident>,
    skip: Flag,
    result: Flag,
    with: Option<With>,
}

impl Field {
    fn expr(&self, index: usize) -> syn::Expr {
        if let Some(ref ident) = self.ident {
            parse_quote!(self.#ident)
        } else {
            parse_quote!(self.#index)
        }
    }

    fn bind_expr(&self, index: usize) -> Result<syn::Expr> {
        let expr = self.expr(index);

        let expr = if let Some(ref with) = self.with {
            with.wrap(&expr)
        } else {
            expr
        };

        let expr = if self.result.is_present() {
            parse_quote!(#expr?)
        } else {
            expr
        };

        let expr = if self.borrow.is_present() {
            if !matches!(&self.ty, syn::Type::Reference(_)) {
                return Err(self.borrow_error());
            }
            parse_quote!(squire::ffi::Static::new(#expr))
        } else {
            expr
        };

        Ok(expr)
    }

    fn parameter(&self, index: usize, sequential: bool) -> Parameter {
        match (
            sequential,
            self.index,
            self.rename.as_ref(),
            self.ident.as_ref(),
        ) {
            (_, Some(index), _, _) => Parameter::Sequential(index),
            (false, None, Some(name), _) => Parameter::Named(name.to_string()),
            (false, None, None, Some(ident)) => Parameter::Named(ident.to_string()),
            _ => Parameter::Sequential(unsafe { NonZero::new_unchecked((index as i32) + 1) }),
        }
    }

    fn name(&self) -> Option<String> {
        match self.parameter(0, false) {
            Parameter::Named(name) => Some(name),
            Parameter::Sequential(_) => None,
        }
    }

    fn borrow_bound(&self) -> Option<syn::Lifetime> {
        if self.borrow.is_present() {
            if let syn::Type::Reference(syn::TypeReference {
                lifetime: Some(ref lifetime),
                ..
            }) = self.ty
            {
                Some(lifetime.clone())
            } else {
                None
            }
        } else {
            None
        }
    }

    fn borrow_error(&self) -> darling::Error {
        darling::Error::custom("borrow can only be used with references")
            .with_span(&self.borrow.span())
    }
}

#[derive(Debug, Clone)]
pub enum With {
    /// Simple path: `my_function`
    Path(Path),
    /// String reference: `"my_function"`
    Name(String),
    /// Complex expression with placeholders: `some::func(_, some::Enum::Value)`
    Expression(syn::Expr),
}

impl With {
    /// Pipe the given [`Expr`](syn::Expr) through the [`With`] wrapper.
    pub fn wrap(&self, value_expr: &syn::Expr) -> syn::Expr {
        match self {
            With::Path(path) => {
                syn::parse_quote!(#path(#value_expr))
            }
            With::Name(func_name) => {
                let path: syn::Path =
                    syn::parse_str(func_name).unwrap_or_else(|_| syn::parse_quote!(#func_name));
                syn::parse_quote!(#path(#value_expr))
            }
            With::Expression(expr) => Self::replace_placeholders(expr, value_expr),
        }
    }

    fn replace_placeholders(expr: &syn::Expr, replacement: &syn::Expr) -> syn::Expr {
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
            syn::Meta::NameValue(meta) => {
                match &meta.value {
                    // Handle string literals: with = "my_function"
                    syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(lit_str),
                        ..
                    }) => Ok(With::Name(lit_str.value())),

                    // Handle direct paths: with = my_function
                    syn::Expr::Path(path) => Ok(With::Path(path.path.clone())),

                    // Handle complex expressions: with = some::func(_, Enum::Value)
                    expr => Ok(With::Expression(expr.clone())),
                }
            }
            _ => Err(darling::Error::custom("Invalid function attribute format")),
        }
    }
}
