use db_set_macros::DbSet;
use sqlx::{postgres::PgPoolOptions, QueryBuilder};

#[derive(DbSet)]
#[dbset(table_name = "users")]
pub struct User {
    #[key]
    id: String,
    name: String,
    details: Option<String>,
    #[unique]
    email: String,
}

#[tokio::test]
async fn simple_query() -> Result<(), String> {
    let docker = testcontainers::clients::Cli::default();
    let container = docker.run(testcontainers_modules::postgres::Postgres::default());
    let connection_string = &format!(
        "postgres://postgres:postgres@127.0.0.1:{}/postgres",
        container.get_host_port_ipv4(5432)
    );

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(connection_string)
        .await
        .expect("Could not connect to postgres");

    sqlx::raw_sql(
        "create table users (name text not null, id text not null, email text not null, details text);",
    )
    .execute(&pool)
    .await
    .expect("Could not initialise db");

    sqlx::raw_sql("insert into users (name, id, email) values ('bob', 'user-1', 'bob@bob.com');")
        .execute(&pool)
        .await
        .expect("Could not initialise db");

    let user = UserDbSet::by_id(&pool, "user-1".to_string())
        .await
        .expect("Could not query user");

    // Still allowed to make query_as
    sqlx::query_as!(User, "SELECT id,name,email,details FROM users LIMIT 1;")
        .fetch_one(&pool)
        .await
        .expect("Could not fetch one");

    sqlx::raw_sql("insert into users (name, id, email, details) values ('bob', 'user-2', 'bobo@bob.com', 'the best bob');")
        .execute(&pool)
        .await
        .expect("Could not initialise db");

    // let query = UserDbSetQueryBuilder::new().name_eq("bob".to_string());
    // let users = query.fetch(&pool).await.expect("could not fetch users");
    let users = UserDbSet::many()
        .name_eq("bob".to_string())
        .fetch(&pool)
        .await
        .expect("could not fetch users");

    assert_eq!(users.len(), 2);

    let users_b = UserDbSet::many()
        .name_eq("bob".to_string())
        .details_eq("the best bob".to_string())
        .fetch(&pool)
        .await
        .expect("could not fetch users");

    assert_eq!(users.len(), 2);
    Ok(())
}
