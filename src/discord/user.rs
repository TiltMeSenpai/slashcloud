use serde::{Serialize, Deserialize};
use super::snowflake::Snowflake;

#[derive(Serialize, Deserialize, Default)]
pub struct User {
    id: Snowflake,
    username: String,
    discriminatior: String,
    avatar: Option<String>,
    bot: bool,
    system: bool,
    mfa_enabled: bool,
    banner: Option<String>,
    accent_color: Option<u32>,
    locale: String,
    flags: u32,
    premium_type: u32,
    public_flags: u32
}