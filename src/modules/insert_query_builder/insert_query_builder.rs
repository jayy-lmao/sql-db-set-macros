use convert_case::{Case, Casing};
use proc_macro2::Ident;
use quote::{quote, ToTokens};
use syn::{Attribute, DeriveInput, Type};

use crate::common::utils::{
    get_all_fields, get_auto_fields, get_dbset_name, get_inner_option_type,
    get_query_fields_string, get_struct_name, get_table_name, is_custom_enum_attr,
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

    let is_not_auto_field = |(field, _, _): &(&proc_macro2::Ident, &Type, &Vec<Attribute>)| {
        !auto_fields
            .iter()
            .any(|(auto_field, _, _)| auto_field == field)
    };

    let non_nullable_fields = all_fields
        .iter()
        .filter(|(_, ty, _)| get_inner_option_type(ty).is_none());

    let all_required_insert_fields = non_nullable_fields.filter(|&x| is_not_auto_field(x));

    let all_insert_fields = all_fields.iter().filter(|&x| is_not_auto_field(x));

    // Get builder struct generics
    let builder_struct_generics = all_required_insert_fields
        .clone()
        .map(|(field_name, _, _)| {
            let gen_name_pascal = quote::format_ident!(
                "{}",
                field_name
                    .to_string()
                    .from_case(Case::Snake)
                    .to_case(Case::Pascal)
            );
            quote! {
                #gen_name_pascal = NotSet,
            }
        });

    let struct_fields = all_insert_fields
        .clone()
        .filter(|&x| is_not_auto_field(x))
        .map(|(name, ty, _)| {
            let inner_field_type = get_inner_option_type(ty);

            let type_arg = match inner_field_type {
                Some(inner) => inner,
                None => ty,
            };

            quote! { #name: Option< #type_arg >, }
        });

    let phantom_struct_fields = all_required_insert_fields.clone().map(|(name, _, _)| {
        let gen_name_pascal = quote::format_ident!(
            "{}",
            name.to_string()
                .from_case(Case::Snake)
                .to_case(Case::Pascal)
        );
        let ph_name = quote::format_ident!("_{}", name);
        quote! { #ph_name: std::marker::PhantomData::<#gen_name_pascal>, }
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

    let initial_struct_fields = all_insert_fields.clone().map(|(name, _, _)| {
        quote! { #name: None, }
    });

    let initial_phantom_struct_fields = all_required_insert_fields.clone().map(|(name, _, _)| {
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
        .map(|(name, _, _)| (name, quote! { #name: self.#name, }));

    let fill_other_phantom_fields = all_required_insert_fields.clone().map(|(name, _, _)| {
        let ph_name = quote::format_ident!("_{}", name);
        (name, quote! { #ph_name: self.#ph_name, })
    });

    let builder_methods = all_insert_fields.clone()
        .map(|(field_name, field_type,_)| {
            let method_name = quote::format_ident!("{}", field_name);
            let ph_name = quote::format_ident!("_{}", field_name);

            let inner_field_type= get_inner_option_type(field_type);

            let ph_field =
            if inner_field_type.is_none() {
                quote! { #ph_name: std::marker::PhantomData::<Set>, }
            } else {
                quote!{}
            };
            let remaining_fill = fill_other_fields
                .clone()
                .filter(|(other_field_name, _)| *other_field_name != field_name)
                .map(|(_, value)| value);

            let pre_impl_generics_in = all_required_insert_fields.clone().map(|(gen_name, _,_)|{
               if gen_name != field_name {
                    let gen_name_pascal = quote::format_ident!("{}", gen_name.to_string().from_case(Case::Snake).to_case(Case::Pascal));
                    return quote!{ #gen_name_pascal, }
                }
                quote!{}
            });

            let generics_in = all_required_insert_fields.clone().map(|(gen_name, _,_)|{
                if gen_name != field_name {
                    let gen_name_pascal = quote::format_ident!("{}", gen_name.to_string().from_case(Case::Snake).to_case(Case::Pascal));
                    return quote!{ #gen_name_pascal, }
                }
                quote!{ NotSet, }
            });
            let generics_out = all_required_insert_fields.clone().map(|(gen_name, _, _)|{
                if gen_name != field_name {
                    let gen_name_pascal = quote::format_ident!("{}", gen_name.to_string().from_case(Case::Snake).to_case(Case::Pascal));
                    return quote!{ #gen_name_pascal, }
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

    let insert_method_generics = all_required_insert_fields.clone().map(|(_, _, _)| {
        quote! { Set, }
    });

    // let all_fields_str = all_fields
    //     .iter()
    //     .map(|(field_name, _, _)| field_name.to_string())
    //     .collect::<Vec<_>>()
    //     .join(", ");
    let all_fields_str = get_query_fields_string(input);

    let all_insert_fields_str = all_insert_fields
        .clone()
        .map(|(name, _ty, _attrs)| name.to_string())
        .collect::<Vec<String>>()
        .join(", ");

    let all_params = all_insert_fields
        .clone()
        .enumerate()
        .map(|(index, _)| format!("${}", (index + 1)))
        .collect::<Vec<String>>()
        .join(", ");

    let query = format!("INSERT INTO {table_name}({all_insert_fields_str}) VALUES ({all_params}) RETURNING {all_fields_str};");

    let query_args = all_insert_fields.clone().map(|(name, ty, attrs)| {
        let is_custom_enum = attrs.iter().any(is_custom_enum_attr);
        if is_custom_enum {
            quote! { self.#name as Option<#ty>, }
        } else {
            quote! { self.#name, }
        }
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
