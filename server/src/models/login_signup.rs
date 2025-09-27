use serde::Deserialize;

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct LoginData {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct SignupData {
    pub username: String,
    pub password: String,
}
