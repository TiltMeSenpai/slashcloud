use serde_json::from_str;
use worker::*;

#[allow(unused_imports)]
use types::command::*;
#[allow(unused_imports)]
use types::interactions::*;

mod types;
mod utils;
pub mod discord;
pub use types::*;
pub use discord::DiscordResponse;

#[macro_use]
#[allow(unused_imports)]
extern crate macros;

pub use async_trait::async_trait;

pub use command::*;
pub use interactions::*;
pub use serde_json::json;
pub use std::iter::FromIterator;

#[cfg(not(target_arch = "wasm32"))]
pub fn gen_bulk_command_json<T>()
where
    T: CommandOption,
{
    use std::fs::*;
    let val = T::to_value();
    let _f = write("commands.json", serde_json::to_vec(&val).unwrap());
}

#[cfg(not(target_arch = "wasm32"))]
pub fn gen_command_json<T>()
where
    T: CommandOption,
{
    use std::fs::*;
    let _dir = create_dir("commands");
    T::to_value()
        .as_array()
        .unwrap()
        .iter()
        .for_each(|command| {
            let name = format!("commands/{}", command["name"].as_str().unwrap());
            println!("Writing {}", name);
            let _f = write(name, serde_json::to_vec(command).unwrap());
        })
}

pub async fn handle_request<T, C, R>(mut req: Request, env: Env) -> Result<Response>
where
    T: CommandOption,
    C: CommandHandler<T>,
    R: InteractionHandler<T>
{
    let body = req.text().await.unwrap();
    let ctx = utils::JsCtx::new();
    let key = match ctx.get_key(&env).await {
        Ok(key) => key,
        Err(msg) => return worker::Response::error(msg, 500),
    };
    match ctx.verify_request(&key, &req, body.as_bytes()).await {
        Err(msg) => return worker::Response::error(msg, 401),
        Ok(()) => {
            let json: InteractionRequest = match from_str(&body) {
                Ok(res) => res,
                Err(msg) => {
                    console_log!("Error deserializing: {:?}", msg);
                    return worker::Response::error("Malformed payload", 400);
                }
            };
            console_log!("Payload: {:?}", json);
            match json {
                InteractionRequest {
                    t: InteractionRequestType::Ping,
                    ..
                } => worker::Response::from_json(&InteractionResponse::Pong),
                req
                @
                InteractionRequest {
                    t: InteractionRequestType::ApplicationCommand,
                    ..
                } => match req.data.to_owned() {
                    Some(arg_val) => match T::from_value(&arg_val) {
                        Some(args) => {
                            worker::Response::from_json(&C::handle_command(env, args, req).await)
                        },
                        None => {
                            console_log!("Error deserializing args. JSON: {:?}", &arg_val);
                            worker::Response::error("Could not deserialize args", 400)
                        }
                    },
                    None => worker::Response::error("Missing args", 400),
                },
                _ => worker::Response::error("Request type not recognized", 404),
            }
        }
    }
}
