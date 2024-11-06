use pretty_assertions::assert_eq;

use crate::{
    common::utils::{derive_input_from_string, pretty_print_tokenstream, tokenstream_from_string},
    modules::update_query_builder::update_query_builder,
};

pub fn compare_computed_to_expected(input_string: &str, output_string: &str) {
    let input_tokens = derive_input_from_string(input_string).expect("Could not get tokens");
    let out_tokens = update_query_builder::get_update_query_builder(&input_tokens);
    let pretty_out = pretty_print_tokenstream(out_tokens);
    let pretty_expected =
        pretty_print_tokenstream(tokenstream_from_string(output_string).expect("coudnt"));
    assert_eq!(pretty_out.to_string(), pretty_expected);
}

#[test]
fn can_parse_user_struct_with_unique_and_auto_key_into_one_builder() -> Result<(), String> {
    let input_str = r#"
#[dbset(table_name = "users")]
pub struct Account {
    #[key]
    #[auto]
    id: uuid::Uuid,
    #[unique]
    email: String, }
    "#;

    let output = r#"
pub struct AccountDbSetUpdateBuilder {}
pub struct AccountDbSetUpdateBuilderWithData {
    updatable: Account,
}
impl AccountDbSetUpdateBuilder {
    pub fn new() -> AccountDbSetUpdateBuilder {
        Self {}
    }
}
impl AccountDbSetUpdateBuilder {
    pub fn data(self, value: Account) -> AccountDbSetUpdateBuilderWithData {
        AccountDbSetUpdateBuilderWithData {
            updatable: value,
        }
    }
}
impl AccountDbSetUpdateBuilderWithData {
    pub async fn update<'e, E: sqlx::PgExecutor<'e>>(
        self,
        executor: E,
    ) -> Result<Account, sqlx::Error> {

        sqlx::query_as!(Account, "UPDATE users SET email = $2 WHERE id = $1 RETURNING id, email;",
            self.updatable.email,
        ).fetch_one(executor).await
    }
}


    "#;

    compare_computed_to_expected(input_str, output);
    Ok(())
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
    email: String, }
    "#;

    let output = r#"
pub struct UserDbSetUpdateBuilder {}
pub struct UserDbSetUpdateBuilderWithData {
    updatable: User,
}

impl UserDbSetUpdateBuilder {
    pub fn new() -> UserDbSetUpdateBuilder {
        Self { }
    }
}
impl UserDbSetUpdateBuilder {
    pub fn data(self, value: User) -> UserDbSetUpdateBuilderWithData {
        UserDbSetUpdateBuilderWithData {
            updatable: value,
        }
    }
}
impl UserDbSetUpdateBuilderWithData {
    pub async fn update<'e, E: sqlx::PgExecutor<'e>>(
        self,
        executor: E,
    ) -> Result<User, sqlx::Error> {
        sqlx::query_as!(
            User, "UPDATE users SET name = $2, details = $3, email = $4 WHERE id = $1 RETURNING id, name, details, email;",
            self.updatable.id, self.updatable.name, self.updatable.details, self.updatable.email,
        )
            .fetch_one(executor)
            .await
    }
}


    "#;

    compare_computed_to_expected(input_str, output);
    Ok(())
}
