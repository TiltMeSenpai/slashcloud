use serde_json::Value;
use async_trait::async_trait;

use crate::types::interactions::*;
use worker::Env;

pub use macros::CommandOption;

pub trait CommandOption: Sized {
    fn from_value(options: &Value) -> Option<Self>;
    #[cfg(any(feature = "keep_json", not(target_arch = "wasm32")))]
    fn to_value() -> Value;
}

#[async_trait]
pub trait CommandHandler {
    async fn handle(&self, req: InteractionRequest, env: &Env) -> InteractionResponse;
}

pub trait InteractionHandler {
    fn handle_request(&self, req: InteractionRequest, env: &Env) -> InteractionResponse;
}

impl<T> InteractionHandler for T {
    fn handle_request(&self, _req: InteractionRequest, _env: &Env) -> InteractionResponse {
        InteractionResponse::update()
    }
}
