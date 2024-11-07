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
pub struct AccountDbSetInsertBuilder<Email = NotSet> {
    email: Option<String>,
    _email: std::marker::PhantomData<Email>,
}
impl AccountDbSetInsertBuilder {
    pub fn new() -> AccountDbSetInsertBuilder<NotSet> {
        Self {
            email: None,
            _email: std::marker::PhantomData::<NotSet>,
        }
    }
}
impl AccountDbSetInsertBuilder<NotSet> {
    pub fn email(self, email: String) -> AccountDbSetInsertBuilder<Set> {
        AccountDbSetInsertBuilder {
            email: Some(email),
            _email: std::marker::PhantomData::<Set>,
        }
    }
}
impl AccountDbSetInsertBuilder<Set> {
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
pub struct UserDbSetInsertBuilder<Id = NotSet, Name = NotSet, Email = NotSet> {
    id: Option<String>,
    name: Option<String>,
    details: Option<String>,
    email: Option<String>,
    _id: std::marker::PhantomData<Id>,
    _name: std::marker::PhantomData<Name>,
    _email: std::marker::PhantomData<Email>,
}
impl UserDbSetInsertBuilder {
    pub fn new() -> UserDbSetInsertBuilder<NotSet, NotSet, NotSet> {
        Self {
            id: None,
            name: None,
            details: None,
            email: None,
            _id: std::marker::PhantomData::<NotSet>,
            _name: std::marker::PhantomData::<NotSet>,
            _email: std::marker::PhantomData::<NotSet>,
        }
    }
}
impl<Name, Email> UserDbSetInsertBuilder<NotSet, Name, Email> {
    pub fn id(self, id: String) -> UserDbSetInsertBuilder<Set, Name, Email> {
        UserDbSetInsertBuilder {
            id: Some(id),
            name: self.name,
            details: self.details,
            email: self.email,
            _id: std::marker::PhantomData::<Set>,
            _name: self._name,
            _email: self._email,
        }
    }
}
impl<Id, Email> UserDbSetInsertBuilder<Id, NotSet, Email> {
    pub fn name(self, name: String) -> UserDbSetInsertBuilder<Id, Set, Email> {
        UserDbSetInsertBuilder {
            name: Some(name),
            id: self.id,
            details: self.details,
            email: self.email,
            _name: std::marker::PhantomData::<Set>,
            _id: self._id,
            _email: self._email,
        }
    }
}
impl<Id,Name, Email> UserDbSetInsertBuilder<Id, Name, Email> {
    pub fn details(self, details: String) -> UserDbSetInsertBuilder<Id, Name, Email> {
        UserDbSetInsertBuilder {
            details: Some(details),
            id: self.id,
            name: self.name,
            email: self.email,
            _id: self._id,
            _name: self._name,
            _email: self._email,
        }
    }
}
impl<Id, Name> UserDbSetInsertBuilder<Id, Name, NotSet> {
    pub fn email(self, email: String) -> UserDbSetInsertBuilder<Id, Name, Set> {
        UserDbSetInsertBuilder {
            email: Some(email),
            id: self.id,
            name: self.name,
            details: self.details,
            _email: std::marker::PhantomData::<Set>,
            _id: self._id,
            _name: self._name,
        }
    }
}
impl UserDbSetInsertBuilder<Set, Set, Set> {
    pub async fn insert<'e, E: sqlx::PgExecutor<'e>>(
        self,
        executor: E,
    ) -> Result<User, sqlx::Error> {
        sqlx::query_as!(
            User,
            "INSERT INTO users(id, name, details, email) VALUES ($1, $2, $3, $4) RETURNING id, name, details, email;",
            self.id, self.name, self.details, self.email,
        )
            .fetch_one(executor)
            .await
    }
}


    "#;

    compare_computed_to_expected(input_str, output);
    Ok(())
}
