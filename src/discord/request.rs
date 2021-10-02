use worker::{Fetch, Headers};
pub use worker::{Request, RequestInit, Method, Env};
use std::time;
use std::str::FromStr;

pub trait Requestable {
    fn ratelimit_bucket(&self) -> String;
    fn build_request(&self) -> worker::Request;
}

fn timestamp_to_time(t: i32) -> time::SystemTime {
    time::UNIX_EPOCH + time::Duration::from_millis(t as u64)
}

fn header_or_default<T>(headers: &Headers, key: &str) -> T where T: FromStr + Default {
    headers.get(key).ok().flatten().map(|v| v.parse().ok()).flatten().unwrap_or_default()
}

fn ratelimit_from_headers(headers: &Headers) -> RateLimitInfo {
    RateLimitInfo {
        remaining: header_or_default(headers, "x-ratelimit-remaining"),
        limit: header_or_default(headers, "x-ratelimit-limit"),
        reset: timestamp_to_time(header_or_default(headers, "x-ratelimit-reset")),
        bucket: headers.get("x-ratelimit-bucket").ok().flatten().unwrap_or_default()
    }
}

#[allow(dead_code)]
pub struct RateLimitInfo {
    remaining: u32,
    limit: u32,
    reset: time::SystemTime,
    bucket: String
}

pub enum DiscordResponse<T> {
    MissingTokenError,
    WorkerError(worker::Error),
    RequestError(serde_json::Value),
    ServerError(serde_json::Value),
    Ok(T, RateLimitInfo)
}

#[allow(dead_code)]
#[cfg(feature = "ratelimit")]
pub async fn request<T, R>(req: &T, env: Env) -> DiscordResponse<R> where T: Requestable, R: serde::de::DeserializeOwned
{
    let mut request = req.build_request();
    let token = match env.secret("DISCORD_TOKEN") {
        Ok(token) => token,
        Err(_) => {
            return DiscordResponse::MissingTokenError;
        }
    };
    let headers = request.headers_mut().unwrap();
    headers.set("Authorization", &format!("Bot {}", token.to_string())).unwrap();
    let resp = if cfg!(feature = "ratelimit") {
        let obj = env.durable_object("DISCORD_RATELIMITER").unwrap();
        let limit = obj.id_from_name(&req.ratelimit_bucket()).unwrap().get_stub().unwrap();
        limit.fetch_with_request(request).await
    }
    else {
        let r = Fetch::Request(request);
        r.send().await
    };
    match resp {
        Ok(mut r) => match r.status_code() {
            200..=299 => DiscordResponse::Ok(r.json().await.unwrap(), ratelimit_from_headers(r.headers())),
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

#[allow(unused_imports)]
pub(crate) use build_uri;
