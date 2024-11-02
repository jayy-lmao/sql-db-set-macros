use pretty_assertions::assert_eq;

use crate::common::utils::{
    derive_input_from_string, pretty_print_tokenstream, tokenstream_from_string,
};

use super::one_query_builder;

pub fn compare_computed_to_expected(input_string: &str, output_string: &str) {
    let input_tokens = derive_input_from_string(input_string).expect("Could not get tokens");
    let out_tokens = one_query_builder::get_query_builder(&input_tokens);
    let pretty_out = pretty_print_tokenstream(out_tokens);
    let pretty_expected =
        pretty_print_tokenstream(tokenstream_from_string(output_string).expect("coudnt"));
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
    email: String, }
    "#;

    let output = r#"
pub struct NotSet;
pub struct Set;
#[derive(Debug)]
pub struct UserDbSetOneQueryBuilder<KeyFields = NotSet, UniqueFields = NotSet> {
    id: Option<String>, 
    email: Option<String>,
    _key_fields: std::marker::PhantomData<KeyFields>,
    _unique_fields: std::marker::PhantomData<UniqueFields>,
}
impl UserDbSetOneQueryBuilder {
    pub fn new() -> UserDbSetOneQueryBuilder<NotSet, NotSet> {
        Self {
            id: None,
            email: None,
            _key_fields: std::marker::PhantomData::<NotSet>,
            _unique_fields: std::marker::PhantomData::<NotSet>,
        }
    }
}
impl UserDbSetOneQueryBuilder<NotSet, NotSet> {
    pub fn email_eq(self, value: String) -> UserDbSetOneQueryBuilder<NotSet, Set> {
        UserDbSetOneQueryBuilder::<NotSet, Set> {
            email: Some(value),
            _key_fields: std::marker::PhantomData::<NotSet>,
            _unique_fields: std::marker::PhantomData::<Set>,
            id: self.id,
        }
    }
    pub fn id_eq(self, value: String) -> UserDbSetOneQueryBuilder<Set, NotSet> {
        UserDbSetOneQueryBuilder::<Set, NotSet> {
            id: Some(value),
            _key_fields: std::marker::PhantomData::<Set>,
            _unique_fields: std::marker::PhantomData::<NotSet>,
            email: self.email,
        }
    }
}
impl UserDbSetOneQueryBuilder<NotSet, Set> {
    pub async fn fetch_one<'e, E: sqlx::PgExecutor<'e>>(
        self,
        executor: E,
    ) -> Result<User, sqlx::Error> {
        sqlx::query_as!(
            User,
            "SELECT id, name, details, email FROM users WHERE (email = $1 OR $1 is null)",
            self.email,
        )
            .fetch_one(executor)
            .await
    }
}
impl UserDbSetOneQueryBuilder<Set, NotSet> {
    pub async fn fetch_one<'e, E: sqlx::PgExecutor<'e>>(
        self,
        executor: E,
    ) -> Result<User, sqlx::Error> {
        sqlx::query_as!(
            User,
            "SELECT id, name, details, email FROM users WHERE (id = $1 OR $1 is null)",
            self.id,
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
fn can_parse_order_with_key_into_one_builder() -> Result<(), String> {
    let input_str = r#"
#[dbset(table_name = "orders")]
pub struct Order {
    #[key]
    id: uuid::Uuid,
    item_name: String,
}
    "#;

    let output = r#"
pub struct NotSet;
pub struct Set;
#[derive(Debug)]
pub struct OrderDbSetOneQueryBuilder<KeyFields = NotSet> {
    id: Option<uuid::Uuid>,
    _key_fields: std::marker::PhantomData<KeyFields>,
}
impl OrderDbSetOneQueryBuilder {
    pub fn new() -> OrderDbSetOneQueryBuilder<NotSet> {
        Self {
            id: None,
            _key_fields: std::marker::PhantomData::<NotSet>,
        }
    }
}
impl OrderDbSetOneQueryBuilder<NotSet> {
    pub fn id_eq(self, value: uuid::Uuid) -> OrderDbSetOneQueryBuilder<Set> {
        OrderDbSetOneQueryBuilder::<Set> {
            id: Some(value),
            _key_fields: std::marker::PhantomData::<Set>,
        }
    }
}
impl OrderDbSetOneQueryBuilder<Set> {
    pub async fn fetch_one<'e, E: sqlx::PgExecutor<'e>>(
        self,
        executor: E,
    ) -> Result<Order, sqlx::Error> {
        sqlx::query_as!(
            Order,
            "SELECT id, item_name FROM orders WHERE (id = $1 OR $1 is null)",
            self.id,
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
fn can_parse_tag_with_unique_into_one_builder() -> Result<(), String> {
    let input_str = r#"
#[dbset(table_name = "orders")]
pub struct Tag {
    #[unique]
    tag_name: String,
}
    "#;

    let output = r#"
pub struct NotSet;
pub struct Set;
#[derive(Debug)]
pub struct TagDbSetOneQueryBuilder<UniqueFields = NotSet> {
    tag_name: Option<String>,
    _unique_fields: std::marker::PhantomData<UniqueFields>,
}
impl TagDbSetOneQueryBuilder {
    pub fn new() -> TagDbSetOneQueryBuilder<NotSet> {
        Self {
            tag_name: None,
            _unique_fields: std::marker::PhantomData::<NotSet>,
        }
    }
}
impl TagDbSetOneQueryBuilder<NotSet> {
    pub fn tag_name_eq(self, value: String) -> TagDbSetOneQueryBuilder<Set> {
        TagDbSetOneQueryBuilder::<Set> {
            tag_name: Some(value),
            _unique_fields: std::marker::PhantomData::<Set>,
        }
    }
}
impl TagDbSetOneQueryBuilder<Set> {
    pub async fn fetch_one<'e, E: sqlx::PgExecutor<'e>>(
        self,
        executor: E,
    ) -> Result<Tag, sqlx::Error> {
        sqlx::query_as!(
            Tag,
            "SELECT tag_name FROM orders WHERE (tag_name = $1 OR $1 is null)",
            self.tag_name,
        )
            .fetch_one(executor)
            .await
    }
}
    "#;

    compare_computed_to_expected(input_str, output);
    Ok(())
}
