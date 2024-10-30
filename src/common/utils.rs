use syn::{
    meta::ParseNestedMeta, punctuated::Punctuated, token::Comma, Attribute, Data, DeriveInput,
    Field, Fields, Ident, Meta, PathArguments, Type,
};

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

pub fn is_key_attr(attr: &Attribute) -> bool {
    match attr.meta {
        Meta::Path(ref path) => path.is_ident("key"),
        _ => false,
    }
}

fn extract_inner_string(input: &str) -> Option<String> {
    // Remove leading "= " and surrounding quotes
    if let Some(stripped) = input
        .strip_prefix("= \"")
        .and_then(|s| s.strip_suffix("\""))
    {
        return Some(stripped.to_string());
    }
    None
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
                            let parsed_inn_string =
                                extract_inner_string(&instring).expect("Could not extract inner");
                            table_name = parsed_inn_string;
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
                            let parsed_inn_string =
                                extract_inner_string(&instring).expect("Could not extract inner");
                            set_name = parsed_inn_string;
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
        let field_name = field.ident.as_ref().expect("could not cast ident as ref");
        field_names.push(field_name);
    }
    field_names
}

pub fn get_key_fields(input: &DeriveInput) -> Vec<(&Ident, &Type)> {
    let fields = get_fields(input);
    let mut key_fields = Vec::new();

    for field in fields {
        let field_name = field.ident.as_ref().expect("could not cast ident as ref");
        let field_type = &field.ty;
        let is_key = field.attrs.iter().any(is_key_attr);

        if is_key {
            key_fields.push((field_name, field_type));
        }
    }
    key_fields
}

pub fn get_unique_fields(input: &DeriveInput) -> Vec<(&Ident, &Type)> {
    let fields = get_fields(input);
    let mut unique_fields = Vec::new();

    for field in fields {
        let field_name = field.ident.as_ref().expect("could not cast ident as ref");
        let field_type = &field.ty;
        let is_unique = field.attrs.iter().any(is_unique_attr);

        if is_unique {
            unique_fields.push((field_name, field_type));
        }
    }
    unique_fields
}

pub fn get_all_fields(input: &DeriveInput) -> Vec<(&Ident, &Type)> {
    let mut all_fields = Vec::new();
    let fields = get_fields(input);
    for field in fields {
        let field_name = field.ident.as_ref().expect("could not cast ident as ref");
        let field_type = &field.ty;

        all_fields.push((field_name, field_type));
    }
    all_fields
}

pub fn join_field_names(fields: &[(&Ident, &Type)], separator: &str) -> String {
    fields
        .iter()
        .map(|(field_name, _)| field_name.to_string())
        .collect::<Vec<_>>()
        .join(separator)
}
