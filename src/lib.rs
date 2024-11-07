use convert_case::{Case, Casing};
use modules::{delete_query_builder, update_query_builder};
use modules::{insert_query_builder, one_query_builder};
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

mod common;
mod modules;
use modules::dbset;
use modules::from_row;
use modules::many_query_builder;

use common::utils::{self};

#[proc_macro_derive(DbSet, attributes(unique, dbset, relation, auto, key))]
pub fn dbset_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let dbset_name = utils::get_dbset_name(&input);

    let many_query_builder_impl = many_query_builder::get_query_builder(&input);
    let one_query_builder_impl = one_query_builder::get_query_builder(&input);
    let insert_builder_impl = insert_query_builder::get_insert_query_builder(&input);
    let update_builder_impl = update_query_builder::get_update_query_builder(&input);
    let delete_builder_impl = delete_query_builder::get_query_builder(&input);

    // println!("{}", pretty_print_tokenstream(insert_builder_impl.clone()));

    let from_row_impl = from_row::get_from_row_impl(&input);
    let dbset_impl = dbset::get_dbset_impl(&input);

    let module_name = quote::format_ident!(
        "{}_module",
        dbset_name
            .to_string()
            .from_case(Case::Pascal)
            .to_case(Case::Snake)
    );

    let expanded = quote! {

        mod #module_name {
            use super::User;

            pub struct Set;
            pub struct NotSet;

            #from_row_impl
            #many_query_builder_impl
            #one_query_builder_impl
            #insert_builder_impl
            #update_builder_impl
            #delete_builder_impl
            #dbset_impl
        }

        pub use #module_name::#dbset_name;
    };

    TokenStream::from(expanded)
}
