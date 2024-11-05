use proc_macro2::Ident;
use quote::quote;
use syn::{DeriveInput};

use crate::common::utils::{
    get_all_fields,  get_dbset_name, get_inner_option_type, get_key_fields, get_struct_name, get_table_name, get_unique_fields
};
pub fn get_one_builder_struct_name(input: &DeriveInput) -> Ident {
    let dbset_name = get_dbset_name(input);
    quote::format_ident!("{}OneQueryBuilder", dbset_name)
}

pub fn get_query_builder(input: &DeriveInput) -> proc_macro2::TokenStream {
    let table_name = get_table_name(input);
    let struct_name = get_struct_name(input);
    let builder_struct_name = get_one_builder_struct_name(input);
    let key_fields = get_key_fields(input);
    let unique_fields = get_unique_fields(input);
    let all_fields = get_all_fields(input);


    let non_nullable_fields = key_fields
        .iter()
        .filter(|(_, ty)| get_inner_option_type(ty).is_none());

    let all_required_insert_fields = non_nullable_fields;

    let all_query_one_fields = key_fields.iter().chain(unique_fields.iter());

    // Get builder struct generics
    let builder_struct_generics = all_required_insert_fields.clone().map(|(field_name, _)| {
        quote! {
            #field_name = NotSet,
        }
    }).chain(vec![
        quote! {
            UniqueFields = NotSet,
        }
    ]);

    let struct_fields = all_query_one_fields
        .clone()
        .map(|(name, ty)| {
            let inner_field_type = get_inner_option_type(ty);

            let type_arg = match inner_field_type {
                Some(inner) => inner,
                None => ty,
            };

            quote! { #name: Option< #type_arg >, }
        })
    .chain(vec![
        quote! {
            _unique_fields: std::marker::PhantomData::<UniqueFields>,
        }
    ]);


    let phantom_struct_fields = all_required_insert_fields.clone().map(|(name, _)| {
        let ph_name = quote::format_ident!("_{}", name);
        quote! { #ph_name: std::marker::PhantomData::<#name>, }
    });



    // Create Builder Struct
    let builder_struct = quote! {
        pub struct #builder_struct_name <#(#builder_struct_generics)*> {
            #(#struct_fields)*
            #(#phantom_struct_fields)*
        }
    };

    // Create new impl
    let initial_generics = all_required_insert_fields.clone().map(|_| {
        quote! {
            NotSet,
        }
    }).chain(vec![quote!{NotSet}]);

    let initial_struct_fields = all_query_one_fields.clone().map(|(name, _)| {
        quote! { #name: None, }
    }).chain(vec![quote!{
            _unique_fields: std::marker::PhantomData::<NotSet>,
    }]);

    let initial_phantom_struct_fields = all_required_insert_fields.clone().map(|(name, _)| {
        let ph_name = quote::format_ident!("_{}", name);
        quote! { #ph_name: std::marker::PhantomData::<NotSet>, }
    });

    let new_impl = quote! {
            pub fn new() -> #builder_struct_name <#(#initial_generics)*>  {
                Self {
                    #(#initial_struct_fields)*
                    #(#initial_phantom_struct_fields)*
                }
            }
    };

    // Create add value functions
    let fill_other_fields = all_query_one_fields
        .clone()
        .map(|(name, _)| (name, quote! { #name: self.#name, }));

    let fill_other_phantom_fields = all_required_insert_fields.clone().map(|(name, _)| {
        let ph_name = quote::format_ident!("_{}", name);
        (name, quote! { #ph_name: self.#ph_name, })
    });

    let builder_methods = all_query_one_fields.clone()
        .map(|(field_name, field_type)| {
            let is_unique_field = unique_fields.iter().any(|uf| uf.0 == *field_name);
            let method_name = quote::format_ident!("{}_eq", field_name);
            let ph_name = quote::format_ident!("_{}", field_name);

            let inner_field_type= get_inner_option_type(field_type);

            let ph_field =
            if inner_field_type.is_none() {
                if is_unique_field  {
                quote! { 
                    _unique_fields: std::marker::PhantomData::<Set>,
                } 

                } else {
                quote! { #ph_name: std::marker::PhantomData::<Set>, } 
                }
            } else { 
                quote! { }
            };
            
            let remaining_fill = fill_other_fields
                .clone()
                .filter(|(other_field_name, _)| *other_field_name != field_name)
                .map(|(_, value)| value);


            let pre_impl_generics_in = all_required_insert_fields.clone().map(|(gen_name, _)|{
                if gen_name != field_name {
                    return quote!{ #gen_name, }
                }
                quote!{  }
            });

            let generics_in = all_required_insert_fields.clone().map(|(gen_name, _)|{
                if gen_name != field_name {
                    return quote!{ #gen_name, }
                }
                quote!{ NotSet, }
            }).chain(vec![quote!{NotSet}]);
            
            let generics_out = all_required_insert_fields.clone().map(|(gen_name, _)|{
                if gen_name != field_name {
                    return quote!{ #gen_name, }
                }
                quote!{ Set, }
                }).chain( if is_unique_field {
                    vec![quote!{Set}]
                } else {
                    vec![quote!{NotSet}]
                });

            let remaining_phantom_fill = fill_other_phantom_fields
                .clone()
                .filter(|(other_field_name, _)| *other_field_name != field_name)
                .map(|(_, value)| value).chain( if is_unique_field {vec![]} else {vec![
                    quote!{ 
                _unique_fields: std::marker::PhantomData::<NotSet>,
                    }

                ]});

            let type_arg = match inner_field_type {
                Some(inner) => inner,
                None => field_type,
            };

            quote! {
                impl <#(#pre_impl_generics_in)*> #builder_struct_name <#(#generics_in)*> {
                        pub fn #method_name(self, #field_name: #type_arg) -> #builder_struct_name <#(#generics_out)*>  {
                            #builder_struct_name  {
                                #field_name: Some(#field_name),
                                #(#remaining_fill)*
                                #ph_field
                                #(#remaining_phantom_fill)*

                            }
                        }

            }
            }
        });

    // Create complete impl

    let key_fetch_one_method_generics = all_required_insert_fields.clone().map(|(_, _)| {
        quote! { Set, }
    }).chain(vec![ quote!{ NotSet }, ]);
    let unique_fetch_one_method_generics = all_required_insert_fields.clone().map(|(_, _)| {
        quote! { NotSet, }
    }).chain(vec![ quote!{ Set }, ]);

    let all_fields_str = all_fields
        .iter()
        .map(|(field_name, _)| field_name.to_string())
        .collect::<Vec<_>>()
        .join(", ");


        let key_query_builder_fields_where_clause = key_fields
            .iter()
            .enumerate()
            .map(|(index, (field_name, _))| {
                format!(
                    "{} = ${}",
                    field_name,
                    index + 1,
                )
            })
            .collect::<Vec<_>>()
            .join(" AND ");

        let unique_query_builder_fields_where_clause = unique_fields
            .iter()
            .enumerate()
            .map(|(index, (field_name, _))| {
                format!(
                    "({} = ${} OR ${} is null)",
                    field_name,
                    index + 1,
                    index + 1,
                )
            })
            .collect::<Vec<_>>()
            .join(" AND ");




        // let query = format!("INSERT INTO {table_name}({all_insert_fields_str}) VALUES ({all_params}) RETURNING {all_fields_str};");

    let unique_query_args = unique_fields.clone().into_iter().map(|(name, _)| {
        quote! { self.#name, }
    });

    let unique_fetch_one = if !unique_fields.is_empty() {
    let query = format!("SELECT {all_fields_str} FROM {table_name} WHERE {unique_query_builder_fields_where_clause}");

    quote! {

        impl  #builder_struct_name <#(#unique_fetch_one_method_generics)*> {
                pub async fn fetch_one<'e, E: sqlx::PgExecutor<'e>>(
                    self,
                    executor: E,
                ) -> Result<#struct_name, sqlx::Error> {
                    sqlx::query_as!(
                        #struct_name,
                        #query,
                        #(#unique_query_args)*
                    )
                        .fetch_one(executor)
                        .await
            }
        }
        }
    } else {quote!{}};

    let key_query_args = key_fields.clone().into_iter().map(|(name, _)| {
        quote! { self.#name, }
    });

    let key_fetch_one = if !key_fields.is_empty() {
    let query = format!("SELECT {all_fields_str} FROM {table_name} WHERE {key_query_builder_fields_where_clause}");

        quote! {
        impl  #builder_struct_name <#(#key_fetch_one_method_generics)*> {
                pub async fn fetch_one<'e, E: sqlx::PgExecutor<'e>>(
                    self,
                    executor: E,
                ) -> Result<#struct_name, sqlx::Error> {
                    sqlx::query_as!(
                        #struct_name,
                        #query,
                        #(#key_query_args)*
                    )
                        .fetch_one(executor)
                        .await
            }
        }
    }} else {quote!{}};

    let builder_struct_impl = quote! {
        #builder_struct

        impl #builder_struct_name {
            #new_impl
        }

        #(#builder_methods)*

        #key_fetch_one
        #unique_fetch_one
    };

    builder_struct_impl
}
