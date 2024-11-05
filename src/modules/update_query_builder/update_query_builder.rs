use proc_macro2::Ident;
use quote::quote;
use syn::{DeriveInput, Type};

use crate::common::utils::{
    get_all_fields, get_auto_fields, get_dbset_name, get_inner_option_type, get_key_fields,
    get_struct_name, get_table_name,
};
pub fn get_update_builder_struct_name(input: &DeriveInput) -> Ident {
    let dbset_name = get_dbset_name(input);
    quote::format_ident!("{}UpdateBuilder", dbset_name)
}

pub fn get_update_query_builder(input: &DeriveInput) -> proc_macro2::TokenStream {
    let table_name = get_table_name(input);
    let struct_name = get_struct_name(input);
    let builder_struct_name = get_update_builder_struct_name(input);
    let all_fields = get_all_fields(input);
    let auto_fields = get_auto_fields(input);

    let is_not_auto_field = |(field, _): &(&proc_macro2::Ident, &Type)| {
        !auto_fields
            .iter()
            .any(|(auto_field, _)| auto_field == field)
    };

    let all_update_fields = all_fields.iter().filter(|&x| is_not_auto_field(x));

    // Create Builder Struct
    let builder_struct = quote! {
        pub struct #builder_struct_name <updatable = NotSet> {
             updatable: #struct_name,
             _updatable: std::marker::PhantomData::<updatable>,
        }
    };

    let new_impl = quote! {
            pub fn new() -> #builder_struct_name <NotSet>  {
                Self {
                    updatable: None,
                     _updatable: std::marker::PhantomData::<updatable>,
                }
            }
    };

    let builder_method = quote! {
        impl #builder_struct_name <NotSet> {
                pub fn data(self, value: #struct_name) -> #builder_struct_name <Set>  {
                    #builder_struct_name  {
                        updatable: Some(value),
                         _updatable: std::marker::PhantomData::<Set>,

                    }
                }

    }
    };

    let key_fields = get_key_fields(input);
    // Create complete impl
    let where_size = key_fields.len();
    let query_builder_where_fields = key_fields
        .iter()
        .enumerate()
        .map(|(index, (field_name, _))| format!("{} = ${}", field_name, index + 1))
        .collect::<Vec<_>>()
        .join(" AND ");

    let set_fields = all_fields
        .iter()
        .filter(|(ident, _)| !key_fields.iter().any(|(kf_ident, _)| kf_ident == ident))
        .enumerate()
        .map(|(index, (field_name, _))| format!("{} = ${}", field_name, index + where_size + 1))
        .collect::<Vec<_>>()
        .join(", ");

    let query = format!(
        "UPDATE {table_name} SET {set_fields} WHERE {query_builder_where_fields} RETURNING *;"
    );

    let query_args = all_update_fields.clone().map(|(name, _)| {
        quote! { self.updatable.#name, }
    });

    let update_method = quote! {
        impl  #builder_struct_name <Set> {
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

    let builder_struct_impl = quote! {
        #builder_struct

        impl #builder_struct_name {
            #new_impl
        }

        #builder_method

        #update_method
    };

    builder_struct_impl
}