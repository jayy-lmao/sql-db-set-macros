use pretty_assertions::assert_eq;

use crate::{
    common::utils::{derive_input_from_string, pretty_print_tokenstream, tokenstream_from_string},
    modules::many_query_builder::get_query_builder,
};

pub fn compare_computed_to_expected(input_string: &str, output_string: &str) {
    let input_tokens = derive_input_from_string(input_string).expect("Could not get tokens");
    let out_tokens = get_query_builder(&input_tokens);
    let pretty_out = pretty_print_tokenstream(out_tokens);
    let tokenstream = tokenstream_from_string(output_string);
    assert!(
        tokenstream.is_ok(),
        "Could not parse output: {output_string}"
    );
    let pretty_expected = pretty_print_tokenstream(tokenstream.expect("couldnt unw"));
    assert_eq!(pretty_out.to_string(), pretty_expected);
}

#[test]
fn can_parse_user_struct_with_unique_and_key_into_one_builder() -> Result<(), String> {
    let input_str = r#"
#[dbset(table_name = "users")]
pub struct User {
    #[key]
    id: String,
    name: String,
    details: Option<String>,
    #[unique]
    email: String, 
    }
    "#;

    let output = r#"

pub struct UserDbSetManyQueryBuilder {
    name: Option<String>,
    details: Option<String>,
}
impl UserDbSetManyQueryBuilder {
    pub fn new() -> Self {
        Self { name: None, details: None }
    }
    pub fn name_eq(mut self, value: String) -> Self {
        self.name = Some(value);
        self
    }
    pub fn details_eq(mut self, value: String) -> Self {
        self.details = Some(value);
        self
    }
    pub async fn fetch_all<'e, E: sqlx::PgExecutor<'e>>(
        self,
        executor: E,
    ) -> Result<Vec<User>, sqlx::Error> {
        sqlx::query_as!(
            User,
            "SELECT id, name, details, email FROM users WHERE (name = $1 or $1 is null) AND (details = $2 or $2 is null)",
            self.name, self.details,
        )
            .fetch_all(executor)
            .await
    }
}
    "#;

    compare_computed_to_expected(input_str, output);
    Ok(())
}

#[test]
fn can_parse_order_with_two_keys_into_one_builder() -> Result<(), String> {
    let input_str = r#"
#[dbset(table_name = "favourite_products")]
pub struct FavouritedProduct {
    #[key]
    product_id: uuid::Uuid,
    #[key]
    user_id: uuid::Uuid,
}
    "#;

    let output = r#"

pub struct FavouritedProductDbSetManyQueryBuilder {
    product_id: Option<uuid::Uuid>,
    user_id: Option<uuid::Uuid>,
}

impl FavouritedProductDbSetManyQueryBuilder {
    pub fn new() -> Self {
        Self {
    product_id: None,
    user_id: None,
        }
    }

    pub async fn fetch_all<'e, E: sqlx::PgExecutor<'e>>(
        self,
        executor: E,
    ) -> Result<Vec<User>, sqlx::Error> {
        sqlx::query_as!(
            FavouritedProduct,
            "SELECT product_id, user_id FROM favourite_products WHERE (product_id = $1 or $1 is null) AND (user_id = $2 or $2 is null)",
            self.product_id, self.user_id,
        )
            .fetch_all(executor)
            .await
            }


}

    "#;

    compare_computed_to_expected(input_str, output);
    Ok(())
}
