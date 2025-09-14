#[macro_use]
extern crate rocket;

mod database;
mod routers;

use std::{env, path::{Path, PathBuf}};

use dotenv::dotenv;
use rocket::{fairing::{Fairing, Info, Kind}, fs::NamedFile, http::{Header, Status}, serde::json::Json, Request, Response, State};
use rocket_db_pools::sqlx::{self, FromRow, PgPool};
use serde::{Deserialize, Serialize};

pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to responses",
            kind: Kind::Response
        }
    }

    async fn on_response<'r>(&self, request: &'r Request<'_>, response: &mut Response<'r>) {
        let origin = request.headers().get_one("Origin");
        
        if let Some(origin) = origin {
            if origin.starts_with("http://localhost") || origin.starts_with("https://localhost") {
                response.set_header(Header::new("Access-Control-Allow-Origin", origin));
            }
        }
        
        response.set_header(Header::new("Access-Control-Allow-Methods", "POST, GET, PATCH, PUT, OPTIONS"));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

type DbPool = PgPool;

#[get("/")]
async fn index() -> Option<NamedFile> {
    NamedFile::open(Path::new("../webpage/dist/index.html")).await.ok()
}

#[options("/<_..>")]
fn all_options() -> Status {
    Status::Ok
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
        .attach(CORS)
        .manage(pool)
        .mount("/", routes![index, files, all_options, db, routers::users::users, routers::users::try_login, routers::users::signup])
}
