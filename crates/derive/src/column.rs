use std::collections::{BTreeMap, BTreeSet};

use darling::{FromDeriveInput, FromField, Result, ast, util::Flag};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Expr, Generics, Ident, Type, parse_quote};

use crate::common::{
    BindingMode, FieldIdentity, With, assume_array_init, impl_generics_with_lifetime,
    process_fields,
};

#[derive(FromDeriveInput, Debug)]
#[darling(
    attributes(squire),
    supports(struct_named, struct_newtype, struct_tuple)
)]
pub struct ColumnsDerive {
    ident: Ident,
    generics: Generics,
    data: ast::Data<(), FieldDerive>,

    named: Flag,
    sequential: Flag,
}

impl ColumnsDerive {
    pub fn derive(self) -> Result<TokenStream> {
        // Step 1: Extract and validate fields
        let (fields, style) = self.extract_fields()?;

        // Step 2: Determine binding mode from flags and struct style
        let binding_mode = BindingMode::from_flags_and_style(&self.named, &self.sequential, style)?;

        // Step 3: Build metadata for each field
        let field_metas = process_fields(&fields, |i, field| field.build_meta(i, binding_mode))?;

        // Step 4: Build the trait implementation
        let meta = ColumnsMeta {
            ident: self.ident,
            generics: self.generics,
            fields: field_metas,
            binding_mode,
            style,
        };

        meta.generate_impl()
    }

    fn extract_fields(&self) -> Result<(Vec<&FieldDerive>, ast::Style)> {
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

    bytes: Flag,
    index: Option<i32>,
    rename: Option<Ident>,
    skip: Flag,
    result: Flag,
    fetch_with: Option<With>,
}

impl FieldDerive {
    fn build_meta(&self, field_index: usize, binding_mode: BindingMode) -> Result<Column> {
        // Determine the column identity
        let sequential = binding_mode == BindingMode::Sequential;
        let identity = FieldIdentity::from_field(
            &self.ident,
            field_index,
            self.rename.as_ref(),
            self.index,
            sequential,
        );

        // Build the fetch expression
        let fetch_expr = self.build_fetch_expr(field_index)?;

        // Extract lifetime bound if using bytes wrapper
        let bytes_bound = self.bytes_bound();

        Ok(Column {
            ident: self.ident.clone(),
            ty: self.ty.clone(),
            field_index,
            identity,
            fetch_expr,
            bytes_bound,
        })
    }

