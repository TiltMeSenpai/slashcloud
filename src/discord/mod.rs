#[cfg(feature = "ratelimit")]
mod limiter;

mod snowflake;
mod request;

#[cfg(feature = "user")]
mod user;

#[cfg(feature = "role")]
mod role;

#[cfg(feature = "emoji")]
mod emoji;

#[cfg(feature = "guild")]
mod guild;

#[cfg(feature = "channel")]
mod channel;