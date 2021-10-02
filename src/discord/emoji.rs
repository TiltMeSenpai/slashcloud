use super::snowflake::Snowflake;
use super::role::Role;
use super::user::User;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Emoji {
    id: Snowflake,
    name: String,
    #[cfg(feature = "role")]
    roles: Vec<Role>,
    #[cfg(feature = "user")]
    user: Option<User>
}