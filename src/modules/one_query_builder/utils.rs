use proc_macro2::Ident;
use quote::quote;
use syn::{DeriveInput, Type};

use crate::{
    common::utils::{get_key_fields, get_unique_fields},
    utils::{get_dbset_name, get_fields, get_inner_option_type, is_key_attr, is_unique_attr},
};

use super::one_query_builder::Variants;

pub fn get_one_query_builder_struct_name(input: &DeriveInput) -> Ident {
    let dbset_name = get_dbset_name(input);
    quote::format_ident!("{}OneQueryBuilder", dbset_name)
}

pub fn get_one_query_builder_unique_fields(input: &DeriveInput) -> Vec<(&Ident, &Type)> {
    let fields = get_fields(input);

    let mut query_builder_unique_fields = Vec::new();
    for field in fields {
        let field_name = field.ident.as_ref().expect("could not cast ident as ref");
        let field_type = &field.ty;
        let is_unique = field.attrs.iter().any(is_unique_attr);
        let is_key = field.attrs.iter().any(is_key_attr);

        if is_unique && !is_key {
            query_builder_unique_fields.push((field_name, field_type));
        }
    }
    query_builder_unique_fields
}

pub fn get_one_query_builder_key_fields(input: &DeriveInput) -> Vec<(&Ident, &Type)> {
    let fields = get_fields(input);

    let mut query_builder_key_fields = Vec::new();

    for field in fields {
        let field_name = field.ident.as_ref().expect("could not cast ident as ref");
        let field_type = &field.ty;
        let is_key = field.attrs.iter().any(is_key_attr);

        if is_key {
            query_builder_key_fields.push((field_name, field_type));
        }
    }
    query_builder_key_fields
}

