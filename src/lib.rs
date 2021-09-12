use worker::*;

mod utils;
mod types;

#[event(fetch)]
pub async fn main(mut req: Request, env: Env) -> Result<Response> {
    let body = req.bytes().await.unwrap();
    let ctx = utils::JsCtx::new();
    let key = match ctx.get_key(&env).await{
        Ok(key) => key,
        Err(msg) => return worker::Response::error(msg, 500)
    };
    match ctx.verify_request(&key, &req, body).await {
        Ok(()) => worker::Response::from_json(&types::interactions::InteractionResponse::Pong),
        Err(msg) => return worker::Response::error(msg, 401)
    }
}
