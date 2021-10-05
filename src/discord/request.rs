use worker::{Headers, Env};
pub use worker::{Request, RequestInit, Method};
use std::str::FromStr;
use wasm_bindgen::JsValue;

pub trait Requestable {
    fn ratelimit_bucket(&self) -> String;
    fn build_request(&self) -> worker::Request;
}


fn timestamp_to_time(t: f64) -> worker::Date {
    worker::Date::new(worker::DateInit::Millis((t * 1000.0) as u64))
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
    pub remaining: u32,
    pub limit: u32,
    pub reset: worker::Date,
    pub bucket: String
}

pub enum DiscordResponse<T> {
    #[cfg(feature = "ratelimit")]
    MissingLimiterError,
    WorkerError(worker::Error),
    RequestError(serde_json::Value),
    ServerError(serde_json::Value),
    Ok(T, RateLimitInfo)
}

#[allow(dead_code)]
pub async fn request<T, R>(req: &T, env: Env) -> DiscordResponse<R> where T: Requestable, R: serde::de::DeserializeOwned
{
    let request = req.build_request();
    let resp = if cfg!(feature = "ratelimit"){
        if let Ok(limiter) = env.durable_object("DISCORD_RATELIMITER"){
            let limit = limiter.id_from_name(&req.ratelimit_bucket()).unwrap().get_stub().unwrap();
            limit.fetch_with_request(request).await
        } else {
            return DiscordResponse::MissingLimiterError;
        }
    } else {
        use worker::Fetch;
        let fetch = Fetch::Request(request);
        fetch.send().await
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

#[allow(dead_code)]
pub fn to_body<T>(body: &T) -> JsValue where T: serde::Serialize{
    serde_json::to_string(body).map(|val| JsValue::from_str(&val)).unwrap()
}

#[allow(unused_macros)]
macro_rules! build_request {
    ($method:tt [$($path:expr),+], $body:expr) => {
        Request::new_with_init(&format!($($path),+),
            worker::RequestInit::new()
                .with_method(worker::Method::$method)
                .with_body(Some(to_body($body)))
            ).unwrap()
    };
    ($method:tt [$($path:expr),+]) => {
        Request::new(&format!($($path),+), worker::Method::$method).unwrap()
    }
}

pub(crate) use build_request;