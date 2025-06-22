//! Query builder generation traits and utilities.
//!
//! This module provides a robust, trait-based approach to generating different
//! types of query builders with shared patterns, validation, and reduced code duplication.

use proc_macro2::TokenStream;
use quote::quote;
use std::collections::HashMap;
use syn::DeriveInput;

use crate::common::field_analysis::{
    analyze_struct_fields, extract_field_names, get_insert_fields, get_key_fields_from_analysis,
    get_many_query_fields, get_one_query_fields, FieldAnalysis,
};
use crate::common::utils::{get_dbset_name, get_table_name};

/// Query type enumeration for different builder patterns
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum QueryType {
    /// Query that returns many records with optional filtering
    Many,
    /// Query that returns one record by key or unique field
    One,
    /// Query that inserts a new record
    Insert,
    /// Query that updates an existing record
    Update,
    /// Query that deletes a record
    Delete,
}

impl QueryType {
    /// Get a human-readable description of the query type
    pub fn description(&self) -> &'static str {
        match self {
            QueryType::Many => "many records query with optional filtering",
            QueryType::One => "single record query by key or unique field",
            QueryType::Insert => "record insertion query",
            QueryType::Update => "record update query",
            QueryType::Delete => "record deletion query",
        }
    }

    /// Check if this query type typically returns multiple records
    pub fn returns_multiple(&self) -> bool {
        matches!(self, QueryType::Many)
    }

    /// Check if this query type modifies data
    pub fn is_mutating(&self) -> bool {
        matches!(
            self,
            QueryType::Insert | QueryType::Update | QueryType::Delete
        )
    }
}

/// Configuration for query builder generation with validation
#[derive(Clone, Debug)]
pub struct QueryBuilderConfig {
    pub query_type: QueryType,
    pub struct_name: String,
    pub table_name: String,
    pub dbset_name: String,
    pub custom_settings: HashMap<String, String>,
}

impl QueryBuilderConfig {
    /// Create a new configuration with validation
    pub fn new(input: &DeriveInput, query_type: QueryType) -> Result<Self, QueryBuilderError> {
        let struct_name = input.ident.to_string();
        let table_name = get_table_name(input);
        let dbset_name = get_dbset_name(input).to_string();

        // Validate configuration
        if struct_name.is_empty() {
            return Err(QueryBuilderError::InvalidConfig(
                "Empty struct name".to_string(),
            ));
        }

        if table_name.is_empty() {
            return Err(QueryBuilderError::InvalidConfig(
                "Empty table name".to_string(),
            ));
        }

        if dbset_name.is_empty() {
            return Err(QueryBuilderError::InvalidConfig(
                "Empty dbset name".to_string(),
            ));
        }

        Ok(Self {
            query_type,
            struct_name,
            table_name,
            dbset_name,
            custom_settings: HashMap::new(),
        })
    }

    /// Add a custom setting to the configuration
    pub fn with_setting(mut self, key: String, value: String) -> Self {
        self.custom_settings.insert(key, value);
        self
    }

    /// Get a custom setting value
    pub fn get_setting(&self, key: &str) -> Option<&String> {
        self.custom_settings.get(key)
    }

    /// Validate that this configuration is compatible with the given fields
    pub fn validate_fields(&self, fields: &[FieldAnalysis]) -> Result<(), QueryBuilderError> {
        match self.query_type {
            QueryType::One => {
                let one_fields = get_one_query_fields(fields);
                if one_fields.is_empty() {
                    return Err(QueryBuilderError::InvalidConfig(
                        "One query requires at least one key or unique field".to_string(),
                    ));
                }
            }
            QueryType::Update | QueryType::Delete => {
                let key_fields = get_key_fields_from_analysis(fields);
                if key_fields.is_empty() {
                    return Err(QueryBuilderError::InvalidConfig(format!(
                        "{} query requires at least one key field",
                        if matches!(self.query_type, QueryType::Update) {
                            "Update"
                        } else {
                            "Delete"
                        }
                    )));
                }
            }
            QueryType::Insert => {
                let insert_fields = get_insert_fields(fields);
                if insert_fields.is_empty() {
                    return Err(QueryBuilderError::InvalidConfig(
                        "Insert query requires at least one insertable field".to_string(),
                    ));
                }
            }
            QueryType::Many => {
                // Many queries are more permissive, but still need some fields
                if fields.is_empty() {
                    return Err(QueryBuilderError::InvalidConfig(
                        "Query requires at least one field".to_string(),
                    ));
                }
            }
        }
        Ok(())
    }

