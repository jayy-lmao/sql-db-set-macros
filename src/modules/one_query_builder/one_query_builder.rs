use quote::quote;
use syn::DeriveInput;

use crate::utils;

use super::utils::{
    get_one_query_builder_key_fields, get_one_query_builder_key_methods,
    get_one_query_builder_struct_fields_initial, get_one_query_builder_struct_fieldsl,
    get_one_query_builder_struct_name, get_one_query_builder_unique_fields,
    get_one_query_builder_unique_methods,
};

pub enum Variants {
    UniqueFieldsExist,
    KeyFieldsExist,
    KeyFieldsAndUniqueFieldsExist,
    NeitherExist,
}

pub fn get_query_builder(input: &DeriveInput) -> proc_macro2::TokenStream {
    let struct_name = utils::get_struct_name(input);
    let table_name = utils::get_table_name(input);
    let query_builder_struct_name = get_one_query_builder_struct_name(input);
    let query_builder_key_fields = get_one_query_builder_key_fields(input);
    let query_builder_unique_fields = get_one_query_builder_unique_fields(input);
    let query_builder_struct_fields = get_one_query_builder_struct_fieldsl(input);
    let query_builder_struct_fields_initial = get_one_query_builder_struct_fields_initial(input);
    let query_builder_unique_methods = get_one_query_builder_unique_methods(input);
    let query_builder_key_methods = get_one_query_builder_key_methods(input);
    let all_fields = utils::get_all_fields(input);

    let variant = match (
        !query_builder_unique_fields.is_empty(),
        !query_builder_key_fields.is_empty(),
    ) {
        (true, true) => Variants::KeyFieldsAndUniqueFieldsExist,
        (false, true) => Variants::KeyFieldsExist,
        (true, false) => Variants::UniqueFieldsExist,
        _ => Variants::NeitherExist,
    };

    let all_fields_str = all_fields
        .iter()
        .map(|(field_name, _)| field_name.to_string())
        .collect::<Vec<_>>()
        .join(", ");

    let query_builder_unique_fetch = {
        let unique_query_builder_fields_where_clause = query_builder_unique_fields
            .iter()
            .enumerate()
            .map(|(index, (field_name, _))| {
                format!(
                    "({} = ${} OR ${} is null)",
                    field_name,
                    index + 1,
                    index + 1
                )
            })
            .collect::<Vec<_>>()
            .join(" AND ");

        let full_where_clause = match variant {
            Variants::KeyFieldsAndUniqueFieldsExist | Variants::UniqueFieldsExist => {
                format!("WHERE {unique_query_builder_fields_where_clause}")
            }
            _ => String::new(),
        };

        let query = format!("SELECT {all_fields_str} FROM {table_name} {full_where_clause}");

        let query_args = query_builder_unique_fields
            .iter()
            .map(|(field_name, _field_type)| {
                quote! {
                    self.#field_name,
                }
            });

        let res = quote! {
            pub async fn fetch_one<'e, E: sqlx::PgExecutor<'e>>(
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


        };
        res
    };

    let query_builder_key_fetch = {
        let key_where_clause = {
            let key_query_builder_fields_where_clause_match = query_builder_key_fields
                .iter()
                .enumerate()
                .map(|(index, (field_name, _))| format!("{} = ${}", field_name, index + 1))
                .collect::<Vec<_>>()
                .join(" AND ");

            let key_query_builder_fields_where_clause_all_null = query_builder_key_fields
                .iter()
                .enumerate()
                .map(|(index, (_, _))| format!("${} is null", index + 1))
                .collect::<Vec<_>>()
                .join(" AND ");

            format!(
                "({} OR {})",
                key_query_builder_fields_where_clause_match,
                key_query_builder_fields_where_clause_all_null
            )
        };
        let full_where_clause = match variant {
            Variants::KeyFieldsAndUniqueFieldsExist | Variants::KeyFieldsExist => {
                format!("WHERE {key_where_clause}")
            }
            _ => String::new(),
        };

        let query = format!("SELECT {all_fields_str} FROM {table_name} {full_where_clause}");

        let query_args = query_builder_key_fields
            .iter()
            .map(|(field_name, _field_type)| {
                quote! {
                    self.#field_name,
                }
            });

        let res = quote! {
            pub async fn fetch_one<'e, E: sqlx::PgExecutor<'e>>(
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


        };
        res
    };

    let generics = match variant {
        Variants::UniqueFieldsExist => quote! { <UniqueFields = NotSet> },
        Variants::KeyFieldsExist => quote! {<KeyFields = NotSet> },
        Variants::KeyFieldsAndUniqueFieldsExist => {
            quote! { <KeyFields = NotSet, UniqueFields = NotSet> }
        }
        Variants::NeitherExist => quote! {},
    };

    let generics_initial = match variant {
        Variants::UniqueFieldsExist => quote! { <NotSet> },
        Variants::KeyFieldsExist => quote! {<NotSet> },
        Variants::KeyFieldsAndUniqueFieldsExist => {
            quote! { <NotSet, NotSet> }
        }
        Variants::NeitherExist => quote! {},
    };

    let key_phantom_field = match variant {
        Variants::KeyFieldsAndUniqueFieldsExist | Variants::KeyFieldsExist => {
            quote! { , _key_fields: std::marker::PhantomData::<KeyFields>}
        }
        _ => quote! {},
    };

    let unique_phantom_field = match variant {
        Variants::KeyFieldsAndUniqueFieldsExist | Variants::UniqueFieldsExist => {
            quote! { , _unique_fields: std::marker::PhantomData::<UniqueFields> }
        }
        _ => quote! {},
    };

    let intial_key_phantom_field = match variant {
        Variants::KeyFieldsAndUniqueFieldsExist | Variants::KeyFieldsExist => {
            quote! {, _key_fields: std::marker::PhantomData::<NotSet>}
        }
        _ => quote! {},
    };

    let intial_unique_phantom_field = match variant {
        Variants::KeyFieldsAndUniqueFieldsExist | Variants::UniqueFieldsExist => {
            quote! {, _unique_fields: std::marker::PhantomData::<NotSet>}
        }
        _ => quote! {},
    };

    let unique_set_impl = if !query_builder_unique_fields.is_empty() {
        let unique_set_generic = if !query_builder_key_fields.is_empty() {
            quote! {
                <NotSet,Set>
            }
        } else {
            quote! {
                <Set>
            }
        };

        quote! {
        impl #query_builder_struct_name #unique_set_generic {
            #query_builder_unique_fetch
        }}
    } else {
        quote! {}
    };
    let key_set_impl = if !query_builder_key_fields.is_empty() {
        let key_set_generic = if !query_builder_unique_fields.is_empty() {
            quote! {
                <Set,NotSet>
            }
        } else {
            quote! {
                <Set>
            }
        };
        quote! {

        impl #query_builder_struct_name #key_set_generic {
            #query_builder_key_fetch
        }
        }
    } else {
        quote! {}
    };

    quote! {
        pub struct NotSet;
        pub struct Set;

        #[derive(Debug)]
        pub struct #query_builder_struct_name #generics  {
            #(#query_builder_struct_fields),*
            #key_phantom_field
            #unique_phantom_field
        }

        impl #query_builder_struct_name {
            pub fn new() -> #query_builder_struct_name #generics_initial {
            Self {
                #(#query_builder_struct_fields_initial),*
                #intial_key_phantom_field
                #intial_unique_phantom_field
                }
            }
        }
        impl #query_builder_struct_name #generics_initial {
            #(#query_builder_unique_methods)*
            #(#query_builder_key_methods )*
        }

        #unique_set_impl

        #key_set_impl
    }
}
