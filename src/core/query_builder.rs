//! Base query builder implementation with common functionality.
//!
//! This module provides a foundation for different types of query builders,
//! implementing common patterns and reducing code duplication.

use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::DeriveInput;

use super::{FieldInfo, FieldCategory, BuilderMethodGenerator};

/// Base query builder with common functionality
pub struct BaseQueryBuilder {
    pub builder_name: String,
    pub struct_name: String,
    pub table_name: String,
}

impl BaseQueryBuilder {
    pub fn new(input: &DeriveInput) -> Self {
        let struct_name = input.ident.to_string();
        let table_name = crate::common::utils::get_table_name(input);
        
        Self {
            builder_name: format!("{}QueryBuilder", struct_name),
            struct_name,
            table_name,
        }
    }

    /// Generate basic setter methods for fields
    pub fn generate_setter_methods(&self, fields: &[FieldInfo]) -> TokenStream {
        let methods = fields.iter().map(|field| {
            let field_name = &field.name;
            let method_name = quote::format_ident!("{}_eq", field_name);
            let field_type = field.sql_type();

            quote! {
                pub fn #method_name(mut self, value: #field_type) -> Self {
                    self.#field_name = Some(value);
                    self
                }
            }
        });

        quote! {
            #(#methods)*
        }
    }

    /// Generate struct field declarations
    pub fn generate_struct_fields(&self, fields: &[FieldInfo]) -> TokenStream {
        let fields = fields.iter().map(|field| {
            let field_name = &field.name;
            let field_type = field.sql_type();
            quote! {
                #field_name: Option<#field_type>,
            }
        });

        quote! {
            #(#fields)*
        }
    }

    /// Generate struct initialization
    pub fn generate_struct_init(&self, fields: &[FieldInfo]) -> TokenStream {
        let inits = fields.iter().map(|field| {
            let field_name = &field.name;
            quote! {
                #field_name: None,
            }
        });

        quote! {
            Self {
                #(#inits)*
            }
        }
    }
}
