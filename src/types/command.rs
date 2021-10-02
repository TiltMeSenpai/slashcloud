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
    fn new_command(env: Env, command: T, req: InteractionRequest) -> Self;
    async fn handle_command(&self) -> InteractionResponse;
}

#[async_trait(?Send)]
pub trait InteractionHandler<T> {
    fn new_interaction(env: Env, interaction: T, req: InteractionRequest) -> Self;
    async fn handle_interaction(&mut self) -> InteractionResponse;
}

#[async_trait(?Send)]
impl<T, R> InteractionHandler<R> for T where T: CommandHandler<R>, R: CommandOption {
    fn new_interaction(env: Env, interaction: R, req: InteractionRequest) -> Self {
        T::new_command(env, interaction, req)
    }

    async fn handle_interaction(&mut self) -> InteractionResponse {
        InteractionResponse::update()
    }
}
