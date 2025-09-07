use std::path::Path;

use rocket::fs::NamedFile;

#[macro_use] extern crate rocket;

#[get("/")]
fn index() -> &'static str{
    "Hello world!"
}

#[get("/iframe")]
async fn iframe() -> Option<NamedFile> {
    NamedFile::open(Path::new("html/pgadmin_iframe.html")).await.ok()
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index, iframe])
}
