//! Core traits for extensible query generation.
//!
//! This module defines the key traits that enable different query builders
//! to be implemented in a consistent and extensible manner.

use proc_macro2::TokenStream;
use syn::DeriveInput;

use super::FieldInfo;

/// Trait for generating query builder implementations
pub trait QueryBuilderGenerator {
    /// Generate the complete query builder implementation
    fn generate(&self, input: &DeriveInput, fields: &[FieldInfo]) -> TokenStream;
    
    /// Get the name of the query builder struct
    fn builder_name(&self, input: &DeriveInput) -> proc_macro2::Ident;
}

/// Trait for SQL query generation
pub trait SqlQueryGenerator {
    /// Generate the SQL query string
    fn generate_sql(&self, table_name: &str, fields: &[FieldInfo]) -> String;
    
    /// Generate query parameters
    fn generate_parameters(&self, fields: &[FieldInfo]) -> TokenStream;
}

/// Trait for filtering fields based on query type requirements
pub trait FieldFilter {
    /// Filter fields that should be included in the query builder
    fn filter_fields<'a>(&self, fields: &'a [FieldInfo]) -> Vec<&'a FieldInfo>;
    
    /// Get required fields that must be set for the query to be valid
    fn required_fields<'a>(&self, fields: &'a [FieldInfo]) -> Vec<&'a FieldInfo>;
}

/// Trait for generating builder method implementations
pub trait BuilderMethodGenerator {
    /// Generate setter methods for the query builder
    fn generate_methods(&self, fields: &[FieldInfo]) -> TokenStream;
    
    /// Generate the execute method (fetch_one, fetch_all, etc.)
    fn generate_execute_method(&self, fields: &[FieldInfo], table_name: &str) -> TokenStream;
}
