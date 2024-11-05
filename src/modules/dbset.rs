use quote::quote;
use syn::DeriveInput;

use crate::common::utils;
use crate::modules::insert_query_builder::get_insert_builder_struct_name;
use crate::modules::many_query_builder;
use crate::modules::one_query_builder::get_one_builder_struct_name;
use crate::modules::update_query_builder::get_update_builder_struct_name;
pub fn get_dbset_impl(input: &DeriveInput) -> proc_macro2::TokenStream {
    let many_query_builder_struct_name =
        many_query_builder::utils::get_many_query_builder_struct_name(input);
    let one_query_builder_struct_name = get_one_builder_struct_name(input);
    let insert_builder_struct_name = get_insert_builder_struct_name(input);
    let update_builder_struct_name = get_update_builder_struct_name(input);

    let dbset_name = utils::get_dbset_name(input);

    quote! {
        pub struct #dbset_name;

        impl #dbset_name {
            pub fn many() -> #many_query_builder_struct_name {
                #many_query_builder_struct_name::new()
            }
            pub fn one() -> #one_query_builder_struct_name {
                #one_query_builder_struct_name::new()
            }
            pub fn insert() -> #insert_builder_struct_name {
                #insert_builder_struct_name::new()
            }
            pub fn update() -> #update_builder_struct_name {
                #update_builder_struct_name::new()
            }
        }
    }
}
