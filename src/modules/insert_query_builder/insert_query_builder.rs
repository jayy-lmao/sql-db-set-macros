use proc_macro2::Ident;
use quote::quote;
use syn::{Attribute, DeriveInput, Type};

use crate::common::utils::{
    get_all_fields, get_auto_fields, get_dbset_name, get_inner_option_type,
    get_query_fields_string, get_struct_name, get_table_name, is_custom_enum_attr,
};
use crate::modules::query_builder_shared as shared;

pub fn get_insert_builder_struct_name(input: &DeriveInput) -> Ident {
    let dbset_name = get_dbset_name(input);
    quote::format_ident!("{}InsertBuilder", dbset_name)
}

pub fn get_insert_query_builder(input: &DeriveInput) -> proc_macro2::TokenStream {
    let table_name = get_table_name(input);
    let struct_name = get_struct_name(input);
    let builder_struct_name = get_insert_builder_struct_name(input);
    let all_fields = get_all_fields(input);
    let auto_fields = get_auto_fields(input);
    let all_fields_str = get_query_fields_string(input);

    let insert_fields = shared::filter_non_auto_fields(&all_fields, &auto_fields);
    let required_fields =
        shared::get_required_fields(&all_fields, &auto_fields, &get_inner_option_type);

    let builder_struct_generics = shared::build_builder_struct_generics(&required_fields);
    let struct_fields = shared::build_struct_fields(&insert_fields, &get_inner_option_type);
    let phantom_struct_fields = shared::build_phantom_struct_fields(&required_fields);
    let initial_generics = shared::build_initial_generics(&required_fields);
    let initial_struct_fields = shared::build_initial_struct_fields(&insert_fields);
    let initial_phantom_struct_fields =
        shared::build_initial_phantom_struct_fields(&required_fields);
    let builder_methods = shared::build_builder_methods(
        &insert_fields,
        &required_fields,
        &builder_struct_name,
        &get_inner_option_type,
    );
    let (query, query_args) =
        shared::build_insert_query_and_args(&table_name, &insert_fields, &all_fields_str);
    let insert_method_generics = required_fields.iter().map(|_| quote! { Set, });

    let builder_struct = quote! {
        pub struct #builder_struct_name <#(#builder_struct_generics)*> {
            #(#struct_fields)*
            #(#phantom_struct_fields)*
        }
    };
    let new_impl = quote! {
        pub fn new() -> #builder_struct_name <#(#initial_generics)*>  {
            Self {
                #(#initial_struct_fields)*
                #(#initial_phantom_struct_fields)*
            }
        }
    };
    let insert_method = quote! {
        impl  #builder_struct_name <#(#insert_method_generics)*> {
            pub async fn insert<'e, E: sqlx::PgExecutor<'e>>(
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
    quote! {
        #builder_struct
        impl #builder_struct_name {
            #new_impl
        }
        #(#builder_methods)*
        #insert_method
    }
}
