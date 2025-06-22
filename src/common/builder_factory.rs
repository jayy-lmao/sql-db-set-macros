//! Builder factory for generating different types of query builders.
//!
//! This module demonstrates how the trait-based system can be used to
//! create a factory pattern for generating query builders.

use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

use crate::common::field_analysis::analyze_struct_fields;
use crate::common::modern_builders::ModernManyQueryBuilder;
use crate::common::query_builder_gen::{QueryBuilderGenerator, QueryType};

/// Factory for creating different types of query builders
pub struct QueryBuilderFactory;

impl QueryBuilderFactory {
    /// Generate a specific type of query builder
    pub fn generate_builder(query_type: QueryType, input: &DeriveInput) -> TokenStream {
        match query_type {
            QueryType::Many => {
                let generator = ModernManyQueryBuilder;
                generator.generate(input).unwrap_or_else(|e| {
                    let msg = format!("QueryBuilder error: {:?}", e);
                    quote! { compile_error!(#msg); }
                })
            }
            _ => {
                // For now, fall back to empty implementation for other types
                // In a complete implementation, we'd have generators for all types
                quote! {
                    // TODO: Implement other query builder types
                }
            }
        }
    }

    /// Generate all query builders for a struct
    pub fn generate_all_builders(input: &DeriveInput) -> TokenStream {
        let many_builder = Self::generate_builder(QueryType::Many, input);

        // In a complete implementation, we'd generate all builder types
        quote! {
            #many_builder
            // TODO: Generate other builder types (One, Insert, Update, Delete)
        }
    }

    /// Analyze and report on the fields available for query building
    pub fn analyze_query_capabilities(input: &DeriveInput) -> String {
        let fields = analyze_struct_fields(input);
        let mut report = String::new();

        report.push_str(&format!("Query capabilities for {}:\n", input.ident));
        report.push_str(&format!("  Total fields: {}\n", fields.len()));

        let key_fields: Vec<_> = fields.iter().filter(|f| f.is_key).collect();
        let unique_fields: Vec<_> = fields.iter().filter(|f| f.is_unique).collect();
        let auto_fields: Vec<_> = fields.iter().filter(|f| f.is_auto).collect();
        let enum_fields: Vec<_> = fields.iter().filter(|f| f.is_custom_enum).collect();

        report.push_str(&format!(
            "  Key fields: {} ({:?})\n",
            key_fields.len(),
            key_fields
                .iter()
                .map(|f| f.name.to_string())
                .collect::<Vec<_>>()
        ));
        report.push_str(&format!(
            "  Unique fields: {} ({:?})\n",
            unique_fields.len(),
            unique_fields
                .iter()
                .map(|f| f.name.to_string())
                .collect::<Vec<_>>()
        ));
        report.push_str(&format!(
            "  Auto fields: {} ({:?})\n",
            auto_fields.len(),
            auto_fields
                .iter()
                .map(|f| f.name.to_string())
                .collect::<Vec<_>>()
        ));
        report.push_str(&format!(
            "  Enum fields: {} ({:?})\n",
            enum_fields.len(),
            enum_fields
                .iter()
                .map(|f| f.name.to_string())
                .collect::<Vec<_>>()
        ));

        report
    }
}

/// Trait for validating query builder configurations
pub trait QueryBuilderValidator {
    /// Validate that the configuration is suitable for the query type
    fn validate(&self, input: &DeriveInput) -> Result<(), String>;
}

/// Validator for many query builders
pub struct ManyQueryValidator;

impl QueryBuilderValidator for ManyQueryValidator {
    fn validate(&self, input: &DeriveInput) -> Result<(), String> {
        let fields = analyze_struct_fields(input);
        let non_key_unique_fields: Vec<_> = fields
            .iter()
            .filter(|f| !f.is_key && !f.is_unique)
            .collect();

        if non_key_unique_fields.is_empty() {
            Err(format!(
                "Struct {} has no non-key, non-unique fields available for many queries",
                input.ident
            ))
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_query_capability_analysis() {
        let input: DeriveInput = parse_quote! {
            #[dbset(table_name = "users")]
            pub struct User {
                #[key]
                id: String,
                name: String,
                #[unique]
                email: String,
                #[auto]
                created_at: chrono::DateTime<chrono::Utc>,
                #[custom_enum]
                status: UserStatus,
            }
        };

        let report = QueryBuilderFactory::analyze_query_capabilities(&input);
        assert!(report.contains("Query capabilities for User"));
        assert!(report.contains("Total fields: 5"));
        println!("{}", report);
    }

    #[test]
    fn test_many_query_validation() {
        let validator = ManyQueryValidator;

        let valid_input: DeriveInput = parse_quote! {
            pub struct User {
                #[key]
                id: String,
                name: String,
            }
        };

        assert!(validator.validate(&valid_input).is_ok());

        let invalid_input: DeriveInput = parse_quote! {
            pub struct User {
                #[key]
                id: String,
                #[unique]
                email: String,
            }
        };

        assert!(validator.validate(&invalid_input).is_err());
    }
}
