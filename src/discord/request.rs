use worker::{Headers, ObjectNamespace};
pub use worker::{Request, RequestInit, Method};
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
    pub remaining: u32,
    pub limit: u32,
    pub reset: time::SystemTime,
    pub bucket: String
}

pub enum DiscordResponse<T> {
    WorkerError(worker::Error),
    RequestError(serde_json::Value),
    ServerError(serde_json::Value),
    Ok(T, RateLimitInfo)
}

#[allow(dead_code)]
#[cfg(feature = "ratelimit")]
pub async fn request<T, R>(req: &T, limiter: ObjectNamespace) -> DiscordResponse<R> where T: Requestable, R: serde::de::DeserializeOwned
{
    let request = req.build_request();
    let limit = limiter.id_from_name(&req.ratelimit_bucket()).unwrap().get_stub().unwrap();
    let resp = limit.fetch_with_request(request).await;
    match resp {
        Ok(mut r) => match r.status_code() {
            200..=299 => DiscordResponse::Ok(r.json().await.unwrap(), ratelimit_from_headers(r.headers())),
            400..=499 => DiscordResponse::RequestError(r.json().await.unwrap()),
            _ => DiscordResponse::ServerError(r.json().await.unwrap())
        },
        Err(err) => DiscordResponse::WorkerError(err)
    }
}
