use super::{Snowflake, DiscordResponse};
use super::request::*;
use serde::{Serialize, Deserialize};
use worker::{Request, Env};

#[derive(Serialize, Default)]
pub struct AllowedMentions {
    parse: Vec<String>,
    roles: Vec<Snowflake>,
    users: Vec<Snowflake>
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Message {
    pub id: Snowflake,
    #[serde(skip_serializing)]
    pub channel_id: Snowflake,
    #[serde(skip_serializing)]
    pub guild_id: Snowflake,
    pub tts: bool,
    pub content: String,
    #[serde(skip_deserializing)]
    pub allowed_mentions: AllowedMentions
}

impl Message {
    pub async fn list(env: Env, channel: Snowflake) -> DiscordResponse<Vec<Message>> {
        request(&MessageRequest::GetMessages(channel), env).await
    }

    pub async fn create(env: Env, message: Message) -> DiscordResponse<Message> {
        request(&MessageRequest::CreateMessage(message), env).await
    }

    pub async fn update(env: Env, message: Message) -> DiscordResponse<Message> {
        request(&MessageRequest::UpdateMessage(message), env).await
    }
    
    pub async fn delete(env: Env, message: Message) -> DiscordResponse<()> {
        request(&MessageRequest::DeleteMessage(message), env).await
    }
}

enum MessageRequest {
    GetMessages(Snowflake),
    CreateMessage(Message),
    UpdateMessage(Message),
    DeleteMessage(Message)
}

impl Requestable for MessageRequest {
    fn ratelimit_bucket(&self) -> String {
        match self {
            MessageRequest::GetMessages(channel) => format!("GET /channels/{}/messages", channel),
            MessageRequest::CreateMessage(msg) => format!("POST /channels/{}/messages", msg.channel_id),
            MessageRequest::UpdateMessage(msg) => format!("PATCH /channels/{}/messages", msg.channel_id),
            MessageRequest::DeleteMessage(msg) => format!("DELETE /channels/{}/messages", msg.channel_id)
        }
    }

    fn build_request(&self) -> Request {
        match self {
            MessageRequest::GetMessages(channel) => 
                build_request!(
                    Get ["/channels/{}/messages", channel]
                ),
            MessageRequest::CreateMessage(msg) => 
                build_request!(
                    Post ["/channels/{}/messages", msg.channel_id],
                    msg
                ),
            MessageRequest::UpdateMessage(msg) => 
                build_request!(
                    Patch ["/channels/{}/messages", msg.channel_id],
                    msg
                ),
            MessageRequest::DeleteMessage(msg) => 
                build_request!(
                    Delete  ["/channels/{}/messages/{}", msg.channel_id, msg.id]
                )
        }
    }
}