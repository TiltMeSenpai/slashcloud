use js_sys::Promise;
use web_sys::ServiceWorkerGlobalScope;
use wasm_bindgen::JsCast;
use std::time;
use std::convert::TryInto;
use wasm_bindgen_futures::JsFuture;
use worker::*;

async fn delay(ctx: &ServiceWorkerGlobalScope, mils: i32) {
    let _res = JsFuture::from(Promise::new(&mut | resolve, _reject | {
        ctx.set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, mils).unwrap();
    })).await.unwrap();
}


#[durable_object]
pub struct RateLimiter {
    ctx: ServiceWorkerGlobalScope,
    token: String,
    storage: Storage
}

#[durable_object]
impl DurableObject for RateLimiter {
    fn new(state: State, env: Env) -> Self {
        let token = env.secret("DISCORD_TOKEN").expect("Missing DISCORD_TOKEN secret").to_string();
        let storage = state.storage();
        RateLimiter {
            ctx: js_sys::global().dyn_into().unwrap(),
            token,
            storage
        }
    }

    async fn fetch(&mut self, static_req: Request) -> worker::Result<Response> {
        let mut req = static_req.clone().unwrap();
        let remaining: u32 = self.storage.get("remaining").await.unwrap_or_default();
        let reset: i32 = self.storage.get("reset").await.unwrap_or_default();
        console_log!("Remaining requests: {}, resetting at {}", remaining, reset);
        if remaining < 1 {
            let now = time::SystemTime::now().duration_since(time::SystemTime::UNIX_EPOCH).unwrap().as_millis().try_into().unwrap();
            if reset > now { // Reset is in the future, therefore we have to wait
                let timeout = reset - now;
                console_log!("Rate limit exceeded on {}, waiting {} ms", req.url().unwrap(), timeout);
                delay(&self.ctx, timeout).await;
            }
        }
        let headers = req.headers_mut().unwrap();
        headers.set("Authorization", &format!("Bot {}", self.token)).unwrap();
        let fetch = Fetch::Request(req);
        match fetch.send().await {
            Ok(resp) => {
                let headers = resp.headers();
                let new_reset: i32 = headers.get("x-ratelimit-reset").unwrap().unwrap_or_default().parse().unwrap_or_default();
                let _reset = self.storage.put("reset", new_reset).await.unwrap();
                let new_remaining: i32 = headers.get("x-ratelimit-remaining").unwrap().unwrap_or_default().parse().unwrap_or_default();
                let _storage = self.storage.put("remaining", new_remaining).await.unwrap();
                Ok(resp)
            }
            err => err
        }
    }
}