    /// Get the appropriate builder name suffix for this query type
    pub fn builder_suffix(&self) -> &'static str {
        match self.query_type {
            QueryType::Many => "ManyQueryBuilder",
            QueryType::One => "OneQueryBuilder",
            QueryType::Insert => "InsertQueryBuilder",
            QueryType::Update => "UpdateQueryBuilder",
            QueryType::Delete => "DeleteQueryBuilder",
        }
    }
}

/// Error types for query builder generation
#[derive(Debug, Clone)]
pub enum QueryBuilderError {
    /// Invalid configuration provided
    InvalidConfig(String),
    /// Field analysis error
    FieldError(String),
    /// SQL generation error
    SqlError(String),
    /// Code generation error
    CodeGenError(String),
}

/// Advanced trait for generating query builder structs and implementations
pub trait QueryBuilderGenerator {
    /// Get the query type this generator handles
    fn query_type(&self) -> QueryType;

    /// Get the capabilities of this generator
    fn capabilities(&self) -> QueryBuilderCapabilities {
        QueryBuilderCapabilities::default()
    }

    /// Get the name of the query builder struct
    fn builder_name(&self, config: &QueryBuilderConfig) -> proc_macro2::Ident {
        quote::format_ident!("{}{}", config.dbset_name, config.builder_suffix())
    }

    /// Filter fields that should be included in this query builder
    fn filter_fields(&self, fields: &[FieldAnalysis]) -> Vec<FieldAnalysis> {
        match self.query_type() {
            QueryType::Many => get_many_query_fields(fields).into_iter().cloned().collect(),
            QueryType::One => get_one_query_fields(fields).into_iter().cloned().collect(),
            QueryType::Insert => get_insert_fields(fields).into_iter().cloned().collect(),
            QueryType::Update => fields.to_vec(), // All fields for updates
            QueryType::Delete => get_key_fields_from_analysis(fields)
                .into_iter()
                .cloned()
                .collect(), // Only keys for deletes
        }
    }

    /// Validate configuration and fields before generation
    fn validate(
        &self,
        config: &QueryBuilderConfig,
        fields: &[FieldAnalysis],
    ) -> Result<(), QueryBuilderError> {
        config.validate_fields(fields)?;
        let filtered_fields = self.filter_fields(fields);

        if filtered_fields.is_empty() {
            return Err(QueryBuilderError::FieldError(format!(
                "No suitable fields found for {} query",
                config.query_type.description()
            )));
        }

        Ok(())
    }

    /// Generate the query builder struct definition
    fn generate_struct(
        &self,
        config: &QueryBuilderConfig,
        fields: &[FieldAnalysis],
    ) -> Result<TokenStream, QueryBuilderError>;

    /// Generate the query builder methods (setters and execute methods)
    fn generate_methods(
        &self,
        config: &QueryBuilderConfig,
        fields: &[FieldAnalysis],
    ) -> Result<TokenStream, QueryBuilderError>;

    /// Generate SQL query for this builder type
    fn generate_sql_query(
        &self,
        config: &QueryBuilderConfig,
        fields: &[FieldAnalysis],
    ) -> Result<String, QueryBuilderError>;

    /// Generate query parameters for binding
    fn generate_query_params(
        &self,
        fields: &[FieldAnalysis],
    ) -> Result<TokenStream, QueryBuilderError>;

    /// Generate the complete query builder implementation with error handling
    fn generate(&self, input: &DeriveInput) -> Result<TokenStream, QueryBuilderError> {
        let config = QueryBuilderConfig::new(input, self.query_type())?;
        let all_fields = analyze_struct_fields(input);
        self.validate(&config, &all_fields)?;
        let filtered_fields = self.filter_fields(&all_fields);
        let struct_def = self.generate_struct(&config, &filtered_fields)?;
        let methods = self.generate_methods(&config, &filtered_fields)?;
        Ok(quote! {
            #struct_def
            #methods
        })
    }

