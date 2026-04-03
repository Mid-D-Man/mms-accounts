pub mod client;
pub mod auth;
pub mod admin;

pub use client::*;
pub use auth::{
    SignUpOutcome,
    generate_mid_secret,
    generate_path_id,
    copy_to_clipboard,
};
// admin adds methods to SupabaseClient via a second impl block — no extra exports needed
