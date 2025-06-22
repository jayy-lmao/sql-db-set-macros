use convert_case::{Case, Casing};
use proc_macro2::Ident;
use quote::quote;
use syn::{Attribute, DeriveInput, Type};

use crate::common::utils::{
    get_all_fields, get_auto_fields, get_dbset_name, get_inner_option_type,
    get_query_fields_string, get_struct_name, get_table_name, is_custom_enum_attr,
};

// Helper: filter out auto fields
fn filter_insertable_fields<'a>(
    all_fields: &'a [(&Ident, &Type, &Vec<Attribute>)],
    auto_fields: &[(&Ident, &Type, &Vec<Attribute>)]
) -> Vec<&'a (&'a Ident, &'a Type, &'a Vec<Attribute>)> {
    all_fields.iter().filter(|(field, _, _)| {
        !auto_fields.iter().any(|(auto_field, _, _)| auto_field == field)
    }).collect()
}

// Helper: get required (non-Option, non-auto) fields
fn get_required_insert_fields<'a>(
    all_fields: &'a [(&'a Ident, &'a Type, &'a Vec<Attribute>)],
    auto_fields: &[(&'a Ident, &'a Type, &'a Vec<Attribute>)]
) -> Vec<&'a (&'a Ident, &'a Type, &'a Vec<Attribute>)> {
    all_fields.iter().filter(|(_, ty, _)| get_inner_option_type(ty).is_none())
        .filter(|(field, _, _)| !auto_fields.iter().any(|(auto_field, _, _)| auto_field == field))
        .collect()
}

