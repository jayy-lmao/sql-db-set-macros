//! Demonstration of the advanced query builder system.
//!
//! This module shows how the new trait-based, configurable system
//! can be used to create more maintainable and extensible query builders for any ORM use case.
//!
//! # Note
//! This crate is **not** intended for domain-specific or business-case queries. Instead, it is a general-purpose ORM query builder framework, inspired by production-grade ORMs (e.g., Java, .NET, etc.).
//!
//! ## Extending the ORM
//! To add new query types, extend the system with additional SQL/ORM features (e.g., bulk operations, upserts, soft deletes, advanced filtering, pagination, etc.).
//!
//! See the tests folder for realistic consumer usage examples.

use proc_macro2::TokenStream;
use syn::DeriveInput;
use quote::quote;

use crate::common::{
    config::{DbSetConfigBuilder, extract_config_from_attributes},
    builder_factory::QueryBuilderFactory,
    query_builder_gen::QueryType,
    field_analysis::analyze_struct_fields,
};

/// Demonstrates the usage of the new advanced system
pub struct AdvancedQueryBuilderDemo;

impl AdvancedQueryBuilderDemo {
    /// Generate query builders using the new system with custom configuration
    pub fn generate_with_config(input: &DeriveInput) -> TokenStream {
        // Extract configuration from attributes
        let base_config = extract_config_from_attributes(input);
        
        // Enhance with additional settings
        let _enhanced_config = DbSetConfigBuilder::new()
            .table_name(base_config.table_name.unwrap_or_else(|| input.ident.to_string().to_lowercase()))
            .include_validation(true)
            .async_methods(true)
            .build();

        // Analyze the struct capabilities
        let analysis_report = QueryBuilderFactory::analyze_query_capabilities(input);
        
        // Generate comment with analysis
        let analysis_comment = format!("/*\n{}\n*/", analysis_report);
        
        // Generate the query builders
        let many_builder = QueryBuilderFactory::generate_builder(QueryType::Many, input);
        
        quote! {
            #[doc = #analysis_comment]
            #many_builder
        }
    }

    /// Example of how to extend the system with new ORM/SQL query types
    pub fn demonstrate_extensibility() -> &'static str {
        r#"
        Example: Extending the ORM with new query types (not business logic)

        1. Add a new QueryType variant for an ORM/SQL feature:
           enum QueryType {
               Many, One, Insert, Update, Delete,
               BulkInsert,      // e.g., for batch inserts
               Upsert,          // e.g., insert or update
               SoftDelete,      // e.g., mark as deleted
               Paginated,       // e.g., for pagination support
           }

        2. Implement QueryBuilderGenerator for the new type:
           struct BulkInsertQueryBuilder;
           impl QueryBuilderGenerator for BulkInsertQueryBuilder {
               fn query_type(&self) -> QueryType { QueryType::BulkInsert }
               // ... implement other methods for struct, SQL, params, etc.
           }

        3. Register in the factory:
           match query_type {
               QueryType::BulkInsert => BulkInsertQueryBuilder.generate(input),
               // ... other cases
           }

        4. Configure field behavior as needed:
           let config = DbSetConfigBuilder::new()
               .field_override("created_at", FieldOverride {
                   exclude_from: vec![QueryType::BulkInsert],
                   // ... other settings
               })
               .build();
        "#
    }
}

/// Example: Specialized builder for a domain (for illustration only)
///
/// This is **not** the recommended pattern for most users. The ORM is designed for general-purpose use.
pub struct ECommerceQueryBuilders;

impl ECommerceQueryBuilders {
    /// Generate specialized query builders for e-commerce domain
    /// (For illustration only; not typical usage)
    pub fn generate_product_builders(input: &DeriveInput) -> TokenStream {
        let fields = analyze_struct_fields(input);
        
        // Look for e-commerce specific fields
        let has_price = fields.iter().any(|f| f.name.to_string().contains("price"));
        let has_inventory = fields.iter().any(|f| f.name.to_string().contains("inventory") || f.name.to_string().contains("stock"));
        let has_category = fields.iter().any(|f| f.name.to_string().contains("category"));
        
        let specialized_methods = if has_price && has_inventory && has_category {
            quote! {
                impl ProductSpecializedQueries {
                    /// Find products by price range
                    pub async fn find_by_price_range<'e, E: sqlx::PgExecutor<'e>>(
                        executor: E,
                        min_price: Option<f64>,
                        max_price: Option<f64>,
                    ) -> Result<Vec<super::Product>, sqlx::Error> {
                        // Implementation would go here
                        todo!("Implement price range query")
                    }

                    /// Find low stock products
                    pub async fn find_low_stock<'e, E: sqlx::PgExecutor<'e>>(
                        executor: E,
                        threshold: i32,
                    ) -> Result<Vec<super::Product>, sqlx::Error> {
                        // Implementation would go here
                        todo!("Implement low stock query")
                    }

                    /// Find products by category
                    pub async fn find_by_category<'e, E: sqlx::PgExecutor<'e>>(
                        executor: E,
                        category: String,
                    ) -> Result<Vec<super::Product>, sqlx::Error> {
                        // Implementation would go here
                        todo!("Implement category query")
                    }
                }

                pub struct ProductSpecializedQueries;
            }
        } else {
            quote! {
                // No specialized e-commerce methods available for this struct
            }
        };

        let standard_builders = QueryBuilderFactory::generate_all_builders(input);

        quote! {
            #standard_builders
            #specialized_methods
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_advanced_generation() {
        let input: DeriveInput = parse_quote! {
            #[dbset(table_name = "users", validation = true)]
            pub struct User {
                #[key]
                id: String,
                name: String,
                #[unique]
                email: String,
                #[custom_enum]
                status: UserStatus,
            }
        };

        let result = AdvancedQueryBuilderDemo::generate_with_config(&input);
        let result_str = result.to_string();
        
        // Should contain analysis comment
        assert!(result_str.contains("Query capabilities"));
        
        // Should contain query builder
        assert!(result_str.contains("QueryBuilder"));
    }

    #[test]
    fn test_ecommerce_specialization() {
        let product_input: DeriveInput = parse_quote! {
            pub struct Product {
                id: String,
                name: String,
                price: f64,
                inventory_count: i32,
                category_id: String,
            }
        };

        let result = ECommerceQueryBuilders::generate_product_builders(&product_input);
        let result_str = result.to_string();
        
        // Should contain specialized methods
        assert!(result_str.contains("find_by_price_range"));
        assert!(result_str.contains("find_low_stock"));
        assert!(result_str.contains("find_by_category"));
    }

    #[test]
    fn test_extensibility_documentation() {
        let docs = AdvancedQueryBuilderDemo::demonstrate_extensibility();
        assert!(docs.contains("BulkInsert"));
        assert!(docs.contains("Upsert"));
        assert!(docs.contains("SoftDelete"));
        assert!(docs.contains("Paginated"));
    }
}
