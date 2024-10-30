use quote::quote;
use syn::DeriveInput;

use crate::utils;

pub fn get_query_builder(input: &DeriveInput) -> proc_macro2::TokenStream {
    let struct_name = utils::get_struct_name(input);
    let dbset_name = utils::get_dbset_name(input);
    let table_name = utils::get_table_name(input);
    let query_builder_struct_name = quote::format_ident!("{}ManyQueryBuilder", dbset_name);
    let query_builder_fields = utils::get_query_builder_fields(input);
    let query_builder_struct_fields = utils::get_query_builder_struct_fields(input);
    let query_builder_struct_fields_initial = utils::get_query_builder_struct_fields_initial(input);
    let query_builder_methods = utils::get_query_builder_methods(input);

    let all_fields = utils::get_all_fields(input);
    let query_builder_fetch = {
        let all_query_builder_fields = query_builder_fields
            .iter()
            .enumerate()
            .map(|(index, (field_name, _))| {
                format!("{} = ${} or ${} = null", field_name, index + 1, index + 1)
            })
            .collect::<Vec<_>>()
            .join(" AND ");

        let all_fields_str = all_fields
            .iter()
            .map(|(field_name, _)| field_name.to_string())
            .collect::<Vec<_>>()
            .join(", ");

        let query =
            format!("SELECT {all_fields_str} FROM {table_name} WHERE {all_query_builder_fields}");
        let query_args = query_builder_fields
            .iter()
            .map(|(field_name, _field_type)| {
                quote! {
                    self.#field_name,
                }
            });

        quote! {
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

        }
    };

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
