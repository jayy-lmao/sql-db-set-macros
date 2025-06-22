//! SQL query generation utilities.
//!
//! This module provides reusable components for generating SQL queries
//! from field information.

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use super::{FieldInfo, FieldCategory};

/// Utility for generating SQL SELECT clauses with proper type casting
pub struct SelectClauseGenerator;

impl SelectClauseGenerator {
    /// Generate a SELECT clause from field information
    pub fn generate(fields: &[FieldInfo]) -> String {
        fields
            .iter()
            .map(|field| {
                if field.is_custom_enum() {
                    let field_name = field.name.to_string();
                    let field_type = field.field_type.to_token_stream().to_string();
                    format!("{field_name} AS \"{field_name}:{field_type}\"")
                } else {
                    field.name.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// Utility for generating WHERE clause conditions
pub struct WhereClauseGenerator;

impl WhereClauseGenerator {
    /// Generate WHERE conditions for optional parameters
    pub fn generate_optional_conditions(fields: &[FieldInfo]) -> String {
        fields
            .iter()
            .enumerate()
            .map(|(index, field)| {
                let field_name = field.name.to_string();
                format!(
                    "({field_name} = ${} or ${} is null)",
                    index + 1,
                    index + 1
                )
            })
            .collect::<Vec<_>>()
            .join(" AND ")
    }

    /// Generate WHERE conditions for required parameters
    pub fn generate_required_conditions(fields: &[FieldInfo]) -> String {
        fields
            .iter()
            .enumerate()
            .map(|(index, field)| {
                let field_name = field.name.to_string();
                format!("{field_name} = ${}", index + 1)
            })
            .collect::<Vec<_>>()
            .join(" AND ")
    }
}

/// Utility for generating query parameters
pub struct ParameterGenerator;

impl ParameterGenerator {
    /// Generate parameters for sqlx query macros
    pub fn generate(fields: &[FieldInfo]) -> TokenStream {
        let params = fields.iter().map(|field| {
            let field_name = &field.name;
            if field.is_custom_enum() {
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

        quote! {
            #(#params)*
        }
    }
}

/// Utility for generating INSERT queries
pub struct InsertQueryGenerator;

impl InsertQueryGenerator {
    /// Generate INSERT query with field list and value placeholders
    pub fn generate(table_name: &str, fields: &[FieldInfo]) -> String {
        let field_names = fields
            .iter()
            .map(|f| f.name.to_string())
            .collect::<Vec<_>>()
            .join(", ");

        let placeholders = (1..=fields.len())
            .map(|i| format!("${i}"))
            .collect::<Vec<_>>()
            .join(", ");

        format!("INSERT INTO {table_name} ({field_names}) VALUES ({placeholders}) RETURNING *")
    }
}

/// Utility for generating UPDATE queries
pub struct UpdateQueryGenerator;

impl UpdateQueryGenerator {
    /// Generate UPDATE query with SET clauses
    pub fn generate(table_name: &str, data_fields: &[FieldInfo], key_fields: &[FieldInfo]) -> String {
        let set_clauses = data_fields
            .iter()
            .enumerate()
            .map(|(i, field)| format!("{} = ${}", field.name, i + 1))
            .collect::<Vec<_>>()
            .join(", ");

        let where_conditions = key_fields
            .iter()
            .enumerate()
            .map(|(i, field)| format!("{} = ${}", field.name, data_fields.len() + i + 1))
            .collect::<Vec<_>>()
            .join(" AND ");

        format!("UPDATE {table_name} SET {set_clauses} WHERE {where_conditions} RETURNING *")
    }
}

/// Utility for generating DELETE queries
pub struct DeleteQueryGenerator;

impl DeleteQueryGenerator {
    /// Generate DELETE query with WHERE conditions
    pub fn generate(table_name: &str, fields: &[FieldInfo]) -> String {
        let where_conditions = WhereClauseGenerator::generate_required_conditions(fields);
        format!("DELETE FROM {table_name} WHERE {where_conditions}")
    }
}
