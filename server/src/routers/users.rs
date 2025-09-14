use rocket::{http::Status, serde::json::Json, State};
use serde::Deserialize;
use argon2::{Argon2, password_hash::{rand_core::OsRng, PasswordHasher, SaltString}};

use crate::{DbPool, database::users::User};

#[get("/users")]
pub async fn users(pool: &State<DbPool>) -> Json<Vec<User>> {
    let users = sqlx::query_as::<_, User>("select * from users")
        .fetch_all(pool.inner())
        .await
        .unwrap_or_else(|_| vec![]);
    Json(users)
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct LoginData {
    username: String,
    password: String,
}

#[post("/login", data = "<login_data>")]
pub async fn try_login(login_data: Json<LoginData>, pool: &State<DbPool>) -> Result<(), Status> {
    let password_query = sqlx::query_as!(User, "Select * from users where username = $1", &login_data.username)
        .fetch_optional(pool.inner()).await;
    match password_query {
        Ok(user_option) => {
            match user_option {
                Some(user) => {
                    match user.verify_password(&login_data.password){
                        Ok(()) => Ok(()),
                        Err(e) => {
                            eprintln!("Error authorizing password: {e}");
                            Err(Status::Unauthorized)
                        }
                    }
                }
                None => {
                    eprintln!("No matching username");
                    Err(Status::Unauthorized)
                }
            }
        }
        Err(e) => {
            eprintln!("Error: {e}");
            Err(Status::ServiceUnavailable)
        },
    }
}