// Helper: build builder struct generics
fn build_builder_struct_generics<'a>(required_fields: &[&'a (&'a Ident, &'a Type, &'a Vec<Attribute>)]) -> Vec<proc_macro2::TokenStream> {
    required_fields.iter().map(|(field_name, _, _)| {
        let gen_name_pascal = quote::format_ident!(
            "{}",
            field_name.to_string().from_case(Case::Snake).to_case(Case::Pascal)
        );
        quote! { #gen_name_pascal = NotSet, }
    }).collect()
}

// Helper: build struct fields
fn build_struct_fields<'a>(insert_fields: &[&'a (&'a Ident, &'a Type, &'a Vec<Attribute>)]) -> Vec<proc_macro2::TokenStream> {
    insert_fields.iter().map(|(name, ty, _)| {
        let inner_field_type = get_inner_option_type(ty);
        let type_arg = match inner_field_type { Some(inner) => inner, None => ty };
        quote! { #name: Option<#type_arg>, }
    }).collect()
}

// Helper: build phantom fields
fn build_phantom_struct_fields<'a>(required_fields: &[&'a (&'a Ident, &'a Type, &'a Vec<Attribute>)]) -> Vec<proc_macro2::TokenStream> {
    required_fields.iter().map(|(name, _, _)| {
        let gen_name_pascal = quote::format_ident!(
            "{}",
            name.to_string().from_case(Case::Snake).to_case(Case::Pascal)
        );
        let ph_name = quote::format_ident!("_{}", name);
        quote! { #ph_name: std::marker::PhantomData::<#gen_name_pascal>, }
    }).collect()
}

// Helper: build initial generics
fn build_initial_generics(required_fields: &[&(&Ident, &Type, &Vec<Attribute>)]) -> Vec<proc_macro2::TokenStream> {
    required_fields.iter().map(|_| quote! { NotSet, }).collect()
}

// Helper: build initial struct fields
fn build_initial_struct_fields(insert_fields: &[&(&Ident, &Type, &Vec<Attribute>)]) -> Vec<proc_macro2::TokenStream> {
    insert_fields.iter().map(|(name, _, _)| quote! { #name: None, }).collect()
}

// Helper: build initial phantom struct fields
fn build_initial_phantom_struct_fields(required_fields: &[&(&Ident, &Type, &Vec<Attribute>)]) -> Vec<proc_macro2::TokenStream> {
    required_fields.iter().map(|(name, _, _)| {
        let ph_name = quote::format_ident!("_{}", name);
        quote! { #ph_name: std::marker::PhantomData::<NotSet>, }
    }).collect()
}

// Helper: build builder methods
fn build_builder_methods(
    insert_fields: &[&(&Ident, &Type, &Vec<Attribute>)],
    required_fields: &[&(&Ident, &Type, &Vec<Attribute>)],
    builder_struct_name: &Ident
) -> Vec<proc_macro2::TokenStream> {
    insert_fields.iter().map(|(field_name, field_type, _)| {
        let method_name = quote::format_ident!("{}", field_name);
        let ph_name = quote::format_ident!("_{}", field_name);
        let inner_field_type = get_inner_option_type(field_type);
        let ph_field = if inner_field_type.is_none() {
            quote! { #ph_name: std::marker::PhantomData::<Set>, }
        } else {
            quote!{}
        };
        let type_arg = match inner_field_type { Some(inner) => inner, None => field_type };
        // Generics in/out
        let pre_impl_generics_in = required_fields.iter().map(|(gen_name, _, _)| {
            if gen_name.to_string() != field_name.to_string() {
                let gen_name_pascal = quote::format_ident!("{}", gen_name.to_string().from_case(Case::Snake).to_case(Case::Pascal));
                quote! { #gen_name_pascal, }
            } else { quote!{} }
        });
        let generics_in = required_fields.iter().map(|(gen_name, _, _)| {
            if gen_name.to_string() != field_name.to_string() {
                let gen_name_pascal = quote::format_ident!("{}", gen_name.to_string().from_case(Case::Snake).to_case(Case::Pascal));
                quote! { #gen_name_pascal, }
            } else { quote!{ NotSet, } }
        });
        let generics_out = required_fields.iter().map(|(gen_name, _, _)| {
            if gen_name.to_string() != field_name.to_string() {
                let gen_name_pascal = quote::format_ident!("{}", gen_name.to_string().from_case(Case::Snake).to_case(Case::Pascal));
                quote! { #gen_name_pascal, }
            } else { quote!{ Set, } }
        });
        // Fill other fields
        let fill_other_fields = insert_fields.iter().filter(|(other_field_name, _, _)| other_field_name.to_string() != field_name.to_string())
            .map(|(name, _, _)| quote! { #name: self.#name, });
        let fill_other_phantom_fields = required_fields.iter().filter(|(other_field_name, _, _)| other_field_name.to_string() != field_name.to_string())
            .map(|(name, _, _)| {
                let ph_name = quote::format_ident!("_{}", name);
                quote! { #ph_name: self.#ph_name, }
            });
        quote! {
            impl <#(#pre_impl_generics_in)*> #builder_struct_name <#(#generics_in)*> {
                pub fn #method_name(self, #field_name: #type_arg) -> #builder_struct_name <#(#generics_out)*>  {
                    #builder_struct_name  {
                        #field_name: Some(#field_name),
                        #(#fill_other_fields)*
                        #ph_field
                        #(#fill_other_phantom_fields)*
                    }
                }
            }
        }
    }).collect()
}

// Helper: build query and args
fn build_insert_query_and_args(
    table_name: &str,
    insert_fields: &[&(&Ident, &Type, &Vec<Attribute>)],
    all_fields_str: &str
) -> (String, Vec<proc_macro2::TokenStream>) {
    let all_insert_fields_str = insert_fields.iter().map(|(name, _, _)| name.to_string()).collect::<Vec<_>>().join(", ");
    let all_params = insert_fields.iter().enumerate().map(|(index, _)| format!("${}", index + 1)).collect::<Vec<_>>().join(", ");
    let query = format!("INSERT INTO {table_name}({all_insert_fields_str}) VALUES ({all_params}) RETURNING {all_fields_str};");
    let query_args = insert_fields.iter().map(|(name, ty, attrs)| {
        let is_custom_enum = attrs.iter().any(is_custom_enum_attr);
        if is_custom_enum {
            quote! { self.#name as Option<#ty>, }
        } else {
            quote! { self.#name, }
        }
    }).collect();
    (query, query_args)
}

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
    let all_fields_str = get_query_fields_string(input);

    let insert_fields = filter_insertable_fields(&all_fields, &auto_fields);
    let required_fields = get_required_insert_fields(&all_fields, &auto_fields);

    let builder_struct_generics = build_builder_struct_generics(&required_fields);
    let struct_fields = build_struct_fields(&insert_fields);
    let phantom_struct_fields = build_phantom_struct_fields(&required_fields);
    let initial_generics = build_initial_generics(&required_fields);
    let initial_struct_fields = build_initial_struct_fields(&insert_fields);
    let initial_phantom_struct_fields = build_initial_phantom_struct_fields(&required_fields);
    let builder_methods = build_builder_methods(&insert_fields, &required_fields, &builder_struct_name);
    let (query, query_args) = build_insert_query_and_args(&table_name, &insert_fields, &all_fields_str);
    let insert_method_generics = required_fields.iter().map(|_| quote! { Set, });

    let builder_struct = quote! {
        pub struct #builder_struct_name <#(#builder_struct_generics)*> {
            #(#struct_fields)*
            #(#phantom_struct_fields)*
        }
    };
    let new_impl = quote! {
        pub fn new() -> #builder_struct_name <#(#initial_generics)*>  {
            Self {
                #(#initial_struct_fields)*
                #(#initial_phantom_struct_fields)*
            }
        }
    };
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
    quote! {
        #builder_struct
        impl #builder_struct_name {
            #new_impl
        }
        #(#builder_methods)*
        #insert_method
    }
}
