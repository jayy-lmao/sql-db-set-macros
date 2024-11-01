use quote::quote;
use syn::DeriveInput;

use crate::common::utils;
use crate::modules::{many_query_builder, one_query_builder};
pub fn get_dbset_impl(input: &DeriveInput) -> proc_macro2::TokenStream {
    let many_query_builder_struct_name =
        many_query_builder::utils::get_many_query_builder_struct_name(input);
    let one_query_builder_struct_name =
        one_query_builder::utils::get_one_query_builder_struct_name(input);
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
        }
    }
}
