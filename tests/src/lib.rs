use std::sync::LazyLock;

use db_set_macros::DbSet;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use testcontainers::{clients::Cli, Container};
use tokio::sync::OnceCell;

#[derive(DbSet, Debug, Clone)]
#[dbset(table_name = "users")]
pub struct User {
    #[key]
    id: String,
    name: String,
    details: Option<String>,
    #[unique]
    email: String,
}

type TestPostgres = testcontainers_modules::postgres::Postgres;

// LazyLock for testcontainers::Cli, created once and shared
static DOCKER_CLI: LazyLock<Cli> = LazyLock::new(Cli::default);

// Global LazyLock holding both the container and database pool.
static DB_RESOURCES: LazyLock<OnceCell<(Container<'static, TestPostgres>, Pool<Postgres>)>> =
    LazyLock::new(OnceCell::new);

async fn prepare_db() -> (Container<'static, TestPostgres>, Pool<Postgres>) {
    let container = DOCKER_CLI.run(TestPostgres::default());
    let connection_string = format!(
        "postgres://postgres:postgres@127.0.0.1:{}/postgres",
        container.get_host_port_ipv4(5432)
    );

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&connection_string)
        .await
        .expect("Could not connect to postgres");

    sqlx::query("CREATE TABLE IF NOT EXISTS users (name text not null, id text not null, email text not null, details text);")
        .execute(&pool)
        .await
        .expect("Could not initialise db");

    sqlx::query("INSERT INTO users (name, id, email) VALUES ('bob', 'user-1', 'bob@bob.com');")
        .execute(&pool)
        .await
        .expect("Could not initialise db");

    sqlx::query("INSERT INTO users (name, id, email, details) VALUES ('bob', 'user-2', 'bobo@bob.com', 'the best bob');")
        .execute(&pool)
        .await
        .expect("Could not initialise db");

    sqlx::query(
        "INSERT INTO users (name, id, email) VALUES ('alice', 'user-3', 'alice@alice.com');",
    )
    .execute(&pool)
    .await
    .expect("Could not initialise db");

    (container, pool)
}

async fn get_db_pool() -> &'static Pool<Postgres> {
    let (_, pool) = DB_RESOURCES.get_or_init(prepare_db).await;
    pool
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

    let user: User = sqlx::query_as!(User, "SELECT id, name, email, details FROM users LIMIT 1;")
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

// Doesn't work in CI but works locally
#[ignore]
#[tokio::test]
async fn test_insert_users() -> Result<(), String> {
    let pool = get_db_pool().await;

    let inserted_user = UserDbSet::insert()
        .id("id-3".to_string())
        .email("steven@stevenson.com".to_string())
        .name("steven".to_string())
        .insert(pool)
        .await
        .expect("Could not insert");

    let matched_user = sqlx::query_as!(
        User,
        "SELECT id, name, email, details FROM users WHERE id = 'id-3';"
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
        "SELECT id, name, email, details FROM users WHERE id = 'user-2';"
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
        "SELECT id, name, email, details FROM users WHERE id = 'user-2';"
    )
    .fetch_one(pool)
    .await
    .expect("Could not fetch one");

    assert_eq!(user.email, same_user_again.email);
    assert_eq!(user.details, same_user_again.details);

    Ok(())
}
