//! Field analysis and categorization utilities.
//!
//! This module provides functions to analyze struct fields and categorize them
//! for different query generation purposes.

use proc_macro2::Ident;
use syn::{Attribute, Type, DeriveInput};

use crate::common::utils::{
    get_all_fields, is_custom_enum_attr, get_inner_option_type,
};

/// Represents a field with its metadata for query generation
#[derive(Clone)]
pub struct FieldAnalysis {
    pub name: Ident,
    pub field_type: Type,
    pub is_key: bool,
    pub is_unique: bool,
    pub is_auto: bool,
    pub is_custom_enum: bool,
    pub is_optional: bool,
    pub inner_type: Option<Type>,
}

impl FieldAnalysis {
    /// Create a new FieldAnalysis from field components
    pub fn new(name: Ident, field_type: Type, attributes: &[Attribute]) -> Self {
        let is_key = attributes.iter().any(crate::common::utils::is_key_attr);
        let is_unique = attributes.iter().any(crate::common::utils::is_unique_attr);
        let is_auto = attributes.iter().any(crate::common::utils::is_auto_attr);
        let is_custom_enum = attributes.iter().any(is_custom_enum_attr);
        let is_optional = get_inner_option_type(&field_type).is_some();
        let inner_type = get_inner_option_type(&field_type).map(|t| t.clone());

        Self {
            name,
            field_type,
            is_key,
            is_unique,
            is_auto,
            is_custom_enum,
            is_optional,
            inner_type,
        }
    }

    /// Get the effective type for SQL queries (inner type for Option<T>)
    pub fn sql_type(&self) -> &Type {
        self.inner_type.as_ref().unwrap_or(&self.field_type)
    }

    /// Check if this field should be included in many queries
    pub fn is_many_query_field(&self) -> bool {
        !self.is_key && !self.is_unique
    }

    /// Check if this field should be included in insert queries
    pub fn is_insert_field(&self) -> bool {
        !self.is_auto
    }

    /// Check if this field can be used for one queries
    pub fn is_one_query_field(&self) -> bool {
        self.is_key || self.is_unique
    }
}

/// Analyze all fields in a struct and categorize them
pub fn analyze_struct_fields(input: &DeriveInput) -> Vec<FieldAnalysis> {
    let all_fields = get_all_fields(input);
    
    all_fields
        .iter()
        .map(|(name, field_type, attributes)| {
            FieldAnalysis::new((*name).clone(), (*field_type).clone(), attributes)
        })
        .collect()
}

/// Get fields suitable for many queries (excluding keys and unique fields)
pub fn get_many_query_fields(analyses: &[FieldAnalysis]) -> Vec<&FieldAnalysis> {
    analyses.iter().filter(|f| f.is_many_query_field()).collect()
}

/// Get fields suitable for one queries (keys and unique fields)
pub fn get_one_query_fields(analyses: &[FieldAnalysis]) -> Vec<&FieldAnalysis> {
    analyses.iter().filter(|f| f.is_one_query_field()).collect()
}

/// Get fields suitable for insert queries (excluding auto fields)
pub fn get_insert_fields(analyses: &[FieldAnalysis]) -> Vec<&FieldAnalysis> {
    analyses.iter().filter(|f| f.is_insert_field()).collect()
}

/// Get key fields for update/delete operations
pub fn get_key_fields_from_analysis(analyses: &[FieldAnalysis]) -> Vec<&FieldAnalysis> {
    analyses.iter().filter(|f| f.is_key).collect()
}

/// Extract field names as strings
pub fn extract_field_names(analyses: &[&FieldAnalysis]) -> Vec<String> {
    analyses.iter().map(|f| f.name.to_string()).collect()
}

/// Extract custom enum field information
pub fn extract_custom_enum_info(analyses: &[&FieldAnalysis]) -> Vec<(String, String)> {
    analyses
        .iter()
        .filter(|f| f.is_custom_enum)
        .map(|f| {
            let name = f.name.to_string();
            let type_name = quote::ToTokens::to_token_stream(&f.field_type).to_string();
            (name, type_name)
        })
        .collect()
}
