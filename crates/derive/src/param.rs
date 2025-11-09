use std::{
    collections::{BTreeMap, BTreeSet},
    num::NonZero,
};

use darling::{FromDeriveInput, FromField, Result, ast, util::Flag};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Expr, Generics, Ident, Type, parse_quote};

use crate::common::{
    BindingMode, FieldIdentity, NamedIndexResolution, With, impl_generics_with_lifetime,
    process_fields,
};

#[derive(FromDeriveInput, Debug)]
#[darling(
    attributes(squire),
    supports(struct_named, struct_newtype, struct_tuple)
)]
pub struct ParametersDerive {
    ident: Ident,
    generics: Generics,
    data: ast::Data<(), FieldDerive>,

    named: Flag,
    sequential: Flag,
}

impl ParametersDerive {
    pub fn derive(self) -> Result<TokenStream> {
        // Step 1: Extract and validate fields
        let (fields, style) = self.fields()?;

        // Step 2: Determine binding mode from flags and struct style
        let binding_mode = BindingMode::from_flags_and_style(&self.named, &self.sequential, style)?;

        // Step 3: Build metadata for each field
        let field_metas = process_fields(&fields, |i, field| field.build_meta(i, binding_mode))?;

        // Step 4: Build the trait implementation
        let meta = Parameters {
            ident: self.ident,
            generics: self.generics,
            fields: field_metas,
            binding_mode,
        };

        meta.generate_impl()
    }

    fn fields(&self) -> Result<(Vec<&FieldDerive>, ast::Style)> {
        match &self.data {
            ast::Data::Struct(contents) => match contents.style {
                ast::Style::Struct | ast::Style::Tuple => {
                    let fields = contents
                        .fields
                        .iter()
                        .filter(|field| !field.skip.is_present())
                        .collect();
                    Ok((fields, contents.style))
                }
                ast::Style::Unit => Err(darling::Error::unsupported_shape("unit struct")),
            },
            ast::Data::Enum(_) => Err(darling::Error::unsupported_shape("enum")),
        }
    }
}

#[derive(FromField, Debug)]
#[darling(attributes(squire))]
struct FieldDerive {
    ident: Option<Ident>,
    ty: Type,

    borrow: Flag,
    index: Option<NonZero<i32>>,
    rename: Option<Ident>,
    skip: Flag,
    result: Flag,
    bind_with: Option<With>,
}

impl FieldDerive {
    fn build_meta(&self, field_index: usize, binding_mode: BindingMode) -> Result<Parameter> {
        // Determine the parameter identity
        let sequential = binding_mode == BindingMode::Sequential;
        let identity = FieldIdentity::from_field(
            &self.ident,
            field_index,
            self.rename.as_ref(),
            self.index,
            sequential,
        );

        // Build the bind expression
        let bind_expr = self.build_bind_expr(field_index)?;

        // Extract lifetime bound if using borrow wrapper
        let borrow_bound = self.borrow_bound();

        Ok(Parameter {
            identity,
            bind_expr,
            borrow_bound,
        })
    }

    fn build_bind_expr(&self, field_index: usize) -> Result<Expr> {
        // Start with field access expression
        let mut expr = if let Some(ref ident) = self.ident {
            parse_quote!(self.#ident)
        } else {
            let index = syn::Index::from(field_index);
            parse_quote!(self.#index)
        };

        // Apply custom bind_with function if provided (or legacy with)
        if let Some(ref with) = self.bind_with {
            expr = with.wrap(&expr);
        }

        // Unwrap Result if result flag is set
        if self.result.is_present() {
            expr = parse_quote!(#expr?);
        }

        // Wrap in Borrowed if borrow flag is set
        if self.borrow.is_present() {
            if !matches!(&self.ty, Type::Reference(_)) {
                return Err(self.borrow_error());
            }
            expr = parse_quote!(squire::Borrowed::new(#expr));
        }

        Ok(expr)
    }

    fn borrow_bound(&self) -> Option<syn::Lifetime> {
        if self.borrow.is_present() {
            if let Type::Reference(syn::TypeReference {
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

/// [`ParametersDerive`] data that has been prepared to generate the `impl` tokens.
struct Parameters {
    ident: Ident,
    generics: Generics,
    fields: Vec<Parameter>,
    binding_mode: BindingMode,
}

impl Parameters {
    fn generate_impl(self) -> Result<TokenStream> {
        let ident = &self.ident;
        let (_, ty_generics, where_clause) = self.generics.split_for_impl();

        // Create impl_generics with our 'statement lifetime
        let impl_generics = impl_generics_with_lifetime(&self.generics, "'statement");

        // Collect lifetime bounds from borrow-wrapped fields
        let lifetime_bounds: BTreeSet<_> = self
            .fields
            .iter()
            .filter_map(|f| f.borrow_bound.clone())
            .collect();

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

        // Build parameter name map for named resolution
        let param_names: BTreeMap<&str, usize> = self
            .fields
            .iter()
            .enumerate()
            .filter_map(|(i, field)| field.identity.name().map(|name| (name, i)))
            .collect();

        if self.binding_mode.is_named() && param_names.len() < self.fields.len() {
            return Err(darling::Error::custom("not all fields have names"));
        }

        let NamedIndexResolution { indexes, resolve } =
            if self.binding_mode.is_named() && !param_names.is_empty() {
                NamedIndexResolution::derive(
                    &param_names,
                    quote!(parameters),
                    quote!(squire::BindIndex),
                )
            } else {
                NamedIndexResolution::empty()
            };

        let bind_statements = self.generate_bind_statements(&param_names);

        Ok(quote! {
            impl #impl_generics squire::Parameters<'statement> for #ident #ty_generics
            #where_clause
            {
                #indexes

                fn resolve<'connection>(statement: &squire::Statement<'connection>) -> Option<Self::Indexes> {
                    #resolve
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

    fn generate_bind_statements(&self, param_names: &BTreeMap<&str, usize>) -> Vec<TokenStream> {
        self.fields
            .iter()
            .map(|field| {
                let index_expr = match &field.identity {
                    FieldIdentity::Named(name) => {
                        let offset = param_names.get(name.as_str()).unwrap();
                        quote! { indexes[#offset] }
                    }
                    FieldIdentity::Sequential(index) => {
                        let index_val = index.get();
                        quote! { unsafe { squire::BindIndex::new_unchecked(#index_val) } }
                    }
                };

                let bind_expr = &field.bind_expr;

                quote! {
                    binding.set(#index_expr, #bind_expr)?;
                }
            })
            .collect()
    }
}

/// Processed metadata for a single field in the Parameters derive.
struct Parameter {
    identity: FieldIdentity<NonZero<i32>>,
    bind_expr: Expr,
    borrow_bound: Option<syn::Lifetime>,
}
