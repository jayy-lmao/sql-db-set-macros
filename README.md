## Name pending (currently DB sets)

Inpsired a little bit by Toasty rs and my work on sql-gen.

Can currently:
- [x] Query one
- [x] Query many
- [x] Insert (ignoring auto fields)


See the following: 

TODO:
- [ ] Limit / Offset
- [ ] Update
- [ ] Delete
- [ ] Allow for multiple-field primary keys for query-one
- [ ] create a version of https://github.com/jayy-lmao/sql-gen for generating these
- [ ] Figure out what I will do with query many multiple filed



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
let user = UserDbSet::one()
    .id_eq("user-1".to_string()) // type-state pattern, you must provide a key or unique field to be able to call fetch_one
    .fetch_one(pool)
    .await
    .expect("could not run query");


// Fetch all users
let users = UserDbSet::many()
    .fetch(pool) // Can call without setting fields to match to get all results
    .await
    .expect("Could not fetch users");


// Fetch many users with one field
    let users = UserDbSet::many()
        .name_eq("bob".to_string()) // Can set fields to match on
        .fetch(pool)
        .await
        .expect("Could not fetch users");

// Fetch many users with multiple fields
let users = UserDbSet::many()
    .name_eq("bob".to_string())
    .details_eq("the best bob".to_string()) // Can set multiple fields to match on
    .fetch(pool)
    .await
    .expect("Could not fetch users");

// Insert a user
let inserted_user = UserDbSet::insert()
    .id("id-3".to_string())
    .email("steven@stevenson.com".to_string())
    .name("steven".to_string())
    .insert(pool) // Due to type-state insert can't be called until all non-nullable (besides auto)  fields have been set
    .await
    .expect("Could not insert");
```


