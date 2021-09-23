use serde_json::Value;

use crate::types::interactions::*;

pub use macros::CommandOption;

pub trait CommandOption: Sized {
    fn from_value(options: &Value) -> Option<Self>;
    #[cfg(any(feature = "keep_json", not(target_arch = "wasm32")))]
    fn to_value() -> Value;
}

pub trait CommandHandler {
    fn handle(&self, req: InteractionRequest) -> InteractionResponse;
}

pub trait InteractionHandler {
    fn handle_request(&self, req: InteractionRequest) -> InteractionResponse;
}

impl<T> InteractionHandler for T {
    fn handle_request(&self, _req: InteractionRequest) -> InteractionResponse {
        InteractionResponse::update()
    }
}
