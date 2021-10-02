use serde_json::Value;
use async_trait::async_trait;
use worker::Env;

use crate::types::interactions::*;

pub use macros::CommandOption;

pub trait CommandOption: Sized {
    fn from_value(options: &Value) -> Option<Self>;
    #[cfg(any(feature = "keep_json", not(target_arch = "wasm32")))]
    fn to_value() -> Value;
}

#[async_trait(?Send)]
pub trait CommandHandler<T> where T: CommandOption{
    async fn handle_command(env: Env, command: T, req: InteractionRequest) -> InteractionResponse;
}

#[async_trait(?Send)]
pub trait InteractionHandler<T> {
    async fn handle_interaction(env: Env, req: InteractionRequest) -> InteractionResponse;
}

#[async_trait(?Send)]
impl<T, R> InteractionHandler<R> for T where T: CommandHandler<R>, R: CommandOption {
    async fn handle_interaction(_env: Env, _req: InteractionRequest) -> InteractionResponse {
        InteractionResponse::update()
    }
}
