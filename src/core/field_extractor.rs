//! Utilities for extracting field information from syn structures.
//!
//! This module provides functions to convert from syn types to our FieldInfo structures.

use syn::{DeriveInput, Data, Fields};

use super::FieldInfo;

/// Extract all field information from a DeriveInput
pub fn extract_fields(input: &DeriveInput) -> Vec<FieldInfo> {
    if let Data::Struct(data) = &input.data {
        match &data.fields {
            Fields::Named(fields) => fields
                .named
                .iter()
                .filter_map(|field| {
                    field.ident.as_ref().map(|ident| {
                        FieldInfo::new(
                            ident.clone(),
                            field.ty.clone(),
                            field.attrs.clone(),
                        )
                    })
                })
                .collect(),
            _ => panic!("DbSet can only be derived for structs with named fields"),
        }
    } else {
        panic!("DbSet can only be derived for structs");
    }
}
