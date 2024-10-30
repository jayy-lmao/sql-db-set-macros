use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

mod modules;
use modules::dbset;
use modules::from_row;
use modules::many_query_builder;

mod utils;

#[proc_macro_derive(DbSet, attributes(unique, dbset, relation, auto, key))]
pub fn dbset_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let query_builder_impl = many_query_builder::get_query_builder(&input);
    let from_row_impl = from_row::get_from_row_impl(&input);
    let dbset_impl = dbset::get_dbset_impl(&input);

    println!("EXPANDED:\n{dbset_impl}");

    let expanded = quote! {
        #from_row_impl
        #query_builder_impl
        #dbset_impl
    };

    TokenStream::from(expanded)
}