    /// Generate with fallback error handling for legacy compatibility
    fn generate_safe(&self, input: &DeriveInput) -> TokenStream {
        match self.generate(input) {
            Ok(tokens) => tokens,
            Err(e) => {
                // Generate a compile-time error
                let error_msg = format!("Query builder generation failed: {:?}", e);
                quote! {
                    compile_error!(#error_msg);
                }
            }
        }
    }
}

/// Capabilities that a query builder generator can support
#[derive(Debug, Clone)]
pub struct QueryBuilderCapabilities {
    pub supports_async: bool,
    pub supports_transactions: bool,
    pub supports_custom_types: bool,
    pub supports_joins: bool,
    pub supports_aggregations: bool,
    pub supports_pagination: bool,
}

impl Default for QueryBuilderCapabilities {
    fn default() -> Self {
        Self {
            supports_async: true,
            supports_transactions: false,
            supports_custom_types: true,
            supports_joins: false,
            supports_aggregations: false,
            supports_pagination: false,
        }
    }
}

/// Base implementation for standard query builder generation
pub struct StandardQueryBuilder {
    query_type: QueryType,
    capabilities: QueryBuilderCapabilities,
}

impl StandardQueryBuilder {
    pub fn new(query_type: QueryType) -> Self {
        Self {
            query_type,
            capabilities: QueryBuilderCapabilities::default(),
        }
    }

    pub fn with_capabilities(mut self, capabilities: QueryBuilderCapabilities) -> Self {
        self.capabilities = capabilities;
        self
    }
}

impl QueryBuilderGenerator for StandardQueryBuilder {
    fn query_type(&self) -> QueryType {
        self.query_type.clone()
    }

    fn capabilities(&self) -> QueryBuilderCapabilities {
        self.capabilities.clone()
    }

