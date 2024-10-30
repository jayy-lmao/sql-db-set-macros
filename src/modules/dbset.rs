use quote::quote;
use syn::DeriveInput;

use crate::utils;

pub fn get_dbset_impl(input: &DeriveInput) -> proc_macro2::TokenStream {
    let many_query_builder_struct_name = utils::get_many_query_builder_struct_name(input);
    let struct_name = utils::get_struct_name(input);
    let key_fields = utils::get_key_fields(input);
    let unique_fields = utils::get_unique_fields(input);
    let all_fields = utils::get_all_fields(input);
    let dbset_name = utils::get_dbset_name(input);
    let table_name = utils::get_table_name(input);

    let key_method = {
        let all_keys_name = utils::join_field_names(&key_fields, "_and_");
        let all_fields_comma_sep = utils::join_field_names(&key_fields, ", ");

        let all_keys_where_clauses = key_fields
            .iter()
            .enumerate()
            .map(|(index, (field_name, _))| format!("{} = ${}", field_name, index + 1))
            .collect::<Vec<_>>()
            .join(" AND ");

        let method_name = quote::format_ident!("by_{}", all_keys_name);

        let query = format!(
            "SELECT {all_fields_comma_sep} FROM {table_name} WHERE {all_keys_where_clauses}"
        );
        let docstring = format!(
            "Get a `{struct_name}` by it's `{all_fields_comma_sep}` field.\n\nEquivalent of sqlx's `fetch_one` method."
        );

        let key_function_args = key_fields.iter().map(|(field_name, field_type)| {
            quote! {
                #field_name: #field_type,
            }
        });

        let key_query_args = key_fields.iter().map(|(field_name, _field_type)| {
            quote! {
                #field_name,
            }
        });

        quote! {
            #[doc = #docstring]
            pub async fn #method_name<'e, E: sqlx::PgExecutor<'e>>(
                executor: E,
                #(#key_function_args)*
            ) -> sqlx::Result<#struct_name> {
                sqlx::query_as!(
                    #struct_name,
                    #query,
                    #(#key_query_args)*
                )
                 .fetch_one(executor)
                 .await
            }
        }
    };

    let has_single_key_field = key_fields.len() == 1;
    let unique_methods = unique_fields.iter().filter(|(field_name,_)| {

    if  let Some((key_field_name,_)) = key_fields.first() {
        has_single_key_field  && key_field_name != field_name
    } else { false }

})
    .map(|(field_name, field_type)| {
    let all_fields_str = all_fields
        .iter()
        .map(|(field_name, _)| field_name.to_string())
        .collect::<Vec<_>>()
        .join(", ");

    let method_name = quote::format_ident!("by_{}", field_name);
    let query = format!("SELECT {all_fields_str} FROM {table_name} WHERE {field_name} = ");
    let docstring = format!(
        "Get a `{struct_name}` by it's `{field_name}` field.\n\nEquivalent of sqlx's `fetch_one` method."
    );

    quote! {
        #[doc = #docstring]
        pub async fn #method_name<'e, E: sqlx::PgExecutor<'e>>(
            executor: E,
            #field_name: #field_type,
        ) -> sqlx::Result<#struct_name> {
            sqlx::query_as!(
                #struct_name,
                #query,
                #field_name,
            )
             .fetch_one(executor)
             .await
        }
    }
});

    quote! {
        pub struct #dbset_name;

        impl #dbset_name {
            #key_method
            #(#unique_methods)*

            pub fn many() -> #many_query_builder_struct_name {
                #many_query_builder_struct_name::new()
            }
        }
    }
}
