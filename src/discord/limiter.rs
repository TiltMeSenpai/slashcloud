use js_sys::Promise;
use web_sys::ServiceWorkerGlobalScope;
use wasm_bindgen::JsCast;
use std::convert::TryInto;
use wasm_bindgen_futures::JsFuture;
use worker::*;

extern crate console_error_panic_hook;
use console_error_panic_hook::set_once as panic_hook;

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
        panic_hook();
        let token = env.secret("DISCORD_TOKEN").expect("Missing DISCORD_TOKEN secret").to_string();
        let storage = state.storage();
        RateLimiter {
            ctx: js_sys::global().dyn_into().unwrap(),
            token,
            storage
        }
    }

    async fn fetch(&mut self, mut incoming: Request) -> worker::Result<Response> {
        let path = incoming.path();
        let remaining: u32 = self.storage.get("remaining").await.unwrap_or_default();
        let reset: u64 = self.storage.get("reset").await.unwrap_or_default();
        console_log!("Remaining requests: {}, resetting at {}", remaining, reset);
        if remaining < 1 {
            console_log!("Limits exceeded, Delaying request");
            let now = Date::now().as_millis();
            if reset > now { // Reset is in the future, therefore we have to wait
                let timeout = reset - now;
                console_log!("Rate limit exceeded, waiting {} ms", timeout);
                delay(&self.ctx, timeout.try_into().unwrap()).await;
            } else {
                console_log!("Reset in past, proceeding anyways");
            }
        }

        console_log!("Preparing request");
        let mut headers = Headers::new();
        headers.set("Authorization", &format!("Bot {}", self.token)).unwrap();
        headers.set("Content-Type", "application/json").unwrap();

        let body = incoming.text().await.ok().map(|value| wasm_bindgen::JsValue::from_str(&value));

        let req = Request::new_with_init(&format!("https://discord.com/api/v9{}", path), &RequestInit{
            body: body,
            ..Default::default()
        }).unwrap();
        let fetch = Fetch::Request(req);
        console_log!("Sending request");
        match fetch.send().await {
            Ok(resp) => {
                let headers = resp.headers();
                let new_reset: i32 = headers.get("x-ratelimit-reset").unwrap().unwrap_or_default().parse().unwrap_or_default();
                let _reset = self.storage.put("reset", new_reset).await.unwrap();
                let new_remaining: i32 = headers.get("x-ratelimit-remaining").unwrap().unwrap_or_default().parse().unwrap_or_default();
                let _storage = self.storage.put("remaining", new_remaining).await.unwrap();
                Ok(resp)
            }
            err => {
                console_log!("Error: {:?}", err);
                err
            }
        }
    }
}