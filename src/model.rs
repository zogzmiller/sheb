use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct User {
    pub first_name: String,
    pub last_name: String,
    pub username: String,
    pub password: String,
    pub email: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct UserLogin {
    pub username: String,
    pub password: String,
}