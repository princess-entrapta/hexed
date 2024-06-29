use std::env;

use argon2::{Argon2, PasswordHasher};
use password_hash::SaltString;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};

#[tokio::main]
async fn main() {
    let pool = SqlitePool::connect_with(
        SqliteConnectOptions::new()
            .filename("example.sqlite")
            .create_if_missing(true),
    )
    .await
    .unwrap();
    let args: Vec<String> = env::args().collect();
    let salt = SaltString::generate(&mut argon2::password_hash::rand_core::OsRng);

    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(&args[2].as_bytes(), &salt)
        .unwrap()
        .to_string();
    // Create a table if it does not exist
    let res = sqlx::query(&std::fs::read_to_string("initdb.sql").unwrap())
        .bind(args[1].clone())
        .bind(password_hash)
        .execute(&pool)
        .await;
    if res.is_err() {
        print!("{}", res.unwrap_err().to_string());
    }
}
