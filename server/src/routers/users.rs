use std::error::Error;

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

use crate::{database::users::{User, UserResponse}, DbPool};


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

#[get("/users")]
pub async fn users(pool: &State<DbPool>) -> Json<Vec<UserResponse>> {
    let users = sqlx::query_as::<_, User>("select * from users")
        .fetch_all(pool.inner())
        .await
        .unwrap_or_else(|_| vec![]);
    Json(users.iter().map(|u| u.into()).collect())
}


#[post("/api/login", data = "<login_data>")]
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

#[post("/api/signup", data = "<signup_data>")]
pub async fn signup(
    signup_data: Json<SignupData>,
    pool: &State<DbPool>,
) -> Result<Json<UserResponse>, Status> {
    let _var_name = if !is_valid_email(&signup_data.username) {
        return Err(Status::BadRequest);
    };

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

    let insert_result = insert_user_into_db(pool.inner(), &signup_data).await;

    match insert_result {
        Ok(_) => {
            let new_user = UserResponse {
                username: signup_data.username.clone(),
            };
            Ok(Json(new_user))
        }
        Err(e) => {
            eprintln!("Error inserting user: {e}");
            Err(Status::ServiceUnavailable)
        }
    }
}

async fn insert_user_into_db(
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
        "CREATE USER {} WITH PASSWORD '{}';",
        &signup_data.username, &signup_data.password
    ))
    .execute(pool)
    .await?;

    sqlx::raw_sql(&format!(
        "GRANT housing_reader TO {};",
        &signup_data.username
    ))
    .execute(pool)
    .await?;

    Ok(())
}
