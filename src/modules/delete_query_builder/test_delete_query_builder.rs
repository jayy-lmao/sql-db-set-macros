use pretty_assertions::assert_eq;

use crate::common::utils::{
    derive_input_from_string, pretty_print_tokenstream, tokenstream_from_string,
};

use super::delete_query_builder;

pub fn compare_computed_to_expected(input_string: &str, output_string: &str) {
    let input_tokens = derive_input_from_string(input_string).expect("Could not get tokens");
    let out_tokens = delete_query_builder::get_query_builder(&input_tokens);
    let pretty_out = pretty_print_tokenstream(out_tokens);
    let pretty_expected =
        pretty_print_tokenstream(tokenstream_from_string(output_string).expect("coudnt"));
    assert_eq!(pretty_out.to_string(), pretty_expected);
}

#[test]
fn can_parse_user_struct_with_unique_and_key_into_delete_builder() -> Result<(), String> {
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
pub struct UserDbSetDeleteQueryBuilder<id = NotSet, UniqueFields = NotSet> {
    id: Option<String>,
    email: Option<String>,
    _unique_fields: std::marker::PhantomData<UniqueFields>,
    _id: std::marker::PhantomData<id>,
}
impl UserDbSetDeleteQueryBuilder {
    pub fn new() -> UserDbSetDeleteQueryBuilder<NotSet, NotSet> {
        Self {
            id: None,
            email: None,
            _unique_fields: std::marker::PhantomData::<NotSet>,
            _id: std::marker::PhantomData::<NotSet>,
        }
    }
}
impl UserDbSetDeleteQueryBuilder<NotSet, NotSet> {
    pub fn id_eq(self, id: String) -> UserDbSetDeleteQueryBuilder<Set, NotSet> {
        UserDbSetDeleteQueryBuilder {
            id: Some(id),
            email: self.email,
            _id: std::marker::PhantomData::<Set>,
            _unique_fields: std::marker::PhantomData::<NotSet>,
        }
    }
}
impl<id> UserDbSetDeleteQueryBuilder<id, NotSet> {
    pub fn email_eq(self, email: String) -> UserDbSetDeleteQueryBuilder<id, Set> {
        UserDbSetDeleteQueryBuilder {
            email: Some(email),
            id: self.id,
            _unique_fields: std::marker::PhantomData::<Set>,
            _id: self._id,
        }
    }
}
impl UserDbSetDeleteQueryBuilder<Set, NotSet> {
    pub async fn delete<'e, E: sqlx::PgExecutor<'e>>(
        self,
        executor: E,
    ) -> Result<User, sqlx::Error> {
        sqlx::query!(User, "DELETE FROM users WHERE id = $1", self.id,)
            .execute(executor)
            .await
    }
}
impl UserDbSetDeleteQueryBuilder<NotSet, Set> {
    pub async fn delete<'e, E: sqlx::PgExecutor<'e>>(
        self,
        executor: E,
    ) -> Result<User, sqlx::Error> {
        sqlx::query!(
            User, "DELETE FROM users WHERE (email = $1 OR $1 is null)", self.email,
        )
            .execute(executor)
            .await
    }
}


    "#;

    compare_computed_to_expected(input_str, output);
    Ok(())
}

#[ignore]
#[test]
fn can_parse_order_with_key_into_delete_builder() -> Result<(), String> {
    let input_str = r#"
#[dbset(table_name = "orders")]
pub struct Order {
    #[key]
    id: uuid::Uuid,
    item_name: String,
}
    "#;

    let output = r#"
    "#;

    compare_computed_to_expected(input_str, output);
    Ok(())
}

