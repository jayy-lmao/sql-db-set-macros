## Name pending (currently DB sets)

Inpsired a little bit by Toasty rs and my work on sql-gen.
Idea is to implement the most common SQLX queries, but allow you to still ultimately use SQLX for anything more complex than a basic query.

Currently it's only for PostgresQL, so that I can dogfood. But PRs welcome.

### Why not X

**_Why not SQLX_?**
I _love_ sqlx. It's a delight to work with. I like writing SQL.
That's why _this is sqlx_. This macro allows me to continue writing SQLX, but just shaves off some `sqlx prepare` cycles, and some boring same queries you write over and over again.
Ultimately I still just want to write SQLX, but just save myself all the `by id` style queries.


*Why not SeaORM?*
I've used SeaORM before at a job. I find it verbose.

*Why not Diesel?*
I like the look of diesel. I don't like dedicated single-use config filetypes.
Might end up having to write one though for my codegen tool, so maybe I'll eat those words.

*Why not Toasy?*
Same as diesel. And it's not out yet. But I must like the look of it a little, as I took inspiration from its model definitions.

### Current features

Can currently:
- [x] Query one
- [x] Query many
- [x] Insert (ignoring auto fields)
- [x] Update
- [x] Delete


### Roadmap

TODO:
- [x] Allow for multiple-field primary keys for query-one
- [x] Allow query many by one key field when there are two key fields
- [x] Update
- [x] Delete
- [~] Release early version!
- [ ] Limit / Offset
- [ ] Create a version of https://github.com/jayy-lmao/sql-gen for generating these
- [ ] Do more than just `eq` to match fields (map of ops for each type)

### Examples

```rs
#[derive(DbSet, Debug)] // DbSet also implements sqlx::FromRow by default
#[dbset(table_name = "users")] // Used for queries, will be used for codegen
pub struct User {
    #[key] // For `::one` queries, can add `auto` to have it ignored as required for inserts.
    id: String,
    name: String, // Will be required for insert
    details: Option<String>, // wont be required for insert
    #[unique]
    email: String, // Will generate `::one` queries as it's unique
}

// Fetch one user
let user: User = UserDbSet::one()
    .id_eq("user-1".to_string()) // type-state pattern, you must provide a key or unique field to be able to call fetch_one
    .fetch_one(pool)
    .await?;

let user_maybe: Option<User> = UserDbSet::one()
    .id_eq("user-1".to_string()) // type-state pattern, you must provide a key or unique field to be able to call fetch_one
    .fetch_optional(pool)
    .await?;

// We can also just write regular SQLX queries.
// DbSet implements FromRow for your struct also.
let same_user_again = sqlx::query_as!(
    User,
    "SELECT id, name, email, details FROM users WHERE id = 'user-1';"
)
.fetch_one(pool)
.await?;

// Fetch all users
let users = UserDbSet::many()
    .fetch_all(pool) // Can call without setting fields to match to get all results
    .await?;

// Fetch many users with one field
    let users = UserDbSet::many()
        .name_eq("bob".to_string()) // Can set fields to match on
        .fetch_all(pool)
        .await?;

// Fetch many users with multiple fields
let users = UserDbSet::many()
    .name_eq("bob".to_string())
    .details_eq("the best bob".to_string()) // Can set multiple fields to match on
    .fetch_all(pool)
    .await?;

// Insert a user
let inserted_user = UserDbSet::insert()
    .id("id-3".to_string())
    .email("steven@stevenson.com".to_string())
    .name("steven".to_string())
    .insert(pool) // Due to type-state insert can't be called until all non-nullable (besides auto)  fields have been set
    .await?;

// Update a user
user.details = Some("Updated details!".to_string());
user.email = String::from("mynewemail@bigpond.com.au");
UserDbSet::update()
    .data(user.clone())
    .update(pool)
    .await?;

// Delete a user
UserDbSet::one()
    .id_eq("user-1".to_string()) // type-state pattern, you must provide a key or unique field to be able to call fetch_one
    .delete(pool)
    .await?;
```


