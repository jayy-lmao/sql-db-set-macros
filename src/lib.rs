use modules::one_query_builder;
use proc_macro::TokenStream;
use quote::quote;
use syn::File;
use syn::{parse2, Item};
use syn::{parse_macro_input, DeriveInput};

mod common;
mod modules;
use modules::dbset;
use modules::from_row;
use modules::many_query_builder;

use common::utils::{self, pretty_print_tokenstream};

#[proc_macro_derive(DbSet, attributes(unique, dbset, relation, auto, key))]
pub fn dbset_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let dbset_name = utils::get_dbset_name(&input);

    let many_query_builder_impl = many_query_builder::get_query_builder(&input);
    let one_query_builder_impl = one_query_builder::get_query_builder(&input);

    // println!("{one_query_builder_impl}");
    //pretty_print_tokenstream(one_query_builder_impl);
    let from_row_impl = from_row::get_from_row_impl(&input);
    let dbset_impl = dbset::get_dbset_impl(&input);

    let module_name = quote::format_ident!("{}_module", dbset_name);

    let expanded = quote! {

        mod #module_name {
            use super::User;
            #from_row_impl
            #many_query_builder_impl
            #one_query_builder_impl
            #dbset_impl
        }

        pub use #module_name::#dbset_name;
    };

    TokenStream::from(expanded)
}
