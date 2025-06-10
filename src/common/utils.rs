use std::str::FromStr;

use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
    meta::ParseNestedMeta, parse2, punctuated::Punctuated, token::Comma, Attribute, Data,
    DeriveInput, Field, Fields, File, Ident, Meta, PathArguments, Type,
};

pub enum Additional {
    IsEnum(String),
}

pub fn derive_input_from_string(input: &str) -> Result<DeriveInput, syn::Error> {
    let token_stream = TokenStream::from_str(input)?;
    parse2::<DeriveInput>(token_stream)
}
pub fn tokenstream_from_string(input: &str) -> Result<proc_macro2::TokenStream, String> {
    proc_macro2::TokenStream::from_str(input)
        .map_err(|err| syn::Error::new(proc_macro2::Span::call_site(), err).to_string())
}

pub fn pretty_print_tokenstream(ts: proc_macro2::TokenStream) -> String {
    match parse2::<File>(ts.clone()) {
        Ok(file) => prettyplease::unparse(&file).to_string(),
        Err(err) => format!("Failed to parse TokenStream: {err}. Stream was {ts}"),
    }
}

// Helper function to check if an attribute is #[auto]
pub fn is_auto_attr(attr: &Attribute) -> bool {
    match attr.meta {
        Meta::Path(ref path) => path.is_ident("auto"),
        _ => false,
    }
}

pub fn is_unique_attr(attr: &Attribute) -> bool {
    match attr.meta {
        Meta::Path(ref path) => path.is_ident("unique"),
        _ => false,
    }
}

pub fn is_custom_enum_attr(attr: &Attribute) -> bool {
    match attr.meta {
        Meta::Path(ref path) => path.is_ident("custom_enum"),
        _ => false,
    }
}

pub fn is_key_attr(attr: &Attribute) -> bool {
    match attr.meta {
        Meta::Path(ref path) => path.is_ident("key"),
        _ => false,
    }
}

fn extract_inner_string(input: &str) -> Option<String> {
    // Remove leading "= " and surrounding quotes
    if let Some(stripped) = input
        .trim()
        .strip_prefix("=")?
        .trim()
        .strip_prefix("\"")
        .and_then(|s| s.strip_suffix("\""))
    {
        return Some(stripped.to_string());
    }
    None
}

#[test]
fn extract_inner_string_1() {
    let result = extract_inner_string("= \"foo\"").expect("couldnt get inner");
    assert_eq!(result, "foo");
}
#[test]
fn extract_inner_string_2() {
    let result = extract_inner_string("=\"foo\"").expect("couldnt get inner");
    assert_eq!(result, "foo");
}
#[test]
fn extract_inner_string_3() {
    let result = extract_inner_string(" =\"foo\"").expect("couldnt get inner");
    assert_eq!(result, "foo");
}

pub fn get_table_name(input: &DeriveInput) -> String {
    let struct_name = &input.ident;
    let mut table_name = struct_name.to_string().to_lowercase();

    for attr in &input.attrs {
        if let Meta::List(meta) = attr.meta.clone() {
            if meta.path.is_ident("dbset") {
                let _ = meta.parse_nested_meta(|meta| {
                    if meta.path.is_ident("table_name") {
                        // if let Lit::Str(lit_str) = meta {}
                        if let ParseNestedMeta { input, .. } = meta {
                            let instring = input.to_string();
                            let parsed_inn_string_maybe = extract_inner_string(&instring);
                            if let Some(parsed_inn_string) = parsed_inn_string_maybe {
                                table_name = parsed_inn_string;
                            }
                        }
                    }
                    Ok(())
                });
            }
        }
    }

    table_name
}

pub fn get_dbset_name(input: &DeriveInput) -> Ident {
    let struct_name = &input.ident;
    let mut set_name = format!("{struct_name}DbSet");

    for attr in &input.attrs {
        if let Meta::List(meta) = attr.meta.clone() {
            if meta.path.is_ident("dbset") {
                let _ = meta.parse_nested_meta(|meta| {
                    if meta.path.is_ident("set_name") {
                        // if let Lit::Str(lit_str) = meta {}
                        if let ParseNestedMeta { input, .. } = meta {
                            let instring = input.to_string();
                            let parsed_inn_string_maybe = extract_inner_string(&instring);
                            if let Some(parsed_inn_string) = parsed_inn_string_maybe {
                                set_name = parsed_inn_string;
                            }
                        }
                    }
                    Ok(())
                });
            }
        }
    }

    Ident::new(&set_name, struct_name.span())
}

