//! Query builder generation traits and utilities.
//!
//! This module provides a more structured approach to generating different
//! types of query builders with shared patterns and reduced code duplication.

use proc_macro2::TokenStream;
use syn::DeriveInput;
use quote::quote;

use crate::common::field_analysis::{FieldAnalysis, analyze_struct_fields};

/// Trait for generating query builder structs and implementations
pub trait QueryBuilderGenerator {
    /// Get the name of the query builder struct
    fn builder_name(&self, input: &DeriveInput) -> proc_macro2::Ident;
    
    /// Filter fields that should be included in this query builder
    fn filter_fields<'a>(&self, fields: &'a [FieldAnalysis]) -> Vec<&'a FieldAnalysis>;
    
    /// Generate the query builder struct definition
    fn generate_struct(&self, input: &DeriveInput, fields: &[&FieldAnalysis]) -> TokenStream;
    
    /// Generate the query builder methods
    fn generate_methods(&self, input: &DeriveInput, fields: &[&FieldAnalysis]) -> TokenStream;
    
    /// Generate the complete query builder implementation
    fn generate(&self, input: &DeriveInput) -> TokenStream {
        let all_fields = analyze_struct_fields(input);
        let filtered_fields = self.filter_fields(&all_fields);
        let struct_def = self.generate_struct(input, &filtered_fields);
        let methods = self.generate_methods(input, &filtered_fields);
        
        quote! {
            #struct_def
            #methods
        }
    }
}

/// Generate basic setter methods for query builder fields
pub fn generate_setter_methods(fields: &[&FieldAnalysis]) -> TokenStream {
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

/// Generate struct field declarations for query builders
pub fn generate_struct_fields(fields: &[&FieldAnalysis]) -> TokenStream {
    let field_decls = fields.iter().map(|field| {
        let field_name = &field.name;
        let field_type = field.sql_type();
        quote! {
            #field_name: Option<#field_type>,
        }
    });

    quote! {
        #(#field_decls)*
    }
}

/// Generate struct initialization for query builders
pub fn generate_struct_init(fields: &[&FieldAnalysis]) -> TokenStream {
    let field_inits = fields.iter().map(|field| {
        let field_name = &field.name;
        quote! {
            #field_name: None,
        }
    });

    quote! {
        Self {
            #(#field_inits)*
        }
    }
}
