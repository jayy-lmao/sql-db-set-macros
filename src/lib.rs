use proc_macro::TokenStream;
use quote::quote;
use syn::{
    meta::ParseNestedMeta, parse::discouraged::AnyDelimiter, parse_macro_input,
    punctuated::Punctuated, Attribute, Data, DeriveInput, Fields, Ident, Lit, LitStr, Meta, Token,
};

#[proc_macro_derive(DbSet, attributes(unique, dbset, relation, auto, key))]
pub fn dbset_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;

    let (dbset_name, table_name) = get_dbset_and_table_names(&input.attrs, struct_name);

    let fields = if let Data::Struct(data) = &input.data {
        match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("DbSet can only be derived for structs with named fields"),
        }
    } else {
        panic!("DbSet can only be derived for structs");
    };

    let mut unique_fields = Vec::new();
    let mut key_fields = Vec::new();
    // let mut builder_fields = Vec::new();
    // let mut builder_methods = Vec::new();
    // let mut insertable_fields = Vec::new();
    // let mut insertable_values = Vec::new();
    let mut field_names = Vec::new();
    // let mut auto_fields = Vec::new();

    for field in fields.iter() {
        let field_name = field.ident.as_ref().expect("could not cast ident as ref");
        let field_type = &field.ty;
        let is_auto = field.attrs.iter().any(is_auto_attr);
        let is_unique = field.attrs.iter().any(is_unique_attr);
        let is_key = field.attrs.iter().any(is_key_attr);

        if is_unique {
            unique_fields.push((field_name, field_type));
        }

        if is_key {
            key_fields.push((field_name, field_type));
        }

        field_names.push(field_name);
        // builder_fields.push(quote! { #field_name: Option<#field_type> });
        // insertable_fields.push(quote! { #field_name: #field_type });
        // insertable_values
        //     .push(quote! { #field_name: self.#field_name.expect("could not get field name") });

        // builder_methods.push(quote! {
        //     pub fn #field_name(mut self, value: #field_type) -> Self {
        //         self.#field_name = Some(value);
        //         self
        //     }
        // });

        // if is_auto {
        //     auto_fields.push(field_name);
        // }
    }

    // Generate methods for unique fields (e.g., `by_email`)
    let has_single_key_field = key_fields.len() == 1;

    let key_method = {
        let all_keys_name_for_docstring = key_fields
            .iter()
            .map(|(field_name, _)| field_name.to_string())
            .collect::<Vec<_>>()
            .join(", ");

        let all_keys_name = key_fields
            .iter()
            .map(|(field_name, _)| field_name.to_string())
            .collect::<Vec<_>>()
            .join("_and_");

        let all_keys_where_clauses = key_fields
            .iter()
            .enumerate()
            .map(|(index, (field_name, _))| format!("{} = ${}", field_name.to_string(), index + 1))
            .collect::<Vec<_>>()
            .join(" AND ");

        let method_name = quote::format_ident!("by_{}", all_keys_name);

        let query = format!("SELECT * FROM {table_name} WHERE {all_keys_where_clauses}");
        let docstring = format!(
            "Get a `{struct_name}` by it's `{all_keys_name_for_docstring}` field.\n\nEquivalent of sqlx's `fetch_one` method."
        );

        let key_function_args = key_fields.iter().map(|(field_name, field_type)| {
            quote! {
                #field_name: #field_type,
            }
        });

        let key_query_args = key_fields.iter().map(|(field_name, _field_type)| {
            quote! {
                #field_name,
            }
        });

        quote! {
            #[doc = #docstring]
            pub async fn #method_name<'e, E: sqlx::PgExecutor<'e>>(
                executor: E,
                #(#key_function_args)*
            ) -> sqlx::Result<#struct_name> {
                sqlx::query_as!(
                    #struct_name,
                    #query,
                    #(#key_query_args)*
                )
                 .fetch_one(executor)
                 .await
            }
        }
    };

    let unique_methods = unique_fields.iter().filter(|(field_name,_)| {
        if  let Some((key_field_name,_)) = key_fields.first() {
            has_single_key_field  && key_field_name != field_name
        } else { false }

    })
        .map(|(field_name, field_type)| {
        let method_name = quote::format_ident!("by_{}", field_name);
        let query = format!("SELECT * FROM {table_name} WHERE {field_name} = $1");
        let docstring = format!(
            "Get a `{struct_name}` by it's `{field_name}` field.\n\nEquivalent of sqlx's `fetch_one` method."
        );

        quote! {
            #[doc = #docstring]
            pub async fn #method_name<'e, E: sqlx::PgExecutor<'e>>(
                executor: E,
                #field_name: #field_type,
            ) -> sqlx::Result<#struct_name> {
                sqlx::query_as!(
                    #struct_name,
                    #query,
                    #field_name,
                )
                 .fetch_one(executor)
                 .await
            }
        }
    });

    // // Generate builder struct, insertable struct, and methods
    // let builder_struct_name = quote::format_ident!("{}Builder", struct_name);
    // let insertable_struct_name = quote::format_ident!("{}Insertable", struct_name);

    // let builder_impl = quote! {
    //     pub struct #builder_struct_name {
    //         #(#builder_fields),*
    //     }

    //     impl #builder_struct_name {
    //         pub fn new() -> Self {
    //             Self {
    //                 #(#field_names: None),*
    //             }
    //         }

    //         #(#builder_methods)*

    //         pub fn insertable(self) -> sqlx::Result<#insertable_struct_name> {
    //             #(
    //                 if !#auto_fields && self.#field_names.is_none() {
    //                     return Err(sqlx::Error::ColumnNotFound(stringify!(#field_names).to_string()));
    //                 }
    //             )*
    //             Ok(#insertable_struct_name {
    //                 #(#insertable_values),*
    //             })
    //         }
    //     }
    // };

    // let insertable_impl = quote! {
    //     pub struct #insertable_struct_name {
    //         #(#insertable_fields),*
    //     }
    // };

    // // Generate DbSet struct with insert method
    // let field_names: Vec<syn::Ident> = field_names.into_iter().cloned().collect();

    let dbset_impl = quote! {
        pub struct #dbset_name;

        impl #dbset_name {
            #key_method
            #(#unique_methods)*

    //     //     pub async fn insert<'e, E: sqlx::PgExecutor<'e>>(
    //     //         &self,
    //     //         executor: E,
    //     //         entity: #insertable_struct_name,
    //     //     ) -> sqlx::Result<#struct_name> {
    //     //         sqlx::query_as!(
    //     //             #struct_name,
    //     //             concat!(
    //     //                 "INSERT INTO ", #table_name,
    //     //                 " (", stringify!(#(#field_names),*), ") VALUES (", #(
    //     //                     concat!("$", #(#field_names),*),
    //     //                 ), ") RETURNING *"
    //     //             ),
    //     //             #(
    //     //                 entity.#field_names,
    //     //             )*
    //     //         )
    //     //         .fetch_one(executor)
    //     //         .await
            // }
        }
    };

    let from_row_field_initializers = field_names.iter().map(|field_name| {
        // let field_name = &field.ident; // Field name in the struct
        // let column_name = field_name.as_ref().unwrap().to_string(); // Column name as a string
        let field_name_str = format!("{}", field_name);
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

    let expanded = quote! {
        #from_row_impl
        // #builder_impl
        // #insertable_impl
        #dbset_impl
    };

    println!("{}", expanded);

    TokenStream::from(expanded)
}

// Helper function to check if an attribute is #[auto]
fn is_auto_attr(attr: &Attribute) -> bool {
    match attr.meta {
        Meta::Path(ref path) => path.is_ident("auto"),
        _ => false,
    }
}

fn is_unique_attr(attr: &Attribute) -> bool {
    match attr.meta {
        Meta::Path(ref path) => path.is_ident("unique"),
        _ => false,
    }
}

fn is_key_attr(attr: &Attribute) -> bool {
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

fn get_dbset_and_table_names(attrs: &[Attribute], struct_name: &Ident) -> (Ident, String) {
    let mut set_name = format!("{}DbSet", struct_name);
    let mut table_name = struct_name.to_string().to_lowercase();

    for attr in attrs {
        if let Meta::List(meta) = attr.meta.clone() {
            if meta.path.is_ident("dbset") {
                meta.parse_nested_meta(|meta| {
                    if meta.path.is_ident("table_name") {
                        // if let Lit::Str(lit_str) = meta {}
                        if let ParseNestedMeta { path, input, .. } = meta {
                            let instring = input.to_string();
                            let parsed_inn_string =
                                extract_inner_string(&instring).expect("Couldnt extract inner");
                            table_name = parsed_inn_string;
                        }
                    } else if meta.path.is_ident("set_name") {
                        // if let Lit::Str(lit_str) = meta {}
                        if let ParseNestedMeta { path, input, .. } = meta {
                            let instring = input.to_string();
                            let parsed_inn_string =
                                extract_inner_string(&instring).expect("Could not extract inner");
                            table_name = parsed_inn_string;
                        }
                    }
                    Ok(())
                });
                // let args = meta
                //     .parse_args_with(Punctuated::<LitStr, Token![,]>::parse_terminated)
                //     .expect("Could not parse args");

                // for lit_str in args {
                //     let arg_value = lit_str.value();
                //     if arg_value.starts_with("table_name=") {
                //         // Extract the value after "table_name="
                //         table_name = arg_value
                //             .trim_start_matches("table_name=")
                //             .trim_matches('"')
                //             .to_string();
                //     }
                //     if arg_value.starts_with("set_name=") {
                //         // Extract the value after "set_name="
                //         set_name = arg_value
                //             .trim_start_matches("table_name=")
                //             .trim_matches('"')
                //             .to_string();
                //     }
                // }
            }
        }
    }

    (Ident::new(&set_name, struct_name.span()), table_name)
}
