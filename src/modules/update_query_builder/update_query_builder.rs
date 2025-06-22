use proc_macro2::Ident;
use quote::quote;
use syn::{Attribute, DeriveInput, Type};

use crate::common::utils::{
    get_all_fields, get_auto_fields, get_dbset_name, get_key_fields, get_query_fields_string,
    get_struct_name, get_table_name, is_custom_enum_attr,
};

/// Helper: filter out auto fields
fn filter_updatable_fields<'a>(
    all_fields: &'a [(&Ident, &Type, &Vec<Attribute>)],
    auto_fields: &[(&Ident, &Type, &Vec<Attribute>)]
) -> Vec<&'a (&'a Ident, &'a Type, &'a Vec<Attribute>)> {
    all_fields.iter().filter(|(field, _, _)| {
        !auto_fields.iter().any(|(auto_field, _, _)| auto_field == field)
    }).collect()
}

/// Helper: get set fields for SQL
fn get_set_fields<'a>(
    all_fields: &'a [(&'a Ident, &'a Type, &'a Vec<Attribute>)],
    key_fields: &[(&'a Ident, &'a Type)]
) -> Vec<&'a Ident> {
    all_fields.iter()
        .filter(|(ident, _, _)| !key_fields.iter().any(|(kf_ident, _)| kf_ident == ident))
        .map(|(ident, _, _)| *ident)
        .collect()
}

/// Helper: build SQL update query string
fn build_update_query(
    table_name: &str,
    set_fields: &[&Ident],
    key_fields: &[(&Ident, &Type)],
    all_fields_str: &str
) -> String {
    let where_size = key_fields.len();
    let query_builder_where_fields = key_fields
        .iter()
        .enumerate()
        .map(|(index, (field_name, _))| format!("{} = ${}", field_name, index + 1))
        .collect::<Vec<_>>()
        .join(" AND ");
    let set_fields_str = set_fields
        .iter()
        .enumerate()
        .map(|(index, field_name)| format!("{} = ${}", field_name, index + where_size + 1))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "UPDATE {table_name} SET {set_fields_str} WHERE {query_builder_where_fields} RETURNING {all_fields_str};"
    )
}

/// Helper: build query args
fn build_update_query_args(
    all_fields: &[(&Ident, &Type, &Vec<Attribute>)]
) -> Vec<proc_macro2::TokenStream> {
    all_fields.iter().map(|(name, ty, attrs)| {
        let is_custom_enum = attrs.iter().any(is_custom_enum_attr);
        if is_custom_enum {
            quote! { self.updatable.#name as #ty, }
        } else {
            quote! { self.updatable.#name, }
        }
    }).collect()
}

/// Get the update builder struct name
pub fn get_update_builder_struct_name(input: &DeriveInput) -> Ident {
    let dbset_name = get_dbset_name(input);
    quote::format_ident!("{}UpdateBuilder", dbset_name)
}

/// Generate the update query builder (modular, extensible)
pub fn get_update_query_builder(input: &DeriveInput) -> proc_macro2::TokenStream {
    let table_name = get_table_name(input);
    let struct_name = get_struct_name(input);
    let builder_struct_name = get_update_builder_struct_name(input);
    let all_fields = get_all_fields(input);
    let auto_fields = get_auto_fields(input);
    let all_fields_str = get_query_fields_string(input);
    let key_fields = get_key_fields(input);

    // Modular: filter updatable fields and set fields
    let updatable_fields = filter_updatable_fields(&all_fields, &auto_fields);
    let set_fields = get_set_fields(&all_fields, &key_fields);

    // Modular: build SQL and args
    let query = build_update_query(&table_name, &set_fields, &key_fields, &all_fields_str);
    let query_args = build_update_query_args(&all_fields);

    let builder_struct_name_with_data = quote::format_ident!("{}WithData", builder_struct_name);

    // Struct definitions
    let builder_struct = quote! {
        pub struct #builder_struct_name  {}
        pub struct #builder_struct_name_with_data  {
             updatable: #struct_name,
        }
    };

    // Builder impls
    let new_impl = quote! {
        pub fn new() -> #builder_struct_name  {
            Self {}
        }
    };
    let builder_method = quote! {
        impl #builder_struct_name {
            pub fn data(self, value: #struct_name) -> #builder_struct_name_with_data  {
                #builder_struct_name_with_data  {
                    updatable: value,
                }
            }
        }
    };
    let update_method = quote! {
        impl  #builder_struct_name_with_data  {
            pub async fn update<'e, E: sqlx::PgExecutor<'e>>(
                self,
                executor: E,
            ) -> Result<#struct_name, sqlx::Error> {
                sqlx::query_as!(
                    #struct_name,
                    #query,
                    #(#query_args)*
                )
                .fetch_one(executor)
                .await
            }
        }
    };

    // Compose all parts
    quote! {
        #builder_struct
        impl #builder_struct_name {
            #new_impl
        }
        #builder_method
        #update_method
    }
}