pub fn get_inner_option_type(ty: &Type) -> Option<&Type> {
    if let Type::Path(type_path) = ty {
        // Check if the path is `Option`
        if type_path.path.segments.last()?.ident == "Option" {
            if let PathArguments::AngleBracketed(args) = &type_path.path.segments.last()?.arguments
            {
                // Return the inner type `T` in `Option<T>`
                if let Some(syn::GenericArgument::Type(inner_type)) = args.args.first() {
                    return Some(inner_type);
                }
            }
        }
    }
    None
}

pub fn get_struct_name(input: &DeriveInput) -> &Ident {
    &input.ident
}

pub fn get_fields(input: &DeriveInput) -> &Punctuated<Field, Comma> {
    if let Data::Struct(data) = &input.data {
        match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("DbSet can only be derived for structs with named fields"),
        }
    } else {
        panic!("DbSet can only be derived for structs");
    }
}

pub fn get_field_names(input: &DeriveInput) -> Vec<&Ident> {
    let fields = get_fields(input);
    let mut field_names = Vec::new();
    for field in fields {
        let field_name_maybe = field.ident.as_ref();
        if let Some(field_name) = field_name_maybe {
            field_names.push(field_name);
        }
    }
    field_names
}

pub fn get_auto_fields(input: &DeriveInput) -> Vec<(&Ident, &Type, &Vec<Attribute>)> {
    let fields = get_fields(input);
    let mut auto_fields = Vec::new();

    for field in fields {
        let field_name_maybe = field.ident.as_ref();
        if let Some(field_name) = field_name_maybe {
            let field_type = &field.ty;
            let is_auto = field.attrs.iter().any(is_auto_attr);

            if is_auto {
                auto_fields.push((field_name, field_type, &field.attrs));
            }
        }
    }
    auto_fields
}

pub fn get_key_fields(input: &DeriveInput) -> Vec<(&Ident, &Type)> {
    let fields = get_fields(input);
    let mut key_fields = Vec::new();

    for field in fields {
        let field_name_maybe = field.ident.as_ref();
        if let Some(field_name) = field_name_maybe {
            let field_type = &field.ty;
            let is_key = field.attrs.iter().any(is_key_attr);

            if is_key {
                key_fields.push((field_name, field_type));
            }
        }
    }
    key_fields
}

pub fn get_unique_fields(input: &DeriveInput) -> Vec<(&Ident, &Type)> {
    let fields = get_fields(input);
    let mut unique_fields = Vec::new();

    for field in fields {
        let field_name_maybe = field.ident.as_ref();
        if let Some(field_name) = field_name_maybe {
            let field_type = &field.ty;
            let is_unique = field.attrs.iter().any(is_unique_attr);

            if is_unique {
                unique_fields.push((field_name, field_type));
            }
        }
    }
    unique_fields
}

pub fn get_all_fields(input: &DeriveInput) -> Vec<(&Ident, &Type, &Vec<Attribute>)> {
    let mut all_fields = Vec::new();
    let fields = get_fields(input);
    for field in fields {
        let field_name_maybe = field.ident.as_ref();
        if let Some(field_name) = field_name_maybe {
            let field_type = &field.ty;

            all_fields.push((field_name, field_type, &field.attrs));
        }
    }
    all_fields
}

pub fn get_query_fields_string(input: &DeriveInput) -> String {
    let mut all_fields = Vec::new();
    let fields = get_fields(input);
    for field in fields {
        let field_name_maybe = field.ident.as_ref();
        if let Some(field_name) = field_name_maybe {
            let field_type = &field.ty;

            let field_name_string = field_name.to_string();
            let is_custom_enum = field.attrs.iter().any(is_custom_enum_attr);
            if is_custom_enum {
                let field_type_string = field_type.to_token_stream().to_string();
                let custom_enum_field =
                    format!("{field_name_string} AS \"{field_name_string}:{field_type_string}\"");
                all_fields.push(custom_enum_field);
            } else {
                all_fields.push(field_name.to_string())
            }
        }
    }
    all_fields.join(", ")
}

pub fn join_field_names(fields: &[(&Ident, &Type)], separator: &str) -> String {
    fields
        .iter()
        .map(|(field_name, _)| field_name.to_string())
        .collect::<Vec<_>>()
        .join(separator)
}
