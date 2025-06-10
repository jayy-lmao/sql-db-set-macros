use db_set_macros::DbSet;

use crate::harness::get_db_pool;

#[derive(sqlx::Type, Debug, Clone)]
#[sqlx(type_name = "user_status")]
pub enum UserStatus {
    #[sqlx(rename = "verified")]
    Verified,
    #[sqlx(rename = "Unverified")]
    Unverified,
}

#[derive(DbSet, Debug, Clone)]
#[dbset(table_name = "users")]
pub struct User {
    #[key]
    id: String,
    name: String,
    details: Option<String>,
    #[unique]
    email: String,
    status: UserStatus,
}

#[tokio::test]
async fn test_fetch_user_by_id() -> Result<(), String> {
    let pool = get_db_pool().await;

    let user = UserDbSet::one()
        .id_eq("user-1".to_string())
        .fetch_one(pool)
        .await
        .expect("could not run query");

    assert_eq!(user.id, "user-1");
    assert_eq!(user.name, "bob");
    Ok(())
}

#[tokio::test]
async fn test_query_as_user() -> Result<(), String> {
    let pool = get_db_pool().await;

    let user: User = sqlx::query_as!(
        User,
        "SELECT id, name, email, details, status FROM users LIMIT 1;"
    )
    .fetch_one(pool)
    .await
    .expect("Could not fetch one");

    assert_eq!(user.name, "bob");
    Ok(())
}

#[tokio::test]
async fn test_fetch_users_by_name() -> Result<(), String> {
    let pool = get_db_pool().await;

    let users = UserDbSet::many()
        .name_eq("bob".to_string())
        .fetch_all(pool)
        .await
        .expect("Could not fetch users");

    assert_eq!(users.len(), 2);
    Ok(())
}

#[tokio::test]
async fn test_fetch_users_by_name_and_details() -> Result<(), String> {
    let pool = get_db_pool().await;

    let users = UserDbSet::many()
        .name_eq("bob".to_string())
        .details_eq("the best bob".to_string())
        .fetch_all(pool)
        .await
        .expect("Could not fetch users");

    assert_eq!(users.len(), 1);
    Ok(())
}

#[tokio::test]
async fn test_fetch_all_users() -> Result<(), String> {
    let pool = get_db_pool().await;

    let users = UserDbSet::many()
        .fetch_all(pool)
        .await
        .expect("Could not fetch users");

    assert_eq!(users.len(), 3);
    Ok(())
}

#[tokio::test]
async fn test_insert_users() -> Result<(), String> {
    let pool = get_db_pool().await;

    let inserted_user = UserDbSet::insert()
        .id("id-3".to_string())
        .email("steven@stevenson.com".to_string())
        .name("steven".to_string())
        .status(UserStatus::Verified)
        .insert(pool)
        .await
        .expect("Could not insert");

    let matched_user = sqlx::query_as!(
        User,
        "SELECT id, name, email, details, status FROM users WHERE id = 'id-3';"
    )
    .fetch_one(pool)
    .await
    .expect("Could not fetch one");

    assert_eq!(matched_user.email, inserted_user.email);

    Ok(())
}

#[tokio::test]
async fn test_update_users() -> Result<(), String> {
    let pool = get_db_pool().await;

    let mut user = sqlx::query_as!(
        User,
        "SELECT id, name, email, details, status FROM users WHERE id = 'user-2';"
    )
    .fetch_one(pool)
    .await
    .expect("Could not fetch one");

    user.details = Some("Updated details!".to_string());
    user.email = String::from("mynewemail@bigpond.com.au");
    UserDbSet::update()
        .data(user.clone())
        .update(pool)
        .await
        .expect("Could not update");

    let same_user_again = sqlx::query_as!(
        User,
        "SELECT id, name, email, details, status FROM users WHERE id = 'user-2';"
    )
    .fetch_one(pool)
    .await
    .expect("Could not fetch one");

    assert_eq!(user.email, same_user_again.email);
    assert_eq!(user.details, same_user_again.details);

    Ok(())
}

#[tokio::test]
async fn test_delete_users() -> Result<(), String> {
    let pool = get_db_pool().await;

    sqlx::query("INSERT INTO users (name, id, email, status) VALUES ('lana del ete', 'id-6', 'lana@bigpond.com.au', 'verified');")
        .execute(pool)
        .await
        .expect("Could not initialise db");

    UserDbSet::delete()
        .id_eq("id-6".to_string())
        .delete(pool)
        .await
        .expect("could not delete");

    let matched_user = sqlx::query_as!(
        User,
        "SELECT id, name, email, details, status FROM users WHERE id = 'id-6';"
    )
    .fetch_optional(pool)
    .await
    .expect("Could not fetch one");

    assert!(matched_user.is_none());

    Ok(())
}