#[test]
fn can_parse_tag_with_unique_into_delete_builder() -> Result<(), String> {
    let input_str = r#"
#[dbset(table_name = "tags")]
pub struct Tag {
    #[unique]
    tag_name: String,
}
    "#;

    let output = r#"
pub struct TagDbSetDeleteQueryBuilder<UniqueFields = NotSet> {
    tag_name: Option<String>,
    _unique_fields: std::marker::PhantomData<UniqueFields>,
}
impl TagDbSetDeleteQueryBuilder {
    pub fn new() -> TagDbSetDeleteQueryBuilder<NotSet> {
        Self {
            tag_name: None,
            _unique_fields: std::marker::PhantomData::<NotSet>,
        }
    }
}
impl TagDbSetDeleteQueryBuilder<NotSet> {
    pub fn tag_name_eq(self, tag_name: String) -> TagDbSetDeleteQueryBuilder<Set> {
        TagDbSetDeleteQueryBuilder {
            tag_name: Some(tag_name),
            _unique_fields: std::marker::PhantomData::<Set>,
        }
    }
}
impl TagDbSetDeleteQueryBuilder<Set> {
    pub async fn delete<'e, E: sqlx::PgExecutor<'e>>(
        self,
        executor: E,
    ) -> Result<Tag, sqlx::Error> {
        sqlx::query!(
            Tag, "DELETE FROM tags WHERE (tag_name = $1 OR $1 is null)", self.tag_name,
        )
            .execute(executor)
            .await
    }
}


    "#;

    compare_computed_to_expected(input_str, output);
    Ok(())
}

#[test]
fn can_parse_order_with_two_keys_into_delete_builder() -> Result<(), String> {
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
pub struct FavouritedProductDbSetDeleteQueryBuilder<
    product_id = NotSet,
    user_id = NotSet,
    UniqueFields = NotSet,
> {
    product_id: Option<uuid::Uuid>,
    user_id: Option<uuid::Uuid>,
    _unique_fields: std::marker::PhantomData<UniqueFields>,
    _product_id: std::marker::PhantomData<product_id>,
    _user_id: std::marker::PhantomData<user_id>,
}

impl FavouritedProductDbSetDeleteQueryBuilder {
    pub fn new() -> FavouritedProductDbSetDeleteQueryBuilder<NotSet, NotSet, NotSet> {
        Self {
            product_id: None,
            user_id: None,
            _unique_fields: std::marker::PhantomData::<NotSet>,
            _product_id: std::marker::PhantomData::<NotSet>,
            _user_id: std::marker::PhantomData::<NotSet>,
        }
    }
}
impl<user_id> FavouritedProductDbSetDeleteQueryBuilder<NotSet, user_id, NotSet> {
    pub fn product_id_eq(
        self,
        product_id: uuid::Uuid,
    ) -> FavouritedProductDbSetDeleteQueryBuilder<Set, user_id, NotSet> {
        FavouritedProductDbSetDeleteQueryBuilder {
            product_id: Some(product_id),
            user_id: self.user_id,
            _product_id: std::marker::PhantomData::<Set>,
            _user_id: self._user_id,
            _unique_fields: std::marker::PhantomData::<NotSet>,
        }
    }
}
impl<product_id> FavouritedProductDbSetDeleteQueryBuilder<product_id, NotSet, NotSet> {
    pub fn user_id_eq(
        self,
        user_id: uuid::Uuid,
    ) -> FavouritedProductDbSetDeleteQueryBuilder<product_id, Set, NotSet> {
        FavouritedProductDbSetDeleteQueryBuilder {
            user_id: Some(user_id),
            product_id: self.product_id,
            _user_id: std::marker::PhantomData::<Set>,
            _product_id: self._product_id,
            _unique_fields: std::marker::PhantomData::<NotSet>,
        }
    }
}
impl FavouritedProductDbSetDeleteQueryBuilder<Set, Set, NotSet> {
    pub async fn delete<'e, E: sqlx::PgExecutor<'e>>(
        self,
        executor: E,
    ) -> Result<FavouritedProduct, sqlx::Error> {
        sqlx::query!(
            FavouritedProduct,
            "DELETE FROM favourite_products WHERE product_id = $1 AND user_id = $2", self
            .product_id, self.user_id,
        )
            .execute(executor)
            .await
    }
}


    "#;

    compare_computed_to_expected(input_str, output);
    Ok(())
}