pub fn get_one_query_builder_struct_fieldsl(input: &DeriveInput) -> Vec<proc_macro2::TokenStream> {
    let fields = get_fields(input);

    let mut query_builder_struct_fields = Vec::new();

    for field in fields {
        let field_name = field.ident.as_ref().expect("could not cast ident as ref");
        let field_type = &field.ty;
        let is_unique = field.attrs.iter().any(is_unique_attr);
        let is_key = field.attrs.iter().any(is_key_attr);

        if is_unique || is_key {
            let inner_type = get_inner_option_type(field_type);
            if let Some(inner_type) = inner_type {
                query_builder_struct_fields.push(quote! { #field_name: Option<#inner_type> });
            } else {
                query_builder_struct_fields.push(quote! { #field_name: Option<#field_type> });
            }
        }
    }
    query_builder_struct_fields
}

pub fn get_one_query_builder_struct_fields_initial(
    input: &DeriveInput,
) -> Vec<proc_macro2::TokenStream> {
    let fields = get_fields(input);
    let mut query_builder_struct_fields_initial = Vec::new();

    for field in fields {
        let field_name = field.ident.as_ref().expect("could not cast ident as ref");
        let is_unique = field.attrs.iter().any(is_unique_attr);
        let is_key = field.attrs.iter().any(is_key_attr);

        if is_unique || is_key {
            query_builder_struct_fields_initial.push(quote! { #field_name: None });
        }
    }
    query_builder_struct_fields_initial
}

pub fn get_one_query_builder_unique_methods(input: &DeriveInput) -> Vec<proc_macro2::TokenStream> {
    let fields = get_fields(input);
    let key_fields = get_key_fields(input);
    let unique_fields = get_unique_fields(input);

    let mut query_builder_methods = Vec::new();
    let query_builder_struct_name = get_one_query_builder_struct_name(input);

    let variant = match (!unique_fields.is_empty(), !key_fields.is_empty()) {
        (true, true) => Variants::KeyFieldsAndUniqueFieldsExist,
        (false, true) => Variants::KeyFieldsExist,
        (true, false) => Variants::UniqueFieldsExist,
        _ => Variants::NeitherExist,
    };

    let generic_key_set = match variant {
        Variants::UniqueFieldsExist => quote! {},
        Variants::KeyFieldsExist => quote! { ::<Set> },
        Variants::KeyFieldsAndUniqueFieldsExist => {
            quote! { ::<NotSet, Set> }
        }
        Variants::NeitherExist => quote! {},
    };

    for field in fields {
        let field_name = field.ident.as_ref().expect("could not cast ident as ref");
        let field_type = &field.ty;
        let is_unique = field.attrs.iter().any(is_unique_attr);
        let is_key = field.attrs.iter().any(is_key_attr);

        let key_struct_fields = key_fields
            .iter()
            .filter(|(key_field_name, _)| *key_field_name != field_name)
            .map(|(field_name, _)| {
                quote! {
                    #field_name: self.#field_name,
                }
            });
        let unique_struct_fields = unique_fields
            .iter()
            .filter(|(unique_field_name, _)| *unique_field_name != field_name)
            .map(|(field_name, _)| {
                quote! {
                    #field_name: self.#field_name,
                }
            });

        if is_unique && !is_key {
            let method_name = quote::format_ident!("{}_eq", field_name);

            query_builder_methods.push(quote! {
                        pub fn #method_name(mut self, value: #field_type) -> #query_builder_struct_name<NotSet,Set> {
                            #query_builder_struct_name #generic_key_set {
                                #field_name: Some(value),
                                _key_fields: std::marker::PhantomData::<NotSet>,
                                _unique_fields: std::marker::PhantomData::<Set>,
                                #(#key_struct_fields)*
                                #(#unique_struct_fields)*
                            }

                        }
                });
        }
    }
    query_builder_methods
}

pub fn get_one_query_builder_key_methods(input: &DeriveInput) -> Vec<proc_macro2::TokenStream> {
    let fields = get_fields(input);
    let key_fields = get_key_fields(input);
    let unique_fields = get_unique_fields(input);

    let variant = match (!unique_fields.is_empty(), !key_fields.is_empty()) {
        (true, true) => Variants::KeyFieldsAndUniqueFieldsExist,
        (false, true) => Variants::KeyFieldsExist,
        (true, false) => Variants::UniqueFieldsExist,
        _ => Variants::NeitherExist,
    };

    let mut query_builder_methods = Vec::new();
    let query_builder_struct_name = get_one_query_builder_struct_name(input);

    let generic_key_set = match variant {
        Variants::UniqueFieldsExist => quote! {},
        Variants::KeyFieldsExist => quote! {::<Set>},
        Variants::KeyFieldsAndUniqueFieldsExist => {
            quote! {  ::<Set, NotSet> }
        }
        Variants::NeitherExist => quote! {},
    };

    for field in fields {
        let field_name = field.ident.as_ref().expect("could not cast ident as ref");
        let field_type = &field.ty;
        let is_key = field.attrs.iter().any(is_key_attr);

        let key_struct_fields = key_fields
            .iter()
            .filter(|(key_field_name, _)| *key_field_name != field_name)
            .map(|(field_name, _)| {
                quote! {
                    #field_name: self.#field_name,
                }
            });
        let unique_struct_fields = unique_fields
            .iter()
            .filter(|(unique_field_name, _)| *unique_field_name != field_name)
            .map(|(field_name, _)| {
                quote! {
                    #field_name: self.#field_name,
                }
            });

        let unique_field_phantom_data = if !unique_fields.is_empty() {
            quote! {
                            _unique_fields: std::marker::PhantomData::<NotSet>,
            }
        } else {
            quote! {}
        };

        if is_key {
            let method_name = quote::format_ident!("{}_eq", field_name);
            query_builder_methods.push(quote! {
                    pub fn #method_name(self, value: #field_type) -> #query_builder_struct_name #generic_key_set {
                        #query_builder_struct_name #generic_key_set {
                            #field_name: Some(value),
                            _key_fields: std::marker::PhantomData::<Set>,
                            #unique_field_phantom_data
                            #(#key_struct_fields)*
                            #(#unique_struct_fields)*
                        }
                    }
            });
        }
    }
    query_builder_methods
}
