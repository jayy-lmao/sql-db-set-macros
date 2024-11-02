use pretty_assertions::assert_eq;

use crate::common::utils::{
    derive_input_from_string, pretty_print_tokenstream, tokenstream_from_string,
};

use super::insert_query_builder;

pub fn compare_computed_to_expected(input_string: &str, output_string: &str) {
    let input_tokens = derive_input_from_string(input_string).expect("Could not get tokens");
    let out_tokens = insert_query_builder::get_insert_query_builder(&input_tokens);
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
        pub struct Set;
        pub struct NotSet;

pub struct AccountInsertBuilder<email = NotSet> {
    email: Option<String>,
    _email: std::marker::PhantomData<email>,

}
impl AccountInsertBuilder {
    pub fn new() -> AccountInsertBuilder<NotSet> {
        Self { 
            email: None,
            _email: std::marker::PhantomData::<NotSet>,

        }
    }
}

impl Account<NotSet> {
    pub fn email_eq(self, value: String) -> AccountInsertBuilder<Set> {
        AccountInsertBuilder {
            email: Some(value),
            _email: std::marker::PhantomData::<Set>,
        }
    }
}

impl AccountInsertBuilder<Set>  {
    pub async fn insert<'e, E: sqlx::PgExecutor<'e>>(
        self,
        executor: E,
    ) -> Result<Account, sqlx::Error> {
        sqlx::query_as!(
            Account, "INSERT INTO users(email) VALUES ($1) RETURNING id, email;", self
            .email,
        )
            .fetch_one(executor)
            .await
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
        pub struct Set;
        pub struct NotSet;

pub struct UserInsertBuilder<id = NotSet, email = NotSet> {
    id: Option<String>,
    email: Option<String>,
    _id: std::marker::PhantomData<id>,
    _email: std::marker::PhantomData<email>,
}
impl UserInsertBuilder {
    pub fn new() -> UserInsertBuilder<NotSet, NotSet> {
        Self {
            id: None,
            email: None,
            _id: std::marker::PhantomData::<NotSet>,
            _email: std::marker::PhantomData::<NotSet>,
        }
    }
}

impl<email> User<NotSet, email> {
    pub fn id_eq(self, value: String) -> UserInsertBuilder<Set, email> {
        UserInsertBuilder {
            id: Some(value),
            email: self.email,
            _id: std::marker::PhantomData::<Set>,
            _email: self._email,
        }
    }
}
impl<id> User<id, NotSet> {
    pub fn email_eq(self, value: String) -> UserInsertBuilder<id, Set> {
        UserInsertBuilder {
            email: Some(value),
            id: self.id,
            _email: std::marker::PhantomData::<Set>,
            _id: self._id,
        }
    }
}

impl UserInsertBuilder<Set, Set> {
    pub async fn insert<'e, E: sqlx::PgExecutor<'e>>(
        self,
        executor: E,
    ) -> Result<User, sqlx::Error> {
        sqlx::query_as!(
            User,
            "INSERT INTO users(id, email) VALUES ($1, $2) RETURNING id, name, details, email;",
            self.id, self.email,
        )
            .fetch_one(executor)
            .await
    }
}

    "#;

    compare_computed_to_expected(input_str, output);
    Ok(())
}
