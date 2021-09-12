use serde::ser::{SerializeStruct, Serializer};
use serde::Serialize;

#[derive(Serialize)]
pub struct InteractionResponseBody;

pub enum InteractionResponse {
    Pong,
    ChannelMessage { deferred: bool, body: InteractionResponseBody },
    UpdateMessage  { deferred: bool, body: InteractionResponseBody }
}


impl Serialize for InteractionResponse {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer, {
        let mut s = serializer.serialize_struct("InteractionResponse", 2)?;
        match self {
            InteractionResponse::Pong => {
                s.serialize_field("type", &1)?;
                s.serialize_field("data", &false)?;
            },
            InteractionResponse::ChannelMessage{deferred: false, body} => {
                s.serialize_field("type", &4)?;
                s.serialize_field("data", &body)?;
            },
            InteractionResponse::ChannelMessage{deferred: true, body} => {
                s.serialize_field("type", &5)?;
                s.serialize_field("data", &body)?;
            },
            InteractionResponse::UpdateMessage{deferred: false, body} => {
                s.serialize_field("type", &6)?;
                s.serialize_field("data", &body)?;
            },
            InteractionResponse::UpdateMessage{deferred: true, body} => {
                s.serialize_field("type", &7)?;
                s.serialize_field("data", &body)?;
            },
        }
        s.end()
    }
}

