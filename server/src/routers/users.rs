use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use rocket::{
    State,
    http::{Cookie, CookieJar, SameSite, Status},
    serde::json::Json,
};
use serde::Deserialize;
use uuid::Uuid;

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

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct SignupData {
    username: String,
    password: String,
}

fn is_valid_email(email: &str) -> bool {
    email.contains('@')
        && email.chars().filter(|&c| c == '@').count() == 1
        && email.split('@').nth(1).map_or(false, |domain| {
            domain.contains('.')
                && domain.len() > 3
                && !domain.starts_with('.')
                && !domain.ends_with('.')
        })
        && !email.starts_with('@')
        && !email.ends_with('@')
        && email.len() > 5
}

#[post("/login", data = "<login_data>")]
pub async fn try_login(
    login_data: Json<LoginData>,
    pool: &State<DbPool>,
    cookies: &CookieJar<'_>,
) -> Result<(), Status> {
    let password_query = sqlx::query_as!(
        User,
        "Select * from users where username = $1",
        &login_data.username
    )
    .fetch_optional(pool.inner())
    .await;
    match password_query {
        Ok(user_option) => match user_option {
            Some(user) => match user.verify_password(&login_data.password) {
                Ok(()) => {
                    let session_id = Uuid::new_v4().to_string();
                    let mut cookie = Cookie::new("session_id", session_id);
                    cookie.set_same_site(SameSite::Lax);
                    cookie.set_http_only(true);
                    cookie.set_path("/");
                    cookies.add(cookie);
                    Ok(())
                }
                Err(e) => {
                    eprintln!("Error authorizing password: {e}");
                    Err(Status::Unauthorized)
                }
            },
            None => {
                eprintln!("No matching username");
                Err(Status::Unauthorized)
            }
        },
        Err(e) => {
            eprintln!("Error: {e}");
            Err(Status::ServiceUnavailable)
        }
    }
}

#[post("/signup", data = "<signup_data>")]
pub async fn signup(
    signup_data: Json<SignupData>,
    pool: &State<DbPool>,
) -> Result<Json<User>, Status> {
    if !is_valid_email(&signup_data.username) {
        return Err(Status::BadRequest);
    }

    let existing_user = sqlx::query_as!(
        User,
        "Select * from users where username = $1",
        &signup_data.username
    )
    .fetch_optional(pool.inner())
    .await;

    match existing_user {
        Ok(Some(_)) => return Err(Status::Conflict),
        Ok(None) => {}
        Err(_) => return Err(Status::ServiceUnavailable),
    }

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(signup_data.password.as_bytes(), &salt)
        .map_err(|_| Status::InternalServerError)?
        .to_string();

    let insert_result = sqlx::query!(
        "INSERT INTO users (username, password) VALUES ($1, $2)",
        &signup_data.username,
        &password_hash
    )
    .execute(pool.inner())
    .await;

    match insert_result {
        Ok(_) => {
            let new_user = User {
                username: signup_data.username.clone(),
                password: password_hash.into(),
            };
            Ok(Json(new_user))
        }
        Err(e) => {
            eprintln!("Error inserting user: {e}");
            Err(Status::ServiceUnavailable)
        }
    }
}
