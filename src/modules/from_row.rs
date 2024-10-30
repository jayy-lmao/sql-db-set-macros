use quote::quote;
use syn::DeriveInput;

use crate::utils;

pub fn get_from_row_impl(input: &DeriveInput) -> proc_macro2::TokenStream {
    let struct_name = utils::get_struct_name(input);
    let field_names = utils::get_field_names(input);

    let from_row_field_initializers = field_names.iter().map(|field_name| {
        let field_name_str = format!("{field_name}");
        quote! {
            #field_name: sqlx::Row::try_get(row, #field_name_str)?,
        }
    });

    let from_row_impl = quote! {
            impl<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> for #struct_name {
                fn from_row(row: &'r sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
                    Ok(#struct_name {
                        #(#from_row_field_initializers)*
                    })
                }
            }
    };
    from_row_impl
}
