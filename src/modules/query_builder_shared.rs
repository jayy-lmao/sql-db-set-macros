// Shared helpers for query builders (insert, update, delete, etc.)
// Move common logic here to reduce duplication.

use convert_case::{Case, Casing};
use proc_macro2::Ident;
use quote::quote;
use syn::{Attribute, Type};

// Filter out auto fields from all fields
pub fn filter_non_auto_fields<'a>(
    all_fields: &'a [(&Ident, &Type, &Vec<Attribute>)],
    auto_fields: &[(&Ident, &Type, &Vec<Attribute>)]
) -> Vec<&'a (&'a Ident, &'a Type, &'a Vec<Attribute>)> {
    all_fields.iter().filter(|(field, _, _)| {
        !auto_fields.iter().any(|(auto_field, _, _)| auto_field == field)
    }).collect()
}

// Get required (non-Option, non-auto) fields
pub fn get_required_fields<'a>(
    all_fields: &'a [(&'a Ident, &'a Type, &'a Vec<Attribute>)],
    auto_fields: &[(&'a Ident, &'a Type, &'a Vec<Attribute>)],
    get_inner_option_type: &dyn Fn(&Type) -> Option<&Type>,
) -> Vec<&'a (&'a Ident, &'a Type, &'a Vec<Attribute>)> {
    all_fields.iter().filter(|(_, ty, _)| get_inner_option_type(ty).is_none())
        .filter(|(field, _, _)| !auto_fields.iter().any(|(auto_field, _, _)| auto_field == field))
        .collect()
}

// Build builder struct generics
pub fn build_builder_struct_generics<'a>(required_fields: &[&'a (&'a Ident, &'a Type, &'a Vec<Attribute>)]) -> Vec<proc_macro2::TokenStream> {
    required_fields.iter().map(|(field_name, _, _)| {
        let gen_name_pascal = quote::format_ident!(
            "{}",
            field_name.to_string().from_case(Case::Snake).to_case(Case::Pascal)
        );
        quote! { #gen_name_pascal = NotSet, }
    }).collect()
}

// Build struct fields (Option<T> for all fields)
pub fn build_struct_fields<'a>(
    insert_fields: &[&'a (&'a Ident, &'a Type, &'a Vec<Attribute>)],
    get_inner_option_type: &dyn Fn(&Type) -> Option<&Type>,
) -> Vec<proc_macro2::TokenStream> {
    insert_fields.iter().map(|(name, ty, _)| {
        let inner_field_type = get_inner_option_type(ty);
        let type_arg = match inner_field_type { Some(inner) => inner, None => ty };
        quote! { #name: Option<#type_arg>, }
    }).collect()
}

// Build phantom fields for required fields
pub fn build_phantom_struct_fields<'a>(required_fields: &[&'a (&'a Ident, &'a Type, &'a Vec<Attribute>)]) -> Vec<proc_macro2::TokenStream> {
    required_fields.iter().map(|(name, _, _)| {
        let gen_name_pascal = quote::format_ident!(
            "{}",
            name.to_string().from_case(Case::Snake).to_case(Case::Pascal)
        );
        let ph_name = quote::format_ident!("_{}", name);
        quote! { #ph_name: std::marker::PhantomData::<#gen_name_pascal>, }
    }).collect()
}

// Build initial generics (all NotSet)
pub fn build_initial_generics(required_fields: &[&(&Ident, &Type, &Vec<Attribute>)]) -> Vec<proc_macro2::TokenStream> {
    required_fields.iter().map(|_| quote! { NotSet, }).collect()
}

// Build initial struct fields (all None)
pub fn build_initial_struct_fields(insert_fields: &[&(&Ident, &Type, &Vec<Attribute>)]) -> Vec<proc_macro2::TokenStream> {
    insert_fields.iter().map(|(name, _, _)| quote! { #name: None, }).collect()
}

// Build initial phantom struct fields (all NotSet)
pub fn build_initial_phantom_struct_fields(required_fields: &[&(&Ident, &Type, &Vec<Attribute>)]) -> Vec<proc_macro2::TokenStream> {
    required_fields.iter().map(|(name, _, _)| {
        let ph_name = quote::format_ident!("_{}", name);
        quote! { #ph_name: std::marker::PhantomData::<NotSet>, }
    }).collect()
}

// Build builder methods (parameterized for different builder types)
pub fn build_builder_methods(
    insert_fields: &[&(&Ident, &Type, &Vec<Attribute>)],
    required_fields: &[&(&Ident, &Type, &Vec<Attribute>)],
    builder_struct_name: &Ident,
    get_inner_option_type: &dyn Fn(&Type) -> Option<&Type>,
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