    fn build_fetch_expr(&self, _field_index: usize) -> Result<Expr> {
        // Start with base column fetch - Fetch::fetch returns Result, so unwrap it
        let ty = &self.ty;
        let column_var: Ident = parse_quote!(column);
        let mut expr: Expr =
            parse_quote!(<#ty as squire::Fetch<'row>>::fetch(statement, #column_var)?);

        // Apply custom fetch_with function if provided
        if let Some(ref with) = self.fetch_with {
            expr = with.wrap(&expr);
        }

        // Unwrap Result if result flag is set (for custom functions that return Result)
        if self.result.is_present() {
            expr = parse_quote!(#expr?);
        }

        // Wrap in Bytes if bytes flag is set
        if self.bytes.is_present() {
            if !matches!(&self.ty, Type::Reference(_)) {
                return Err(self.bytes_error());
            }
            // Note: The Bytes wrapper is applied at the type level via the Fetch trait,
            // so we don't need to wrap the expression here
        }

        Ok(expr)
    }

    fn bytes_bound(&self) -> Option<syn::Lifetime> {
        if self.bytes.is_present() {
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

    fn bytes_error(&self) -> darling::Error {
        darling::Error::custom("bytes can only be used with references")
            .with_span(&self.bytes.span())
    }
}

/// Validated and processed metadata for the Columns derive.
struct ColumnsMeta {
    ident: Ident,
    generics: Generics,
    fields: Vec<Column>,
    binding_mode: BindingMode,
    style: ast::Style,
}

impl ColumnsMeta {
    fn generate_impl(self) -> Result<TokenStream> {
        let ident = &self.ident;
        let (_, ty_generics, where_clause) = self.generics.split_for_impl();

        // Create impl_generics with row lifetime
        let impl_generics = impl_generics_with_lifetime(&self.generics, "'row");

        // Collect lifetime bounds from bytes-wrapped fields
        let lifetime_bounds: BTreeSet<_> = self
            .fields
            .iter()
            .filter_map(|f| f.bytes_bound.clone())
            .collect();

        // Build where clause with lifetime bounds
        let mut where_clause = where_clause.cloned();
        if !lifetime_bounds.is_empty() {
            let lifetime_predicates: Vec<syn::WherePredicate> = lifetime_bounds
                .iter()
                .map(|lt| parse_quote!(#lt: 'row))
                .collect();

            if where_clause.is_none() {
                where_clause = Some(parse_quote!(where));
            }

            if let Some(ref mut where_clause) = where_clause {
                where_clause.predicates.extend(lifetime_predicates);
            }
        }

        // Build column name map for named resolution
        let column_names: BTreeMap<&str, usize> = self
            .fields
            .iter()
            .enumerate()
            .filter_map(|(i, field)| field.identity.name().map(|name| (name, i)))
            .collect();

        // Validate that explicit #[squire(named)] on tuple structs has all names
        if self.binding_mode.requires_all_names(self.style)
            && column_names.len() < self.fields.len()
        {
            return Err(darling::Error::custom("not all fields have names"));
        }

        let (indexes_type, resolve_indexes) =
            if self.binding_mode.is_named() && !column_names.is_empty() {
                self.generate_named_resolution(&column_names)
            } else {
                (quote! { type Indexes = (); }, quote! { Some(()) })
            };

        let fetch_statements = self.generate_fetch_statements(&column_names);

        Ok(quote! {
            impl #impl_generics squire::Columns<'row> for #ident #ty_generics
            #where_clause
            {
                #indexes_type

                fn resolve<'connection>(statement: &squire::Statement<'connection>) -> Option<Self::Indexes> {
                    #resolve_indexes
                }

                fn fetch<'connection>(statement: &'row squire::Statement<'connection>, indexes: Self::Indexes) -> squire::Result<Self>
                where
                    'connection: 'row,
                {
                    #fetch_statements
                }
            }
        })
    }

    fn generate_named_resolution(
        &self,
        column_names: &BTreeMap<&str, usize>,
    ) -> (TokenStream, TokenStream) {
        let count = column_names.len();

        let initializers = column_names.keys().enumerate().map(|(i, name)| {
            quote! {
                if let Some(column) = columns.index(#name) {
                    indexes[#i].write(column);
                } else {
                    return None;
                }
            }
        });

        let finalize = assume_array_init(quote!(squire::Column));

        let initialize = quote! {
            let columns = statement.columns();
            let mut indexes = [::core::mem::MaybeUninit::<squire::Column>::uninit(); #count];

            #(#initializers)*

            #finalize
        };

        (
            quote! { type Indexes = [squire::Column; #count]; },
            initialize,
        )
    }

    fn generate_fetch_statements(&self, column_names: &BTreeMap<&str, usize>) -> TokenStream {
        let field_bindings: Vec<_> = self
            .fields
            .iter()
            .enumerate()
            .map(|(i, field)| {
                // For named fields, use the field name; for tuple fields, use field_N
                let var_name = field
                    .ident
                    .as_ref()
                    .map(|id| quote!(#id))
                    .unwrap_or_else(|| {
                        let var_ident =
                            Ident::new(&format!("field_{}", i), proc_macro2::Span::call_site());
                        quote!(#var_ident)
                    });

                let column_expr = match &field.identity {
                    FieldIdentity::Named(name) => {
                        let offset = column_names.get(name.as_str()).unwrap();
                        quote! { indexes[#offset] }
                    }
                    FieldIdentity::Sequential(index) => {
                        quote! { squire::Column::from(#index) }
                    }
                };

                let fetch_expr = &field.fetch_expr;

                quote! {
                    let #var_name = {
                        let column = #column_expr;
                        #fetch_expr
                    };
                }
            })
            .collect();

        let field_names: Vec<_> = self
            .fields
            .iter()
            .enumerate()
            .map(|(i, field)| {
                field
                    .ident
                    .as_ref()
                    .map(|id| quote!(#id))
                    .unwrap_or_else(|| {
                        let var_ident =
                            Ident::new(&format!("field_{}", i), proc_macro2::Span::call_site());
                        quote!(#var_ident)
                    })
            })
            .collect();

        if self.fields.iter().any(|f| f.ident.is_some()) {
            // Named struct
            quote! {
                #(#field_bindings)*
                Ok(Self { #(#field_names),* })
            }
        } else {
            // Tuple struct
            quote! {
                #(#field_bindings)*
                Ok(Self(#(#field_names),*))
            }
        }
    }
}

/// Processed metadata for a single [field](FieldDerive) from [`ColumnsDerive`].
struct Column {
    ident: Option<Ident>,
    ty: Type,
    field_index: usize,
    identity: FieldIdentity<i32>,
    fetch_expr: Expr,
    bytes_bound: Option<syn::Lifetime>,
}
