use worker::*;
use serde_json::from_str;

mod utils;
mod types;

use types::interactions::*;

#[event(fetch)]
pub async fn main(mut req: Request, env: Env) -> Result<Response> {
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
                _ => worker::Response::error("Request type not recognized", 404)
            }
        }
    }
}
