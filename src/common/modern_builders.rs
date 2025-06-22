//! Modern many query builder implementation using the trait-based system.
//!
//! This demonstrates how the new QueryBuilderGenerator trait can be used
//! to create cleaner, more maintainable query builders.

use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

use crate::common::field_analysis::{extract_field_names, FieldAnalysis};
use crate::common::query_builder_gen::{
    QueryBuilderConfig, QueryBuilderError, QueryBuilderGenerator, QueryType,
};
use crate::common::sql_helpers;

/// Modern implementation of the many query builder using traits
pub struct ModernManyQueryBuilder;

impl QueryBuilderGenerator for ModernManyQueryBuilder {
    fn query_type(&self) -> QueryType {
        QueryType::Many
    }

    fn builder_name(&self, config: &QueryBuilderConfig) -> proc_macro2::Ident {
        quote::format_ident!("{}ManyQueryBuilder", config.dbset_name)
    }

    fn filter_fields(&self, fields: &[FieldAnalysis]) -> Vec<FieldAnalysis> {
        fields
            .iter()
            .filter(|field| !field.is_key && !field.is_unique)
            .cloned()
            .collect()
    }

    fn generate_struct(
        &self,
        config: &QueryBuilderConfig,
        fields: &[FieldAnalysis],
    ) -> Result<TokenStream, QueryBuilderError> {
        let builder_name = self.builder_name(config);
        let field_declarations = fields.iter().map(|field| {
            let field_name = &field.name;
            let field_type = field.sql_type();
            quote! {
                #field_name: Option<#field_type>,
            }
        });

        Ok(quote! {
            pub struct #builder_name {
                #(#field_declarations)*
            }
        })
    }

    fn generate_methods(
        &self,
        config: &QueryBuilderConfig,
        fields: &[FieldAnalysis],
    ) -> Result<TokenStream, QueryBuilderError> {
        let builder_name = self.builder_name(config);
        let struct_name = quote::format_ident!("{}", config.struct_name);

        // Generate setter methods
        let setter_methods = fields.iter().map(|field| {
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

        // Generate struct initialization
        let field_inits = fields.iter().map(|field| {
            let field_name = &field.name;
            quote! {
                #field_name: None,
            }
        });

        // Generate execute method with proper SQL
        let sql_query = self.generate_sql_query(config, fields)?;
        let query_params = self.generate_query_params(fields)?;

        Ok(quote! {
            impl #builder_name {
                pub fn new() -> Self {
                    Self {
                        #(#field_inits)*
                    }
                }

                #(#setter_methods)*

                pub async fn fetch_all<'e, E: sqlx::PgExecutor<'e>>(
                    self,
                    executor: E,
                ) -> Result<Vec<#struct_name>, sqlx::Error> {
                    sqlx::query_as!(
                        #struct_name,
                        #sql_query,
                        #query_params
                    )
                    .fetch_all(executor)
                    .await
                }
            }
        })
    }

    fn generate_sql_query(
        &self,
        config: &QueryBuilderConfig,
        fields: &[FieldAnalysis],
    ) -> Result<String, QueryBuilderError> {
        // For now, use the existing pattern from the original many query builder
        // This would be enhanced with proper field analysis in a real implementation
        let field_names = extract_field_names(fields);

        if fields.is_empty() {
            // If no filter fields, select all - this would need all struct fields
            Ok(format!("SELECT * FROM {}", config.table_name))
        } else {
            let where_clause = sql_helpers::generate_optional_where_clause(&field_names);
            Ok(format!(
                "SELECT * FROM {} {}",
                config.table_name, where_clause
            ))
        }
    }

    fn generate_query_params(
        &self,
        fields: &[FieldAnalysis],
    ) -> Result<TokenStream, QueryBuilderError> {
        let params = fields.iter().map(|field| {
            let field_name = &field.name;
            if field.is_custom_enum {
                let field_type = &field.field_type;
                quote! {
                    self.#field_name as Option<#field_type>,
                }
            } else {
                quote! {
                    self.#field_name,
                }
            }
        });
        Ok(quote! {
            #(#params)*
        })
    }
}

/// Helper function to generate a many query builder using the modern system
pub fn generate_modern_many_query_builder(input: &DeriveInput) -> TokenStream {
    let generator = ModernManyQueryBuilder;
    generator.generate(input).unwrap_or_else(|e| {
        let msg = format!("ModernManyQueryBuilder error: {:?}", e);
        quote! { compile_error!(#msg); }
    })
}
