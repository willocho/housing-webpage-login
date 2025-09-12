use rocket::{State, serde::json::Json};

use crate::{DbPool, database::users::User};

#[get("/users")]
pub async fn users(pool: &State<DbPool>) -> Json<Vec<User>> {
    let users = sqlx::query_as::<_, User>("select * from users")
        .fetch_all(pool.inner())
        .await
        .unwrap_or_else(|_| vec![]);
    Json(users)
}
