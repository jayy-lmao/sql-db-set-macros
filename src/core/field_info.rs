//! Field information extraction and management.
//!
//! This module provides utilities for analyzing struct fields and their attributes,
//! extracting metadata needed for query generation.

use proc_macro2::Ident;
use syn::{Attribute, Type};

/// Represents different field categories for query generation
#[derive(Debug, Clone, PartialEq)]
pub enum FieldCategory {
    /// Primary key field(s)
    Key,
    /// Unique fields that can be used for single-record queries
    Unique,
    /// Auto-generated fields (excluded from inserts)
    Auto,
    /// Custom enum fields requiring special SQL handling
    CustomEnum,
    /// Regular data fields
    Regular,
}

/// Comprehensive field information used throughout query generation
#[derive(Debug, Clone)]
pub struct FieldInfo {
    /// Field name identifier
    pub name: Ident,
    /// Field type
    pub field_type: Type,
    /// Field attributes
    pub attributes: Vec<Attribute>,
    /// Field categories (a field can have multiple categories)
    pub categories: Vec<FieldCategory>,
    /// Whether the field is optional (Option<T>)
    pub is_optional: bool,
    /// Inner type for Option<T> fields
    pub inner_type: Option<Type>,
}

impl FieldInfo {
    /// Create a new FieldInfo from field components
    pub fn new(name: Ident, field_type: Type, attributes: Vec<Attribute>) -> Self {
        let categories = Self::extract_categories(&attributes);
        let is_optional = crate::common::utils::get_inner_option_type(&field_type).is_some();
        let inner_type = crate::common::utils::get_inner_option_type(&field_type)
            .map(|t| t.clone());

        Self {
            name,
            field_type,
            attributes,
            categories,
            is_optional,
            inner_type,
        }
    }

    /// Extract field categories from attributes
    fn extract_categories(attributes: &[Attribute]) -> Vec<FieldCategory> {
        let mut categories = Vec::new();

        for attr in attributes {
            match attr.meta {
                syn::Meta::Path(ref path) => {
                    if path.is_ident("key") {
                        categories.push(FieldCategory::Key);
                    } else if path.is_ident("unique") {
                        categories.push(FieldCategory::Unique);
                    } else if path.is_ident("auto") {
                        categories.push(FieldCategory::Auto);
                    } else if path.is_ident("custom_enum") {
                        categories.push(FieldCategory::CustomEnum);
                    }
                }
                _ => {}
            }
        }

        if categories.is_empty() {
            categories.push(FieldCategory::Regular);
        }

        categories
    }

    /// Check if field has a specific category
    pub fn has_category(&self, category: &FieldCategory) -> bool {
        self.categories.contains(category)
    }

    /// Check if field is a key field
    pub fn is_key(&self) -> bool {
        self.has_category(&FieldCategory::Key)
    }

    /// Check if field is unique
    pub fn is_unique(&self) -> bool {
        self.has_category(&FieldCategory::Unique)
    }

    /// Check if field is auto-generated
    pub fn is_auto(&self) -> bool {
        self.has_category(&FieldCategory::Auto)
    }

    /// Check if field is a custom enum
    pub fn is_custom_enum(&self) -> bool {
        self.has_category(&FieldCategory::CustomEnum)
    }

    /// Get the effective type for SQL queries (inner type for Option<T>)
    pub fn sql_type(&self) -> &Type {
        self.inner_type.as_ref().unwrap_or(&self.field_type)
    }
}
