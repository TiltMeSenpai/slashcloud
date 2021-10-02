use serde_json::Value;
use async_trait::async_trait;

use crate::types::interactions::*;
use worker::Env;

pub use macros::CommandOption;

#[repr(transparent)]
pub struct WorkerEnv(Env);

unsafe impl Send for WorkerEnv {}
unsafe impl Sync for WorkerEnv {}

impl AsRef<Env> for WorkerEnv {
    fn as_ref(&self) -> &Env {
        match self {
            WorkerEnv(env) => env
        }
    }
}

impl WorkerEnv {
    pub fn new(env: Env) -> Self {
        WorkerEnv(env)
    }
}

pub trait CommandOption: Sized {
    fn from_value(options: &Value) -> Option<Self>;
    #[cfg(any(feature = "keep_json", not(target_arch = "wasm32")))]
    fn to_value() -> Value;
}

#[async_trait]
pub trait CommandHandler {
    async fn handle(&self, req: InteractionRequest, env: &WorkerEnv) -> InteractionResponse;
}

#[async_trait]
pub trait InteractionHandler {
    async fn handle_request(&self, req: InteractionRequest, env: &WorkerEnv) -> InteractionResponse;
}

#[async_trait]
impl<T> InteractionHandler for T where T: Send + Sync {
    async fn handle_request(&self, _req: InteractionRequest, _env: &WorkerEnv) -> InteractionResponse {
        InteractionResponse::update()
    }
}
