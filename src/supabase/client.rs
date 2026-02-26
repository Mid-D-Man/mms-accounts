use reqwest::Client;
use serde::{Deserialize, Serialize};
use gloo_storage::{LocalStorage, Storage};

pub const SUPABASE_URL: &str = "https://your-project.supabase.co";
pub const SUPABASE_ANON_KEY: &str = "your-anon-key-here";

#[derive(Clone)]
pub struct SupabaseClient {
    client: Client,
    url: String,
    anon_key: String,
}

impl SupabaseClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            url: SUPABASE_URL.to_string(),
            anon_key: SUPABASE_ANON_KEY.to_string(),
        }
    }

    pub fn get_auth_header(&self) -> String {
        if let Ok(token) = LocalStorage::get::<String>("supabase_token") {
            format!("Bearer {}", token)
        } else {
            format!("Bearer {}", self.anon_key)
        }
    }

    pub fn auth_url(&self, path: &str) -> String {
        format!("{}/auth/v1{}", self.url, path)
    }

    pub fn rest_url(&self, table: &str) -> String {
        format!("{}/rest/v1/{}", self.url, table)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthSession {
    pub access_token: String,
    pub refresh_token: String,
    pub user: User,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
    pub user_metadata: serde_json::Value,
}
