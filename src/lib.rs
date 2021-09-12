use serde_json::json;
use worker::*;

mod utils;
mod types;

fn log_request(req: &Request) {
    console_log!(
        "{} - [{}], located at: {:?}, within: {}",
        Date::now().to_string(),
        req.path(),
        req.cf().coordinates().unwrap_or_default(),
        req.cf().region().unwrap_or("unknown region".into())
    );
}

#[event(fetch)]
pub async fn main(mut req: Request, env: Env) -> Result<Response> {
    console_log!("Hello world!");
    log_request(&req);

    let body_txt = req.text().await.unwrap();
    let ctx = utils::JsCtx::new();
    let key = match ctx.get_key(&env).await{
        Ok(key) => key,
        Err(msg) => return worker::Response::error(msg, 500)
    };
    match ctx.verify_request(&key, &req, body_txt).await {
        Ok(()) => worker::Response::ok("Signature verified!"),
        Err(msg) => return worker::Response::error(msg, 500)
    }
}