    fn generate_struct(
        &self,
        config: &QueryBuilderConfig,
        fields: &[FieldAnalysis],
    ) -> Result<TokenStream, QueryBuilderError> {
        let builder_name = self.builder_name(config);
        let field_declarations = generate_struct_fields(fields);

        Ok(quote! {
            pub struct #builder_name {
                #field_declarations
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

        let setter_methods = generate_setter_methods(fields);
        let struct_init = generate_struct_init(fields);
        let execute_method = self.generate_execute_method(config, fields)?;

        Ok(quote! {
            impl #builder_name {
                pub fn new() -> Self {
                    #struct_init
                }

                #setter_methods
                #execute_method
            }
        })
    }

    fn generate_sql_query(
        &self,
        config: &QueryBuilderConfig,
        fields: &[FieldAnalysis],
    ) -> Result<String, QueryBuilderError> {
        match config.query_type {
            QueryType::Many => {
                let field_names = extract_field_names(fields);
                let where_conditions = fields
                    .iter()
                    .enumerate()
                    .map(|(i, field)| {
                        format!("({} = ${} OR ${} IS NULL)", field.name, i + 1, i + 1)
                    })
                    .collect::<Vec<_>>()
                    .join(" AND ");

                Ok(format!(
                    "SELECT {} FROM {} WHERE {}",
                    field_names.join(", "),
                    config.table_name,
                    where_conditions
                ))
            }
            QueryType::One => {
                let all_fields = format!("*"); // For one queries, select all fields
                let where_conditions = fields
                    .iter()
                    .enumerate()
                    .map(|(i, field)| format!("{} = ${}", field.name, i + 1))
                    .collect::<Vec<_>>()
                    .join(" OR ");

                Ok(format!(
                    "SELECT {} FROM {} WHERE {} LIMIT 1",
                    all_fields, config.table_name, where_conditions
                ))
            }
            QueryType::Insert => {
                let field_names = extract_field_names(fields);
                let placeholders = (1..=fields.len())
                    .map(|i| format!("${}", i))
                    .collect::<Vec<_>>();

                Ok(format!(
                    "INSERT INTO {} ({}) VALUES ({}) RETURNING *",
                    config.table_name,
                    field_names.join(", "),
                    placeholders.join(", ")
                ))
            }
            QueryType::Update => {
                let key_fields = get_key_fields_from_analysis(fields);
                if key_fields.is_empty() {
                    return Err(QueryBuilderError::SqlError(
                        "Update requires key fields".to_string(),
                    ));
                }

                let set_clauses = fields
                    .iter()
                    .enumerate()
                    .filter(|(_, field)| !field.is_key)
                    .map(|(i, field)| format!("{} = ${}", field.name, i + 1))
                    .collect::<Vec<_>>()
                    .join(", ");

                let where_clauses = key_fields
                    .iter()
                    .enumerate()
                    .map(|(i, field)| {
                        format!(
                            "{} = ${}",
                            field.name,
                            fields.len() - key_fields.len() + i + 1
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(" AND ");

                Ok(format!(
                    "UPDATE {} SET {} WHERE {} RETURNING *",
                    config.table_name, set_clauses, where_clauses
                ))
            }
            QueryType::Delete => {
                let where_clauses = fields
                    .iter()
                    .enumerate()
                    .map(|(i, field)| format!("{} = ${}", field.name, i + 1))
                    .collect::<Vec<_>>()
                    .join(" AND ");

                Ok(format!(
                    "DELETE FROM {} WHERE {}",
                    config.table_name, where_clauses
                ))
            }
        }
    }

    fn generate_query_params(
        &self,
        fields: &[FieldAnalysis],
    ) -> Result<TokenStream, QueryBuilderError> {
        let params = fields.iter().map(|field| {
            let field_name = &field.name;
            quote! { self.#field_name }
        });

        Ok(quote! {
            #(#params),*
        })
    }
}

impl StandardQueryBuilder {
    fn generate_execute_method(
        &self,
        config: &QueryBuilderConfig,
        fields: &[FieldAnalysis],
    ) -> Result<TokenStream, QueryBuilderError> {
        let struct_name = quote::format_ident!("{}", config.struct_name);
        let sql_query = self.generate_sql_query(config, fields)?;
        let query_params = self.generate_query_params(fields)?;

        match config.query_type {
            QueryType::Many => Ok(quote! {
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
            }),
            QueryType::One => Ok(quote! {
                pub async fn fetch_one<'e, E: sqlx::PgExecutor<'e>>(
                    self,
                    executor: E,
                ) -> Result<#struct_name, sqlx::Error> {
                    sqlx::query_as!(
                        #struct_name,
                        #sql_query,
                        #query_params
                    )
                    .fetch_one(executor)
                    .await
                }

                pub async fn fetch_optional<'e, E: sqlx::PgExecutor<'e>>(
                    self,
                    executor: E,
                ) -> Result<Option<#struct_name>, sqlx::Error> {
                    sqlx::query_as!(
                        #struct_name,
                        #sql_query,
                        #query_params
                    )
                    .fetch_optional(executor)
                    .await
                }
            }),
            QueryType::Insert | QueryType::Update => Ok(quote! {
                pub async fn execute<'e, E: sqlx::PgExecutor<'e>>(
                    self,
                    executor: E,
                ) -> Result<#struct_name, sqlx::Error> {
                    sqlx::query_as!(
                        #struct_name,
                        #sql_query,
                        #query_params
                    )
                    .fetch_one(executor)
                    .await
                }
            }),
            QueryType::Delete => Ok(quote! {
                pub async fn execute<'e, E: sqlx::PgExecutor<'e>>(
                    self,
                    executor: E,
                ) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
                    sqlx::query!(
                        #sql_query,
                        #query_params
                    )
                    .execute(executor)
                    .await
                }
            }),
        }
    }
}

/// Generate basic setter methods for query builder fields
pub fn generate_setter_methods(fields: &[FieldAnalysis]) -> TokenStream {
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
pub fn generate_struct_fields(fields: &[FieldAnalysis]) -> TokenStream {
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
pub fn generate_struct_init(fields: &[FieldAnalysis]) -> TokenStream {
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

fn get_key_fields_from_refs<'a>(fields: &'a [&FieldAnalysis]) -> Vec<&'a FieldAnalysis> {
    fields.iter().copied().filter(|f| f.is_key).collect()
}
