use std::error::Error;

use argon2::{Argon2, PasswordHash, PasswordVerifier};
use rocket_db_pools::sqlx::FromRow;
use serde::{Deserialize, Serialize};
use sqlx::prelude::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[sqlx(transparent)]
struct HashedPassword(String);

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct User {
    username: String,
    password: HashedPassword,
}

impl User {
    pub fn verify_password(&self, password: &String) -> Result<(), Box<dyn Error>> {
        /*
         * Verifies that the string passed in matches the users password
         **/
        let parsed_hash: PasswordHash<'_> = PasswordHash::new(&self.password.0)?;
        let argon2 = Argon2::default();
        argon2.verify_password(&password.as_bytes(), &parsed_hash)?;
        Ok(())
    }
}
