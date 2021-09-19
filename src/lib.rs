use worker::*;
use serde_json::from_str;

use types::interactions::*;
use types::command::*;

mod utils;
mod types;
pub use types::*;

#[macro_use]
#[allow(unused_imports)]
extern crate macros;

pub use crate::command::CommandOption;
pub use serde_json::json;
pub use std::iter::FromIterator;

pub async fn handle_request<T>(mut req: Request, env: Env) -> Result<Response> where T: CommandOption + CommandHandler {
    let body = req.text().await.unwrap();
    let ctx = utils::JsCtx::new();
    let key = match ctx.get_key(&env).await{
        Ok(key) => key,
        Err(msg) => return worker::Response::error(msg, 500)
    };
    match ctx.verify_request(&key, &req, body.as_bytes()).await {
        Err(msg) => return worker::Response::error(msg, 401),
        Ok(()) => {
            let json: InteractionRequest = match from_str(&body){
                Ok(res)  => res,
                Err(msg) => {
                    console_log!("Error deserializing: {:?}", msg);
                    return worker::Response::error("Malformed payload", 400)
                }
            };
            console_log!("Payload: {:?}", json);
            match json {
                InteractionRequest{t: InteractionRequestType::Ping, ..} => worker::Response::from_json(&InteractionResponse::Pong),
                req @ InteractionRequest{t: InteractionRequestType::ApplicationCommand, ..} => {
                    match req.data.to_owned() {
                        Some(arg_val) => match T::from_value(&arg_val) {
                            Some(args) => worker::Response::from_json(&T::handle(args, req)),
                            None => worker::Response::error("Could not deserialize args", 400)
                        },
                        None => worker::Response::error("Missing args", 400)
                    }
                }
                _ => worker::Response::error("Request type not recognized", 404)
            }
        }
    }
}