use serde::{Serialize, Deserialize};
use std::collections::HashMap;

use super::snowflake::Snowflake;
use super::request::*;

#[cfg(feature = "emoji")]
use super::emoji::Emoji;

use crate::WorkerEnv;

#[derive(Deserialize,Serialize,Default)]
#[serde(default)]
pub struct Guild {
    id: Snowflake,
    name: String,
    icon: Option<String>,
    splash: Option<String>,
    discovery_splash: Option<String>,
    #[cfg(feature = "emoji")]
    emojis: Vec<Emoji>,
    features: Vec<String>,
    approximate_member_count: u64,
    approximate_presence_count: u64,
    description: String,
    #[serde(flatten, skip_serializing)]
    extra: HashMap<String, serde_json::Value>
}

#[allow(dead_code)]
impl Guild {
    pub async fn get(env: &WorkerEnv, id: Snowflake, with_counts: bool) -> DiscordResponse<Self> {
        request(&GuildRequest::GetGuild{guild: id, with_counts}, env).await
    }

    pub async fn update(env: &WorkerEnv, guild: Self) -> DiscordResponse<Self> {
        request(&GuildRequest::ModifyGuild{guild}, env).await
    }
    
    pub async fn delete(env: &WorkerEnv, guild: Self) -> DiscordResponse<()> {
        request(&GuildRequest::DeleteGuild{guild}, env).await
    }
}

#[allow(dead_code)]
enum GuildRequest {
    GetGuild {guild: Snowflake, with_counts: bool},
    ModifyGuild {guild: Guild},
    DeleteGuild {guild: Guild}
}

impl Requestable for GuildRequest {
    fn ratelimit_bucket(&self) -> String {
        match self {
            GuildRequest::GetGuild{guild, ..} => format!("GET /guilds/{}", guild),
            GuildRequest::ModifyGuild{guild} => format!("PATCH /guilds/{}", guild.id),
            GuildRequest::DeleteGuild{guild} => format!("DELETE /guilds/{}", guild.id)
        }
    }
    fn build_request(&self) -> Request {
        match self {
            GuildRequest::GetGuild{guild, with_counts} => Request::new(&build_uri!("/guilds/{}?with_counts={}", guild, with_counts), Method::Get),
            GuildRequest::ModifyGuild{guild} => Request::new_with_init(&build_uri!("/guilds/{}", guild.id), &RequestInit {
                body: Some(serde_wasm_bindgen::to_value(guild).unwrap()),
                method: Method::Patch,
                ..Default::default()
            }),
            GuildRequest::DeleteGuild{guild} => Request::new(&build_uri!("/guilds/{}", guild.id), Method::Delete)
        }.unwrap()
    }
}