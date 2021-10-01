use worker::Fetch;
pub use worker::{Request, RequestInit, Method, Env};

pub trait Requestable {
    fn ratelimit_bucket(&self) -> String;
    fn build_request(&self) -> worker::Request;
}

pub enum DiscordResponse<T> {
    WorkerError(worker::Error),
    RequestError(serde_json::Value),
    ServerError(serde_json::Value),
    Ok(T)
}

#[allow(dead_code)]
#[cfg(feature = "ratelimit")]
pub async fn request<T, R>(req: &T, env: Env) -> DiscordResponse<R> where T: Requestable, R: serde::de::DeserializeOwned
{
    let resp = if cfg!(feature = "ratelimit") {
        let obj = env.durable_object("DISCORD_RATELIMITER").unwrap();
        let limit = obj.id_from_name(&req.ratelimit_bucket()).unwrap().get_stub().unwrap();
        limit.fetch_with_request(req.build_request()).await
    }
    else {
        let r = Fetch::Request(req.build_request());
        r.send().await
    };
    match resp {
        Ok(mut r) => match r.status_code() {
            200..=299 => DiscordResponse::Ok(r.json().await.unwrap()),
            400..=499 => DiscordResponse::RequestError(r.json().await.unwrap()),
            _ => DiscordResponse::ServerError(r.json().await.unwrap())
        },
        Err(err) => DiscordResponse::WorkerError(err)
    }
}

#[macro_export]
macro_rules! build_uri {
    ($($arg:tt)+) => ({
        format!("https://discord.com/api/v9{}", format!($($arg)+))
    })
}

pub(crate) use build_uri;
