use db_set_macros::DbSet;
use sqlx::postgres::PgPoolOptions;

#[derive(DbSet)]
#[dbset(table_name = "users")]
pub struct User {
    #[key]
    id: String,
    name: String,
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
        "create table users (name text not null, id text not null, email text not null);",
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
    sqlx::query_as!(User, "SELECT * FROM users LIMIT 1;")
        .fetch_one(&pool)
        .await
        .expect("Could not fetch one");
    Ok(())
}
