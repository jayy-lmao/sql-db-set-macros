//! Advanced configuration system for query builders.
//!
//! This module provides a flexible configuration system that allows
//! customization of query generation behavior.

use std::collections::HashMap;
use syn::DeriveInput;

use crate::common::field_analysis::FieldAnalysis;
use crate::common::query_builder_gen::QueryType;

/// Configuration options for customizing query generation
#[derive(Clone, Debug)]
pub struct DbSetConfig {
    /// Custom table name (overrides default)
    pub table_name: Option<String>,
    /// Custom DbSet name (overrides default)
    pub dbset_name: Option<String>,
    /// Whether to generate async methods (default: true)
    pub async_methods: bool,
    /// Whether to include validation methods
    pub include_validation: bool,
    /// Custom SQL generation options
    pub sql_options: SqlGenerationOptions,
    /// Field-specific overrides
    pub field_overrides: HashMap<String, FieldOverride>,
}

impl Default for DbSetConfig {
    fn default() -> Self {
        Self {
            table_name: None,
            dbset_name: None,
            async_methods: true,
            include_validation: false,
            sql_options: SqlGenerationOptions::default(),
            field_overrides: HashMap::new(),
        }
    }
}

/// Options for SQL generation customization
#[derive(Clone, Debug)]
pub struct SqlGenerationOptions {
    /// Whether to use prepared statements (default: true)
    pub use_prepared_statements: bool,
    /// Custom SQL fragments for specific operations
    pub custom_fragments: HashMap<QueryType, String>,
    /// Whether to include RETURNING clauses for mutations
    pub include_returning: bool,
    /// Whether to generate type-safe parameter binding
    pub type_safe_params: bool,
}

impl Default for SqlGenerationOptions {
    fn default() -> Self {
        Self {
            use_prepared_statements: true,
            custom_fragments: HashMap::new(),
            include_returning: true,
            type_safe_params: true,
        }
    }
}

/// Field-specific configuration overrides
#[derive(Clone, Debug)]
pub struct FieldOverride {
    /// Custom SQL column name
    pub column_name: Option<String>,
    /// Custom SQL type casting
    pub sql_type: Option<String>,
    /// Whether to exclude this field from certain operations
    pub exclude_from: Vec<QueryType>,
    /// Custom validation rules
    pub validation_rules: Vec<ValidationRule>,
}

/// Validation rules for fields
#[derive(Clone, Debug)]
pub enum ValidationRule {
    /// Field must not be empty/null
    Required,
    /// Field must match a regex pattern
    Pattern(String),
    /// Field must be within a range
    Range { min: Option<i64>, max: Option<i64> },
    /// Field must be one of the specified values
    OneOf(Vec<String>),
    /// Custom validation function name
    Custom(String),
}

/// Builder for creating DbSetConfig instances
pub struct DbSetConfigBuilder {
    config: DbSetConfig,
}

impl DbSetConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: DbSetConfig::default(),
        }
    }

    pub fn table_name<S: Into<String>>(mut self, name: S) -> Self {
        self.config.table_name = Some(name.into());
        self
    }

    pub fn dbset_name<S: Into<String>>(mut self, name: S) -> Self {
        self.config.dbset_name = Some(name.into());
        self
    }

    pub fn async_methods(mut self, enabled: bool) -> Self {
        self.config.async_methods = enabled;
        self
    }

    pub fn include_validation(mut self, enabled: bool) -> Self {
        self.config.include_validation = enabled;
        self
    }

    pub fn field_override<S: Into<String>>(
        mut self,
        field_name: S,
        override_config: FieldOverride,
    ) -> Self {
        self.config
            .field_overrides
            .insert(field_name.into(), override_config);
        self
    }

    pub fn sql_options(mut self, options: SqlGenerationOptions) -> Self {
        self.config.sql_options = options;
        self
    }

    pub fn build(self) -> DbSetConfig {
        self.config
    }
}

impl Default for DbSetConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract configuration from derive input attributes
pub fn extract_config_from_attributes(input: &DeriveInput) -> DbSetConfig {
    let mut config = DbSetConfig::default();

    // Look for #[dbset(...)] attributes
    for attr in &input.attrs {
        if let syn::Meta::List(meta) = &attr.meta {
            if meta.path.is_ident("dbset") {
                let _ = meta.parse_nested_meta(|meta| {
                    if meta.path.is_ident("table_name") {
                        if let Ok(value) = meta.value() {
                            if let Ok(lit) = value.parse::<syn::LitStr>() {
                                config.table_name = Some(lit.value());
                            }
                        }
                    } else if meta.path.is_ident("dbset_name") {
                        if let Ok(value) = meta.value() {
                            if let Ok(lit) = value.parse::<syn::LitStr>() {
                                config.dbset_name = Some(lit.value());
                            }
                        }
                    } else if meta.path.is_ident("async") {
                        if let Ok(value) = meta.value() {
                            if let Ok(lit) = value.parse::<syn::LitBool>() {
                                config.async_methods = lit.value();
                            }
                        }
                    } else if meta.path.is_ident("validation") {
                        if let Ok(value) = meta.value() {
                            if let Ok(lit) = value.parse::<syn::LitBool>() {
                                config.include_validation = lit.value();
                            }
                        }
                    }
                    Ok(())
                });
            }
        }
    }

    config
}

/// Apply configuration-based field filtering
pub fn apply_config_field_filter<'a>(
    fields: &'a [FieldAnalysis],
    config: &DbSetConfig,
    query_type: QueryType,
) -> Vec<&'a FieldAnalysis> {
    fields
        .iter()
        .filter(|field| {
            let field_name = field.name.to_string();

            // Check if field is excluded from this query type
            if let Some(override_config) = config.field_overrides.get(&field_name) {
                if override_config.exclude_from.contains(&query_type) {
                    return false;
                }
            }

            true
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_config_builder() {
        let config = DbSetConfigBuilder::new()
            .table_name("custom_users")
            .async_methods(false)
            .include_validation(true)
            .build();

        assert_eq!(config.table_name, Some("custom_users".to_string()));
        assert!(!config.async_methods);
        assert!(config.include_validation);
    }

    #[test]
    fn test_config_extraction() {
        let input: DeriveInput = parse_quote! {
            #[dbset(table_name = "users", async = true, validation = false)]
            pub struct User {
                id: String,
                name: String,
            }
        };

        let config = extract_config_from_attributes(&input);
        assert_eq!(config.table_name, Some("users".to_string()));
        assert!(config.async_methods);
        assert!(!config.include_validation);
    }

    #[test]
    fn test_field_filtering() {
        use crate::common::field_analysis::analyze_struct_fields;

        let input: DeriveInput = parse_quote! {
            pub struct User {
                id: String,
                name: String,
                email: String,
            }
        };

        let fields = analyze_struct_fields(&input);
        let mut config = DbSetConfig::default();

        // Exclude 'email' field from Many queries
        config.field_overrides.insert(
            "email".to_string(),
            FieldOverride {
                column_name: None,
                sql_type: None,
                exclude_from: vec![QueryType::Many],
                validation_rules: vec![],
            },
        );

        let filtered = apply_config_field_filter(&fields, &config, QueryType::Many);
        let field_names: Vec<_> = filtered.iter().map(|f| f.name.to_string()).collect();

        assert!(!field_names.contains(&"email".to_string()));
        assert!(field_names.contains(&"name".to_string()));
    }
}
