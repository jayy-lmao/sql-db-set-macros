use quote::quote;
use syn::DeriveInput;

use crate::utils;

use super::utils::{
    get_many_query_builder_fields, get_many_query_builder_methods,
    get_many_query_builder_struct_fields_initial, get_many_query_builder_struct_fieldsl,
    get_many_query_builder_struct_name,
};

pub fn get_query_builder(input: &DeriveInput) -> proc_macro2::TokenStream {
    let struct_name = utils::get_struct_name(input);
    let table_name = utils::get_table_name(input);
    let query_builder_struct_name = get_many_query_builder_struct_name(input);
    let query_builder_fields = get_many_query_builder_fields(input);
    let query_builder_struct_fields = get_many_query_builder_struct_fieldsl(input);
    let query_builder_struct_fields_initial = get_many_query_builder_struct_fields_initial(input);
    let query_builder_methods = get_many_query_builder_methods(input);

    let all_fields = utils::get_all_fields(input);
    let query_builder_fetch = {
        let query_builder_where_fields = query_builder_fields
            .iter()
            .enumerate()
            .map(|(index, (field_name, _))| {
                format!(
                    "({} = ${} or ${} is null)",
                    field_name,
                    index + 1,
                    index + 1
                )
            })
            .collect::<Vec<_>>()
            .join(" AND ");

        let all_fields_str = all_fields
            .iter()
            .map(|(field_name, _)| field_name.to_string())
            .collect::<Vec<_>>()
            .join(", ");

        let full_where_clause = if !query_builder_where_fields.is_empty() {
            format!("WHERE {query_builder_where_fields}")
        } else {
            String::new()
        };

        let query = format!("SELECT {all_fields_str} FROM {table_name} {full_where_clause}");

        let query_args = query_builder_fields
            .iter()
            .map(|(field_name, _field_type)| {
                quote! {
                    self.#field_name,
                }
            });

        let res = quote! {
            pub async fn fetch<'e, E: sqlx::PgExecutor<'e>>(
                self,
                executor: E,
            ) -> Result<Vec<User>, sqlx::Error> {

                sqlx::query_as!(
                    #struct_name,
                    #query,
                    #(#query_args)*
                )
                .fetch_all(executor)
                .await
            }


        };
        res
    };

    quote! {
        #[derive(Debug)]
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
