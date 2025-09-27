use std::error::Error;

use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};

use crate::{DbPool, models::login_signup::SignupData};

pub async fn insert_user_into_db(
    pool: &DbPool,
    signup_data: &SignupData,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    perform_database_operations(pool, &signup_data).await
}

fn create_password_hash(signup_data: &SignupData) -> Result<String, Box<dyn Error + Send + Sync>> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(signup_data.password.as_bytes(), &salt)?
        .to_string();
    Ok(password_hash)
}

async fn perform_database_operations(
    pool: &DbPool,
    signup_data: &SignupData,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let password_hash = create_password_hash(&signup_data)?;
    sqlx::query!(
        "INSERT INTO users (username, password) VALUES ($1, $2)",
        &signup_data.username,
        &password_hash
    )
    .execute(pool)
    .await?;

    // You can only use query! with DDL
    sqlx::raw_sql(&format!(
        "CREATE USER \"{}\" WITH PASSWORD '{}';",
        &signup_data.username, &signup_data.password
    ))
    .execute(pool)
    .await?;

    sqlx::raw_sql(&format!(
        "GRANT housing_reader TO \"{}\";",
        &signup_data.username
    ))
    .execute(pool)
    .await?;

    Ok(())
}
