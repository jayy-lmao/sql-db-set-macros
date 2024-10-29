use db_set_macros::DbSet;
use sqlx::postgres::PgPoolOptions;

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

// #[derive(DbSet)]
// #[dbset(table_name = "todos")]
// pub struct Todo {
//     #[unique]
//     id: String,
//     user_id: String,
// }

#[tokio::main]
async fn main() {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://postgres:password@localhost/test")
        .await
        .expect("Could not connect to postgres");

    let user = UserDbSet::by_id(&pool, "user-1".to_string())
        .await
        .expect("Could not query user");
    // println!("Hello {}!", user.name)
}
