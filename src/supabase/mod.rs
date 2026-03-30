pub mod client;
pub mod auth;

pub use client::*;
pub use auth::{SignUpOutcome, generate_mid_secret, copy_to_clipboard};
