use db_set_macros::DbSet;
use sqlx::PgPool;

#[derive(sqlx::Type, Debug, Clone)]
#[sqlx(type_name = "user_status", rename_all = "snake_case")]
pub enum UserStatus {
    Verified,
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
    #[custom_enum]
    status: UserStatus,
}

#[sqlx::test(fixtures("users"))]
async fn test_fetch_user_by_id(pool: PgPool) -> sqlx::Result<()> {
    let user = UserDbSet::one()
        .id_eq("user-1".to_string())
        .fetch_one(&pool)
        .await
        .expect("could not run query");

    assert_eq!(user.id, "user-1");
    assert_eq!(user.name, "bob");
    Ok(())
}

#[sqlx::test(fixtures("users"))]
async fn test_query_as_user(pool: PgPool) -> sqlx::Result<()> {
    let user: User = sqlx::query_as!(
        User,
        "SELECT id, name, email, details, status as \"status:UserStatus\" FROM users LIMIT 1;"
    )
    .fetch_one(&pool)
    .await
    .expect("Could not fetch one");

    assert_eq!(user.name, "bob");
    Ok(())
}

#[sqlx::test(fixtures("users"))]
async fn test_fetch_users_by_name(pool: PgPool) -> sqlx::Result<()> {
    let users = UserDbSet::many()
        .name_eq("bob".to_string())
        .fetch_all(&pool)
        .await
        .expect("Could not fetch users");

    assert_eq!(users.len(), 2);
    Ok(())
}

#[sqlx::test(fixtures("users"))]
async fn test_fetch_users_by_name_and_details(pool: PgPool) -> sqlx::Result<()> {
    let users = UserDbSet::many()
        .name_eq("bob".to_string())
        .details_eq("the best bob".to_string())
        .fetch_all(&pool)
        .await
        .expect("Could not fetch users");

    assert_eq!(users.len(), 1);
    Ok(())
}

#[sqlx::test(fixtures("users"))]
async fn test_fetch_all_users(pool: PgPool) -> sqlx::Result<()> {
    let users = UserDbSet::many()
        .fetch_all(&pool)
        .await
        .expect("Could not fetch users");

    assert_eq!(users.len(), 3);
    Ok(())
}

#[sqlx::test(fixtures("users"))]
async fn test_insert_users(pool: PgPool) -> sqlx::Result<()> {
    let inserted_user = UserDbSet::insert()
        .id("id-3".to_string())
        .email("steven@stevenson.com".to_string())
        .name("steven".to_string())
        .status(UserStatus::Verified)
        .insert(&pool)
        .await
        .expect("Could not insert");

    let matched_user = sqlx::query_as!(
        User,
        "SELECT id, name, email, details, status AS \"status:UserStatus\" FROM users WHERE id = 'id-3';"
    )
    .fetch_one(&pool)
    .await
    .expect("Could not fetch one");

    assert_eq!(matched_user.email, inserted_user.email);

    Ok(())
}

#[sqlx::test(fixtures("users"))]
async fn test_update_users(pool: PgPool) -> sqlx::Result<()> {
    let mut user = sqlx::query_as!(
        User,
        "SELECT id, name, email, details, status AS \"status:UserStatus\" FROM users WHERE id = 'user-2';"
    )
    .fetch_one(&pool)
    .await
    .expect("Could not fetch one");

    user.details = Some("Updated details!".to_string());
    user.email = String::from("mynewemail@bigpond.com.au");
    UserDbSet::update()
        .data(user.clone())
        .update(&pool)
        .await
        .expect("Could not update");

    let same_user_again = sqlx::query_as!(
        User,
        "SELECT id, name, email, details, status AS \"status:UserStatus\" FROM users WHERE id = 'user-2';"
    )
    .fetch_one(&pool)
    .await
    .expect("Could not fetch one");

    assert_eq!(user.email, same_user_again.email);
    assert_eq!(user.details, same_user_again.details);

    Ok(())
}

#[sqlx::test(fixtures("users"))]
async fn test_delete_users(pool: PgPool) -> sqlx::Result<()> {
    sqlx::query("INSERT INTO users (name, id, email, status) VALUES ('lana del ete', 'id-6', 'lana@bigpond.com.au', 'verified');")
        .execute(&pool)
        .await
        .expect("Could not initialise db");

    UserDbSet::delete()
        .id_eq("id-6".to_string())
        .delete(&pool)
        .await
        .expect("could not delete");

    let matched_user = sqlx::query_as!(
        User,
        "SELECT id, name, email, details, status as \"status:UserStatus\" FROM users WHERE id = 'id-6';"
    )
    .fetch_optional(&pool)
    .await
    .expect("Could not fetch one");

    assert!(matched_user.is_none());

    Ok(())
}

#[sqlx::test(fixtures("users"))]
async fn test_fixtures_loaded(pool: PgPool) -> sqlx::Result<()> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&pool)
        .await?;
    
    println!("Number of users in database: {}", count);
    assert_eq!(count, 3);
    Ok(())
}
