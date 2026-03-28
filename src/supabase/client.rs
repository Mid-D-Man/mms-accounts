use serde::{Deserialize, Serialize};
use gloo_storage::{LocalStorage, Storage};

pub const SUPABASE_URL: &str = env!("SUPABASE_URL");
pub const SUPABASE_ANON_KEY: &str = env!("SUPABASE_ANON_KEY");

// reqwest::Client removed — gloo-net is stateless (no persistent client object needed).
// Each request is constructed directly via gloo_net::http::Request.
#[derive(Clone)]
pub struct SupabaseClient {
    pub(crate) url:      String,
    pub(crate) anon_key: String,
}

impl SupabaseClient {
    pub fn new() -> Self {
        Self {
            url:      SUPABASE_URL.to_string(),
            anon_key: SUPABASE_ANON_KEY.to_string(),
        }
    }

    pub fn auth_url(&self, path: &str) -> String {
        format!("{}/auth/v1{}", self.url, path)
    }

    pub fn rest_url(&self, table: &str) -> String {
        format!("{}/rest/v1/{}", self.url, table)
    }

    pub fn is_logged_in() -> bool {
        LocalStorage::get::<String>("mms_access_token").is_ok()
    }

    pub fn clear_session() {
        let _ = LocalStorage::delete("mms_access_token");
        let _ = LocalStorage::delete("mms_refresh_token");
        let _ = LocalStorage::delete("mms_user_id");
    }
}

impl Default for SupabaseClient {
    fn default() -> Self {
        Self::new()
    }
}

// ── Auth response types ────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthSession {
    pub access_token:  String,
    pub refresh_token: String,
    pub token_type:    String,
    pub expires_in:    Option<u64>,
    pub user:          User,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id:            String,
    pub email:         String,
    pub user_metadata: serde_json::Value,
    pub created_at:    Option<String>,
}

impl User {
    pub fn display_name(&self) -> String {
        self.user_metadata
            .get("display_name")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| {
                self.email.split('@').next().unwrap_or("User")
            })
            .to_string()
    }
}

// ── Profile (from public.profiles table) ──────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Profile {
    pub id:           String,
    pub email:        String,
    pub display_name: Option<String>,
    pub avatar_url:   Option<String>,
    pub bio:          Option<String>,
    pub website:      Option<String>,
    pub role:         String,
    pub created_at:   Option<String>,
    pub updated_at:   Option<String>,
}

impl Profile {
    pub fn display_name_or_email(&self) -> String {
        self.display_name
            .clone()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| self.email.clone())
    }

    pub fn is_admin(&self) -> bool {
        self.role == "admin"
    }
}

// ── Supabase error shape ───────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct SupabaseError {
    pub message: String,
    pub error:   Option<String>,
}
