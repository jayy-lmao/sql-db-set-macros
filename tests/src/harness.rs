use std::{sync::LazyLock, time::Duration};

use db_set_macros::DbSet;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use testcontainers::{clients::Cli, Container};
use tokio::sync::OnceCell;

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
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(60)) // Set your preferred timeout
        .connect(&connection_string)
        .await
        .expect("Could not connect to postgres");

    sqlx::query(
        r#"
            DO $$
            BEGIN
              IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'user_status') THEN
                CREATE TYPE user_status AS ENUM ('verified', 'unverified');
              END IF;
            END
            $$;
    "#,
    )
    .execute(&pool)
    .await
    .expect("Could not initialise db");
    sqlx::query("CREATE TABLE IF NOT EXISTS users (name text not null, id text not null, email text not null, details text, status user_status not null);")
        .execute(&pool)
        .await
        .expect("Could not initialise db");

    sqlx::query(
        "INSERT INTO users (name, id, email) VALUES ('bob', 'user-1', 'bob@bob.com', 'verified');",
    )
    .execute(&pool)
    .await
    .expect("Could not initialise db");

    sqlx::query("INSERT INTO users (name, id, email, details) VALUES ('bob', 'user-2', 'bobo@bob.com', 'the best bob', 'unverified');")
        .execute(&pool)
        .await
        .expect("Could not initialise db");

    sqlx::query(
        "INSERT INTO users (name, id, email) VALUES ('alice', 'user-3', 'alice@alice.com', 'verified');",
    )
    .execute(&pool)
    .await
    .expect("Could not initialise db");

    (container, pool)
}

pub async fn get_db_pool() -> &'static Pool<Postgres> {
    let (_, pool) = DB_RESOURCES.get_or_init(prepare_db).await;
    pool
}
