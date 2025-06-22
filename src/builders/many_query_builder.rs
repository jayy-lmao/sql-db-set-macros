//! Many query builder - for fetching multiple records.
//!
//! This builder generates queries that can return multiple records based on
//! optional field conditions.

use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

use crate::core::{
    FieldInfo, QueryBuilderGenerator, FieldFilter, BuilderMethodGenerator,
    BaseQueryBuilder, SelectClauseGenerator, WhereClauseGenerator, ParameterGenerator,
};

pub struct ManyQueryBuilder;

impl FieldFilter for ManyQueryBuilder {
    fn filter_fields<'a>(&self, fields: &'a [FieldInfo]) -> Vec<&'a FieldInfo> {
        // For many queries, we exclude key fields (since they're for unique identification)
        // and unique fields (since they're for single-record queries)
        fields
            .iter()
            .filter(|field| !field.is_key() && !field.is_unique())
            .collect()
    }

    fn required_fields<'a>(&self, _fields: &'a [FieldInfo]) -> Vec<&'a FieldInfo> {
        // Many queries don't require any fields - you can fetch all records
        Vec::new()
    }
}

impl BuilderMethodGenerator for ManyQueryBuilder {
    fn generate_methods(&self, fields: &[FieldInfo]) -> TokenStream {
        let base_builder = BaseQueryBuilder {
            builder_name: "ManyQueryBuilder".to_string(),
            struct_name: "".to_string(),
            table_name: "".to_string(),
        };
        base_builder.generate_setter_methods(fields)
    }

    fn generate_execute_method(&self, fields: &[FieldInfo], table_name: &str) -> TokenStream {
        let select_clause = SelectClauseGenerator::generate(fields);
        let where_clause = if !fields.is_empty() {
            let conditions = WhereClauseGenerator::generate_optional_conditions(fields);
            format!("WHERE {conditions}")
        } else {
            String::new()
        };

        let query = format!("SELECT {select_clause} FROM {table_name} {where_clause}");
        let parameters = ParameterGenerator::generate(fields);

        quote! {
            pub async fn fetch_all<'e, E: sqlx::PgExecutor<'e>>(
                self,
                executor: E,
            ) -> Result<Vec<super::super::#struct_name>, sqlx::Error> {
                sqlx::query_as!(
                    super::super::#struct_name,
                    #query,
                    #parameters
                )
                .fetch_all(executor)
                .await
            }
        }
    }
}

impl QueryBuilderGenerator for ManyQueryBuilder {
    fn generate(&self, input: &DeriveInput, all_fields: &[FieldInfo]) -> TokenStream {
        let struct_name = &input.ident;
        let table_name = crate::common::utils::get_table_name(input);
        let builder_name = self.builder_name(input);
        
        let filtered_fields = self.filter_fields(all_fields);
        let filtered_field_refs: Vec<_> = filtered_fields.iter().map(|&f| f).collect();
        
        let base_builder = BaseQueryBuilder::new(input);
        
        let struct_fields = base_builder.generate_struct_fields(&filtered_field_refs);
        let struct_init = base_builder.generate_struct_init(&filtered_field_refs);
        let methods = self.generate_methods(&filtered_field_refs);
        
        // Generate execute method with proper struct name access
        let select_clause = SelectClauseGenerator::generate(all_fields);
        let where_clause = if !filtered_field_refs.is_empty() {
            let conditions = WhereClauseGenerator::generate_optional_conditions(&filtered_field_refs);
            format!("WHERE {conditions}")
        } else {
            String::new()
        };

        let query = format!("SELECT {select_clause} FROM {table_name} {where_clause}");
        let parameters = ParameterGenerator::generate(&filtered_field_refs);

        let execute_method = quote! {
            pub async fn fetch_all<'e, E: sqlx::PgExecutor<'e>>(
                self,
                executor: E,
            ) -> Result<Vec<#struct_name>, sqlx::Error> {
                sqlx::query_as!(
                    #struct_name,
                    #query,
                    #parameters
                )
                .fetch_all(executor)
                .await
            }
        };

        quote! {
            pub struct #builder_name {
                #struct_fields
            }

            impl #builder_name {
                pub fn new() -> Self {
                    #struct_init
                }

                #methods
                #execute_method
            }
        }
    }

    fn builder_name(&self, input: &DeriveInput) -> proc_macro2::Ident {
        let dbset_name = crate::common::utils::get_dbset_name(input);
        quote::format_ident!("{}ManyQueryBuilder", dbset_name)
    }
}
