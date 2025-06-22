//! SQL query generation helpers.
//!
//! This module provides reusable functions for generating common SQL patterns.

use crate::common::constants::*;

/// Generate a SELECT clause with optional type casting for enums
pub fn generate_select_clause(fields: &[String], custom_enum_fields: &[(String, String)]) -> String {
    let mut clauses = Vec::new();
    
    for field in fields {
        if let Some((_, field_type)) = custom_enum_fields.iter().find(|(name, _)| name == field) {
            clauses.push(format!("{field} AS \"{field}:{field_type}\""));
        } else {
            clauses.push(field.clone());
        }
    }
    
    clauses.join(", ")
}

/// Generate WHERE clause with optional conditions (field = $n OR $n IS NULL)
pub fn generate_optional_where_clause(fields: &[String]) -> String {
    if fields.is_empty() {
        return String::new();
    }
    
    let conditions: Vec<String> = fields
        .iter()
        .enumerate()
        .map(|(i, field)| {
            let param_num = i + 1;
            format!("({field} = ${param_num} OR ${param_num} IS NULL)")
        })
        .collect();
    
    format!("{SQL_WHERE} {}", conditions.join(&format!(" {SQL_AND} ")))
}

/// Generate WHERE clause with required conditions (field = $n)
pub fn generate_required_where_clause(fields: &[String]) -> String {
    if fields.is_empty() {
        return String::new();
    }
    
    let conditions: Vec<String> = fields
        .iter()
        .enumerate()
        .map(|(i, field)| {
            let param_num = i + 1;
            format!("{field} = ${param_num}")
        })
        .collect();
    
    format!("{SQL_WHERE} {}", conditions.join(&format!(" {SQL_AND} ")))
}

/// Generate INSERT statement with RETURNING clause
pub fn generate_insert_query(table_name: &str, fields: &[String]) -> String {
    let field_list = fields.join(", ");
    let placeholders: Vec<String> = (1..=fields.len())
        .map(|i| format!("${i}"))
        .collect();
    let placeholder_list = placeholders.join(", ");
    
    format!("{SQL_INSERT} {table_name} ({field_list}) {SQL_VALUES} ({placeholder_list}) {SQL_RETURNING} *")
}

/// Generate UPDATE statement with RETURNING clause
pub fn generate_update_query(table_name: &str, data_fields: &[String], key_fields: &[String]) -> String {
    let set_clauses: Vec<String> = data_fields
        .iter()
        .enumerate()
        .map(|(i, field)| format!("{field} = ${}", i + 1))
        .collect();
    let set_clause = set_clauses.join(", ");
    
    let where_conditions: Vec<String> = key_fields
        .iter()
        .enumerate()
        .map(|(i, field)| format!("{field} = ${}", data_fields.len() + i + 1))
        .collect();
    let where_clause = where_conditions.join(&format!(" {SQL_AND} "));
    
    format!("{SQL_UPDATE} {table_name} {SQL_SET} {set_clause} {SQL_WHERE} {where_clause} {SQL_RETURNING} *")
}

/// Generate DELETE statement
pub fn generate_delete_query(table_name: &str, fields: &[String]) -> String {
    let where_clause = generate_required_where_clause(fields)
        .strip_prefix(&format!("{SQL_WHERE} "))
        .unwrap_or("")
        .to_string();
    
    format!("{SQL_DELETE} {table_name} {SQL_WHERE} {where_clause}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_select_clause_simple() {
        let fields = vec!["id".to_string(), "name".to_string()];
        let custom_enums = vec![];
        let result = generate_select_clause(&fields, &custom_enums);
        assert_eq!(result, "id, name");
    }

    #[test]
    fn test_generate_select_clause_with_enum() {
        let fields = vec!["id".to_string(), "status".to_string()];
        let custom_enums = vec![("status".to_string(), "UserStatus".to_string())];
        let result = generate_select_clause(&fields, &custom_enums);
        assert_eq!(result, "id, status AS \"status:UserStatus\"");
    }

    #[test]
    fn test_generate_optional_where_clause() {
        let fields = vec!["name".to_string(), "email".to_string()];
        let result = generate_optional_where_clause(&fields);
        assert_eq!(result, "WHERE (name = $1 OR $1 IS NULL) AND (email = $2 OR $2 IS NULL)");
    }

    #[test]
    fn test_generate_insert_query() {
        let fields = vec!["name".to_string(), "email".to_string()];
        let result = generate_insert_query("users", &fields);
        assert_eq!(result, "INSERT INTO users (name, email) VALUES ($1, $2) RETURNING *");
    }
}
