#[macro_use]
extern crate rocket;

mod database;
mod routers;

use std::{env, path::{Path, PathBuf}};

use dotenv::dotenv;
use rocket::{fs::NamedFile, serde::json::Json, State};
use rocket_db_pools::sqlx::{self, FromRow, PgPool};
use serde::{Deserialize, Serialize};

type DbPool = PgPool;

#[get("/")]
async fn index() -> Option<NamedFile> {
    NamedFile::open(Path::new("../webpage/dist/index.html")).await.ok()
}


#[get("/<file..>")]
async fn files(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("../webpage/dist/").join(file)).await.ok()
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct Zoning {
    zoning: String,
    r#use: String,
}

#[get("/db")]
async fn db(pool: &State<DbPool>) -> Json<Vec<Zoning>> {
    let zoning = sqlx::query_as::<_, Zoning>("Select * from zoning")
        .fetch_all(pool.inner())
        .await
        .unwrap_or_else(|_| vec![]);
    Json(zoning)
}

#[launch]
async fn rocket() -> _ {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set");
    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to create database pool");
    rocket::build()
        .manage(pool)
        .mount("/", routes![index, files, db, routers::users::users, routers::users::try_login])
}
