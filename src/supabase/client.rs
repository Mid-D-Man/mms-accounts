use serde::{Deserialize, Serialize};
use gloo_storage::{LocalStorage, Storage};

pub const SUPABASE_URL: &str = env!("SUPABASE_URL");
pub const SUPABASE_ANON_KEY: &str = env!("SUPABASE_ANON_KEY");

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

    pub fn storage_url(&self, bucket: &str, path: &str) -> String {
        format!("{}/storage/v1/object/{}/{}", self.url, bucket, path)
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
    fn default() -> Self { Self::new() }
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
            .unwrap_or_else(|| self.email.split('@').next().unwrap_or("User"))
            .to_string()
    }
}

// ── Profile ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Profile {
    pub id:           String,
    pub email:        String,
    pub display_name: Option<String>,
    pub avatar_url:   Option<String>,
    pub bio:          Option<String>,
    pub website:      Option<String>,
    pub role:         String,
    pub mid_id:       String,
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

    pub fn is_admin(&self) -> bool { self.role == "admin" }
}

// ── MID Secret ─────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MidSecret {
    pub id:            String,
    pub user_id:       String,
    pub mid_id:        String,
    pub secret_hash:   String,
    pub secret_prefix: String,
    pub label:         Option<String>,
    pub last_used_at:  Option<String>,
    pub created_at:    String,
    pub expires_at:    Option<String>,
    pub is_active:     bool,
}

impl MidSecret {
    pub fn display_label(&self) -> String {
        self.label.clone().filter(|s| !s.is_empty())
            .unwrap_or_else(|| "Unnamed Secret".to_string())
    }

    pub fn display_prefix(&self) -> String {
        format!("{}••••••••••••••••", self.secret_prefix)
    }

    pub fn formatted_created(&self) -> String {
        self.created_at.get(..10).unwrap_or("—").to_string()
    }

    pub fn formatted_last_used(&self) -> String {
        self.last_used_at.as_deref()
            .and_then(|s| s.get(..10))
            .unwrap_or("Never")
            .to_string()
    }
}

// ── Service ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    pub id:          String,
    pub slug:        String,
    pub name:        String,
    pub description: Option<String>,
    pub is_active:   bool,
    pub is_free:     bool,
}

// ── Service Subscription ───────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceSubscription {
    pub id:          String,
    pub user_id:     String,
    pub service_id:  String,
    pub status:      String,
    pub tier:        String,
    pub enrolled_at: String,
    pub updated_at:  Option<String>,
}

impl ServiceSubscription {
    pub fn is_active(&self) -> bool { self.status == "active" }
}

// ── Registry Submission ────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrySubmission {
    pub id:                    String,
    pub user_id:               String,
    pub mid_id:                String,
    pub filename:              String,
    pub category:              String,
    pub description:           String,
    pub tags:                  Vec<String>,
    pub version:               String,
    pub status:                String,
    pub admin_note:            Option<String>,
    pub r2_key:                Option<String>,
    pub supabase_storage_path: Option<String>,
    pub submitted_at:          String,
    pub reviewed_at:           Option<String>,
}

impl RegistrySubmission {
    pub fn status_label(&self) -> &str {
        match self.status.as_str() {
            "approved" => "Approved",
            "rejected" => "Rejected",
            _          => "Pending",
        }
    }

    pub fn formatted_submitted(&self) -> String {
        self.submitted_at.get(..10).unwrap_or("—").to_string()
    }
}

// ── Supabase error ─────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct SupabaseError {
    pub message: String,
    pub error:   Option<String>,
}
