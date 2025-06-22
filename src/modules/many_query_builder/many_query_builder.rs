use proc_macro2::Ident;
use quote::quote;
use syn::{Attribute, DeriveInput, Type};

use crate::{
    common::utils::{
        get_all_fields, get_key_fields, get_query_fields_string, get_unique_fields,
        is_custom_enum_attr,
    },
    utils,
};

use super::utils::{
    get_many_query_builder_methods, get_many_query_builder_struct_fields,
    get_many_query_builder_struct_fields_initial, get_many_query_builder_struct_name,
};

fn fetch_all_method<'a>(
    struct_name: &Ident,
    table_name: &str,
    query_fields_string: &str,
    fields_to_include: &[(&'a Ident, &'a Type, &'a Vec<Attribute>)],
) -> proc_macro2::TokenStream {
    let query_builder_where_fields = fields_to_include
        .iter()
        .enumerate()
        .map(|(index, (field_name, _, _))| {
            format!(
                "({} = ${} or ${} is null)",
                field_name,
                index + 1,
                index + 1
            )
        })
        .collect::<Vec<_>>()
        .join(" AND ");

    let full_where_clause = if !query_builder_where_fields.is_empty() {
        format!("WHERE {query_builder_where_fields}")
    } else {
        String::new()
    };

    let query = format!("SELECT {query_fields_string} FROM {table_name} {full_where_clause}");

    let query_args = fields_to_include
        .iter()
        .map(|(field_name, field_type, attrs)| {
            let is_custom_enum = attrs.iter().any(is_custom_enum_attr);
            if is_custom_enum {
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
        pub async fn fetch_all<'e, E: sqlx::PgExecutor<'e>>(
            self,
            executor: E,
        ) -> Result<Vec<#struct_name>, sqlx::Error> {
            sqlx::query_as!(
                #struct_name,
                #query,
                #(#query_args)*
            )
            .fetch_all(executor)
            .await
        }
    }
}

pub fn get_query_builder(input: &DeriveInput) -> proc_macro2::TokenStream {
    let struct_name = utils::get_struct_name(input);
    let table_name = utils::get_table_name(input);
    let query_builder_struct_name = get_many_query_builder_struct_name(input);
    let query_builder_struct_fields = get_many_query_builder_struct_fields(input);
    let query_builder_struct_fields_initial = get_many_query_builder_struct_fields_initial(input);
    let query_builder_methods = get_many_query_builder_methods(input);
    let unique_fields = get_unique_fields(input);
    let key_fields = get_key_fields(input);
    let all_fields = get_all_fields(input);
    let query_fields_string = get_query_fields_string(input);

    let fields_to_include: Vec<(&Ident, &Type, &Vec<Attribute>)> = {
        let mut fields_to_include = vec![];
        for field in all_fields.clone() {
            if unique_fields.iter().any(|(ufn, _)| *ufn == field.0) {
                continue;
            }
            if key_fields.len() == 1 && key_fields.iter().any(|(ufn, _)| *ufn == field.0) {
                continue;
            }
            fields_to_include.push(field);
        }
        fields_to_include
    };

    let query_builder_fetch = fetch_all_method(
        struct_name,
        &table_name,
        &query_fields_string,
        &fields_to_include,
    );

    quote! {
        pub struct #query_builder_struct_name {
            #(#query_builder_struct_fields),*
        }

        impl #query_builder_struct_name {
            pub fn new() -> Self {
                Self {
                    #(#query_builder_struct_fields_initial),*
                }
            }
            #(#query_builder_methods)*
            #query_builder_fetch
        }
    }
}
