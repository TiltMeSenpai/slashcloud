extern crate serde;
use serde::{Serialize,Deserialize};
use serde::ser::{SerializeStruct, Serializer};
use serde_repr::{Deserialize_repr,Serialize_repr};
use serde_json::Value;

#[allow(dead_code)]
#[derive(Serialize_repr,PartialEq)]
#[repr(u8)]
pub enum ButtonStyle {
    Primary = 1,
    Secondary = 2,
    Success = 3,
    Danger = 4,
    Link = 5
}

#[derive(Serialize)]
pub struct ButtonEmoji{
    name: String,
    id: String,
    animated: bool
}

#[derive(Serialize)]
pub struct SelectOptions{
    pub label: String,
    pub value: String,
    pub descritpion: Option<String>,
    pub emoji: Option<ButtonEmoji>,
    pub default: Option<bool>
}

#[allow(dead_code)]
pub enum DiscordComponent {
    ActionRow{
        components: Vec<DiscordComponent>
    },
    Button{
        style: ButtonStyle,
        label: Option<String>,
        emoji: Option<ButtonEmoji>,
        value: String,
        disabled: Option<bool>
    },
    SelectMenu {
        custom_id: String,
        options: Vec<SelectOptions>,
        placeholder: Option<String>,
        min_values:  Option<u8>,
        max_values:  Option<u8>,
        disabled:    Option<bool>
    }
}

impl Serialize for DiscordComponent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer, {
        match self {
            DiscordComponent::ActionRow{components} => {
                let mut s = serializer.serialize_struct("ActionRow", 2)?;
                s.serialize_field("type", &1)?;
                s.serialize_field("component", &components)?;
                s.end()
            }
            DiscordComponent::Button{
                style,
                label,
                emoji,
                value,
                disabled} => {
                let mut s = serializer.serialize_struct("Button", 6)?;
                s.serialize_field("type", &2)?;
                s.serialize_field("style", &style)?;
                if style == &ButtonStyle::Link {
                    s.serialize_field("url", &value)?;
                } else {
                    s.serialize_field("custom_id", &value)?;
                };
                if let Some(l) = label {
                    s.serialize_field("label", &l)?;
                }
                if let Some(e) = emoji {
                    s.serialize_field("emoji", &e)?;
                }
                if let Some(d) = disabled {
                    s.serialize_field("disabled", &d)?;
                }
                s.end()
            }
            DiscordComponent::SelectMenu{
                custom_id,
                options,
                placeholder,
                min_values,
                max_values,
                disabled
            } => {
                let mut s = serializer.serialize_struct("SelectMenu", 8)?;
                s.serialize_field("type", &1)?;
                s.serialize_field("custom_id", &custom_id)?;
                s.serialize_field("options", &options)?;
                if let Some(p) = placeholder {
                    s.serialize_field("placeholder", &p)?;
                }
                if let Some(min) = min_values {
                    s.serialize_field("min_values", &min)?;
                }
                if let Some(max) = max_values {
                    s.serialize_field("max_values", &max)?;
                }
                if let Some(d) = disabled {
                    s.serialize_field("disabled", &d)?;
                }
                s.end()
            }
        }
    }
}

#[derive(Serialize)]
pub struct DiscordEmbed;

#[derive(Serialize, Default)]
pub struct InteractionResponseBody {
    pub tts: Option<bool>,
    pub content: Option<String>,
    pub embeds: Option<Vec<DiscordEmbed>>,
    pub flags: Option<u8>,
    pub components: Option<Vec<DiscordComponent>>
}

#[allow(dead_code)]
pub enum InteractionResponse {
    Pong,
    ChannelMessage { deferred: bool, body: InteractionResponseBody },
    UpdateMessage  { deferred: bool, body: InteractionResponseBody }
}

impl InteractionResponse {
    pub fn message() -> Self{
        InteractionResponse::ChannelMessage {
            deferred: false,
            body: InteractionResponseBody{..Default::default()}
        }
    }

    pub fn update() -> Self {
        InteractionResponse::UpdateMessage {
            deferred: false,
            body: InteractionResponseBody{..Default::default()}
        }
    }

    pub fn set_tts(mut self, val: bool) -> Self {
        match &mut self {
            InteractionResponse::Pong => (),
            InteractionResponse::ChannelMessage{body, ..} |
            InteractionResponse::UpdateMessage{body, ..} => body.tts = Some(val)
        }
        self
    }

    pub fn set_content(mut self, val: String) -> Self {
        match &mut self {
            InteractionResponse::Pong => (),
            InteractionResponse::ChannelMessage{body, ..} |
            InteractionResponse::UpdateMessage{body, ..} => body.content = Some(val)
        }
        self
    }

    pub fn add_embed(mut self, val: DiscordEmbed) -> Self {
        match &mut self {
            InteractionResponse::Pong => (),
            InteractionResponse::ChannelMessage{body, ..} |
            InteractionResponse::UpdateMessage{body, ..} => match &mut body.embeds {
                Some(embeds) => embeds.push(val),
                None => body.embeds = Some(vec!(val))
            }
        }
        self
    }

    pub fn set_flag(mut self, flag: u8) -> Self {
        match &mut self {
            InteractionResponse::Pong => (),
            InteractionResponse::ChannelMessage{body, ..} |
            InteractionResponse::UpdateMessage{body, ..} => match body.flags {
                Some(flags) => body.flags = Some(flags | 1 << flag),
                None => body.flags = Some(1 << flag)
            }
        }
        self
    }

    pub fn add_component(mut self, val: DiscordComponent) -> Self {
        match &mut self {
            InteractionResponse::Pong => (),
            InteractionResponse::ChannelMessage{body, ..} |
            InteractionResponse::UpdateMessage{body, ..} => match &mut body.components {
                Some(components) => components.push(val),
                None => body.components = Some(vec!(val))
            }
        }
        self
    }
}


impl Serialize for InteractionResponse {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer, {
        let mut s = serializer.serialize_struct("InteractionResponse", 2)?;
        match self {
            InteractionResponse::Pong => {
                s.serialize_field("type", &1)?;
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

#[repr(u8)]
#[derive(Deserialize_repr,Debug,Clone)]
pub enum InteractionRequestType {
    Ping = 1,
    ApplicationCommand = 2,
    MessageComponent = 3
}

#[allow(dead_code)]
#[derive(Deserialize,Debug,Clone)]
pub struct InteractionRequest {
    #[serde(rename="type")]
    pub t: InteractionRequestType,
    pub id: String,
    pub application_id: Option<String>,
    pub guild_id: Option<String>,
    pub channel_id: Option<String>,
    #[serde(alias="member")]
    pub user: Option<Value>,
    pub data: Option<Value>,
    pub token: String,
    pub message: Option<Value>
}