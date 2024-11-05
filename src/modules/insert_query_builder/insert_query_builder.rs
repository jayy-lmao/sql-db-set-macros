use proc_macro2::Ident;
use quote::quote;
use syn::{DeriveInput, Type};

use crate::common::utils::{
    get_all_fields, get_auto_fields, get_dbset_name, get_inner_option_type, get_struct_name,
    get_table_name,
};
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

    let is_not_auto_field = |(field, _): &(&proc_macro2::Ident, &Type)| {
        !auto_fields
            .iter()
            .any(|(auto_field, _)| auto_field == field)
    };

    let non_nullable_fields = all_fields
        .iter()
        .filter(|(_, ty)| get_inner_option_type(ty).is_none());

    let all_required_insert_fields = non_nullable_fields.filter(|&x| is_not_auto_field(x));

    let all_insert_fields = all_fields.iter().filter(|&x| is_not_auto_field(x));

    // Get builder struct generics
    let builder_struct_generics = all_required_insert_fields.clone().map(|(field_name, _)| {
        quote! {
            #field_name = NotSet,
        }
    });

    let struct_fields = all_insert_fields
        .clone()
        .filter(|&x| is_not_auto_field(x))
        .map(|(name, ty)| {
            let inner_field_type = get_inner_option_type(ty);

            let type_arg = match inner_field_type {
                Some(inner) => inner,
                None => ty,
            };

            quote! { #name: Option< #type_arg >, }
        });

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
    });

    let initial_struct_fields = all_insert_fields.clone().map(|(name, _)| {
        quote! { #name: None, }
    });

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
    let fill_other_fields = all_insert_fields
        .clone()
        .map(|(name, _)| (name, quote! { #name: self.#name, }));

    let fill_other_phantom_fields = all_required_insert_fields.clone().map(|(name, _)| {
        let ph_name = quote::format_ident!("_{}", name);
        (name, quote! { #ph_name: self.#ph_name, })
    });

    let builder_methods = all_insert_fields.clone()
        .map(|(field_name, field_type)| {
            let method_name = quote::format_ident!("{}", field_name);
            let ph_name = quote::format_ident!("_{}", field_name);

            let inner_field_type= get_inner_option_type(field_type);

            let ph_field =
            if inner_field_type.is_none() {
                quote! { #ph_name: std::marker::PhantomData::<Set>, } 
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
            });
            
            let generics_out = all_required_insert_fields.clone().map(|(gen_name, _)|{
                if gen_name != field_name {
                    return quote!{ #gen_name, }
                }
                quote!{ Set, }
            });

            let remaining_phantom_fill = fill_other_phantom_fields
                .clone()
                .filter(|(other_field_name, _)| *other_field_name != field_name)
                .map(|(_, value)| value);

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

    let insert_method_generics = all_required_insert_fields.clone().map(|(_, _)| {
        quote! { Set, }
    });

    let all_fields_str = all_fields
        .iter()
        .map(|(field_name, _)| field_name.to_string())
        .collect::<Vec<_>>()
        .join(", ");

    let all_insert_fields_str = all_insert_fields
        .clone()
        .map(|(name, _)| name.to_string())
        .collect::<Vec<String>>()
        .join(", ");
    let all_params = all_insert_fields
        .clone()
        .enumerate()
        .map(|(index, _)| format!("${}", (index + 1)))
        .collect::<Vec<String>>()
        .join(", ");
    let query = format!("INSERT INTO {table_name}({all_insert_fields_str}) VALUES ({all_params}) RETURNING {all_fields_str};");

    let query_args = all_insert_fields.clone().map(|(name, _)| {
        quote! { self.#name, }
    });

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

    let builder_struct_impl = quote! {
        #builder_struct

        impl #builder_struct_name {
            #new_impl
        }

        #(#builder_methods)*

        #insert_method
    };

    builder_struct_impl
}
