use super::snowflake::Snowflake;
use super::request::*;
use serde::{Serialize, Deserialize};

type Emoji = Snowflake;

#[derive(Deserialize,Serialize,Default)]
#[serde(default)]
pub struct Guild {
    id: Snowflake,
    name: String,
    icon: Option<String>,
    splash: Option<String>,
    discovery_splash: Option<String>,
    emojis: Vec<Emoji>,
    features: Vec<String>,
    approximate_member_count: u64,
    approximate_presence_count: u64,
    description: String
}

#[allow(dead_code)]
impl Guild {
    async fn get(env: Env, id: Snowflake) -> DiscordResponse<Self> {
        request(&GuildRequest::GetGuild{guild: id}, env).await
    }

    async fn update(env: Env, guild: Self) -> DiscordResponse<Self> {
        request(&GuildRequest::ModifyGuild{guild}, env).await
    }
    
    async fn delete(env: Env, guild: Self) -> DiscordResponse<()> {
        request(&GuildRequest::DeleteGuild{guild}, env).await
    }
}

#[allow(dead_code)]
enum GuildRequest {
    GetGuild {guild: Snowflake},
    ModifyGuild {guild: Guild},
    DeleteGuild {guild: Guild}
}

impl Requestable for GuildRequest {
    fn ratelimit_bucket(&self) -> String {
        match self {
            GuildRequest::GetGuild{guild} => format!("GET /guilds/{}", guild),
            GuildRequest::ModifyGuild{guild} => format!("PATCH /guilds/{}", guild.id),
            GuildRequest::DeleteGuild{guild} => format!("DELETE /guilds/{}", guild.id)
        }
    }
    fn build_request(&self) -> Request {
        match self {
            GuildRequest::GetGuild{guild} => Request::new(&build_uri!("/guilds/{}", guild), Method::Get),
            GuildRequest::ModifyGuild{guild} => Request::new_with_init(&build_uri!("/guilds/{}", guild.id), &RequestInit {
                body: Some(serde_wasm_bindgen::to_value(guild).unwrap()),
                method: Method::Patch,
                ..Default::default()
            }),
            GuildRequest::DeleteGuild{guild} => Request::new(&build_uri!("/guilds/{}", guild.id), Method::Delete)
        }.unwrap()
    }
}