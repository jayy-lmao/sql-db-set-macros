use proc_macro2::Ident;
use quote::quote;
use syn::{DeriveInput, Type};

use crate::utils::{
    get_dbset_name, get_fields, get_inner_option_type, is_key_attr, is_unique_attr,
};

pub fn get_many_query_builder_struct_name(input: &DeriveInput) -> Ident {
    let dbset_name = get_dbset_name(input);
    quote::format_ident!("{}ManyQueryBuilder", dbset_name)
}

pub fn get_many_query_builder_fields(input: &DeriveInput) -> Vec<(&Ident, &Type)> {
    let fields = get_fields(input);

    let mut query_builder_fields = Vec::new();

    for field in fields.iter() {
        let field_name = field.ident.as_ref().expect("could not cast ident as ref");
        let field_type = &field.ty;
        let is_unique = field.attrs.iter().any(is_unique_attr);
        let is_key = field.attrs.iter().any(is_key_attr);

        if !is_unique && !is_key {
            let inner_type = get_inner_option_type(field_type);

            if let Some(inner_type) = inner_type {
                query_builder_fields.push((field_name, inner_type));
            } else {
                query_builder_fields.push((field_name, field_type));
            }
        }
    }
    query_builder_fields
}

pub fn get_many_query_builder_struct_fieldsl(input: &DeriveInput) -> Vec<proc_macro2::TokenStream> {
    let fields = get_fields(input);

    let mut query_builder_struct_fields = Vec::new();

    for field in fields.iter() {
        let field_name = field.ident.as_ref().expect("could not cast ident as ref");
        let field_type = &field.ty;
        let is_unique = field.attrs.iter().any(is_unique_attr);
        let is_key = field.attrs.iter().any(is_key_attr);

        if !is_unique && !is_key {
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

pub fn get_many_query_builder_struct_fields_initial(
    input: &DeriveInput,
) -> Vec<proc_macro2::TokenStream> {
    let fields = get_fields(input);
    let mut query_builder_struct_fields_initial = Vec::new();

    for field in fields.iter() {
        let field_name = field.ident.as_ref().expect("could not cast ident as ref");
        let is_unique = field.attrs.iter().any(is_unique_attr);
        let is_key = field.attrs.iter().any(is_key_attr);

        if !is_unique && !is_key {
            query_builder_struct_fields_initial.push(quote! { #field_name: None });
        }
    }
    query_builder_struct_fields_initial
}

pub fn get_many_query_builder_methods(input: &DeriveInput) -> Vec<proc_macro2::TokenStream> {
    let fields = get_fields(input);
    let mut query_builder_methods = Vec::new();

    for field in fields.iter() {
        let field_name = field.ident.as_ref().expect("could not cast ident as ref");
        let field_type = &field.ty;
        let is_unique = field.attrs.iter().any(is_unique_attr);
        let is_key = field.attrs.iter().any(is_key_attr);

        if !is_unique && !is_key {
            let inner_type = get_inner_option_type(field_type);
            let method_name = quote::format_ident!("{}_eq", field_name);

            if let Some(inner_type) = inner_type {
                query_builder_methods.push(quote! {
                        pub fn #method_name(mut self, value: #inner_type) -> Self {
                            self.#field_name = Some(value);
                            self
                        }
                });
            } else {
                query_builder_methods.push(quote! {
                        pub fn #method_name(mut self, value: #field_type) -> Self {
                            self.#field_name = Some(value);
                            self
                        }
                });
            }
        }
    }
    query_builder_methods
}
