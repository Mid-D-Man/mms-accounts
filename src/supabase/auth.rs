use super::client::*;
use serde::{Deserialize, Serialize};
use gloo_storage::{LocalStorage, Storage};

// ── Request bodies ─────────────────────────────────────────────

#[derive(Serialize)]
struct SignUpBody {
    email:    String,
    password: String,
    data:     serde_json::Value,
}

#[derive(Serialize)]
struct SignInBody {
    email:    String,
    password: String,
}

#[derive(Serialize)]
struct UpdateProfileBody {
    display_name: Option<String>,
    bio:          Option<String>,
    website:      Option<String>,
    avatar_url:   Option<String>,
}

// Flexible signup response — access_token is absent when
// Supabase requires email confirmation first.
#[derive(Debug, Deserialize)]
struct SignUpResponse {
    access_token:  Option<String>,
    refresh_token: Option<String>,
    token_type:    Option<String>,
    expires_in:    Option<u64>,
    user:          Option<User>,
    // Top-level user fields present when confirmation is required
    id:            Option<String>,
    email:         Option<String>,
}

// ── Outcome returned to callers ────────────────────────────────

pub enum SignUpOutcome {
    /// Signed in immediately — session persisted.
    LoggedIn,
    /// Confirmation email sent — user must click link first.
    ConfirmationRequired,
}

// ── Auth methods ───────────────────────────────────────────────

impl SupabaseClient {
    pub async fn sign_up(
        &self,
        email:    &str,
        password: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<SignUpOutcome, String> {
        let body = SignUpBody {
            email:    email.to_string(),
            password: password.to_string(),
            data:     metadata.unwrap_or(serde_json::json!({})),
        };

        let res = self.client
            .post(&self.auth_url("/signup"))
            .header("apikey",       &self.anon_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        if !res.status().is_success() {
            let text = res.text().await.unwrap_or_default();
            let msg = serde_json::from_str::<SupabaseError>(&text)
                .map(|e| e.message)
                .unwrap_or(text);
            return Err(msg);
        }

        let payload: SignUpResponse = res.json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;

        // Full session returned — email confirmation is disabled.
        if let (Some(access_token), Some(refresh_token), Some(user)) = (
            payload.access_token,
            payload.refresh_token,
            payload.user,
        ) {
            let session = AuthSession {
                access_token,
                refresh_token,
                token_type: payload.token_type.unwrap_or_else(|| "bearer".into()),
                expires_in: payload.expires_in,
                user,
            };
            Self::persist_session(&session);
            return Ok(SignUpOutcome::LoggedIn);
        }

        // No access_token — Supabase sent a confirmation email instead.
        Ok(SignUpOutcome::ConfirmationRequired)
    }

    pub async fn sign_in(
        &self,
        email:    &str,
        password: &str,
    ) -> Result<AuthSession, String> {
        let body = SignInBody {
            email:    email.to_string(),
            password: password.to_string(),
        };

        let res = self.client
            .post(&self.auth_url("/token?grant_type=password"))
            .header("apikey",       &self.anon_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        if !res.status().is_success() {
            let text = res.text().await.unwrap_or_default();
            let msg = serde_json::from_str::<SupabaseError>(&text)
                .map(|e| e.message)
                .unwrap_or(text);
            return Err(msg);
        }

        let session: AuthSession = res.json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;

        Self::persist_session(&session);
        Ok(session)
    }

    pub async fn sign_out(&self) -> Result<(), String> {
        let token = LocalStorage::get::<String>("mms_access_token")
            .map_err(|_| "No session")?;

        let _ = self.client
            .post(&self.auth_url("/logout"))
            .header("apikey",        &self.anon_key)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await;

        Self::clear_session();
        Ok(())
    }

    pub async fn get_user(&self) -> Result<User, String> {
        let token = LocalStorage::get::<String>("mms_access_token")
            .map_err(|_| "Not authenticated".to_string())?;

        let res = self.client
            .get(&self.auth_url("/user"))
            .header("apikey",        &self.anon_key)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        if !res.status().is_success() {
            Self::clear_session();
            return Err("Session expired".to_string());
        }

        res.json::<User>()
            .await
            .map_err(|e| format!("Parse error: {}", e))
    }

    pub async fn get_profile(&self, user_id: &str) -> Result<Profile, String> {
        let token = LocalStorage::get::<String>("mms_access_token")
            .map_err(|_| "Not authenticated".to_string())?;

        let url = format!(
            "{}?id=eq.{}&select=*",
            self.rest_url("profiles"),
            user_id
        );

        let res = self.client
            .get(&url)
            .header("apikey",        &self.anon_key)
            .header("Authorization", format!("Bearer {}", token))
            .header("Accept",        "application/json")
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        if !res.status().is_success() {
            return Err("Failed to fetch profile".to_string());
        }

        let profiles: Vec<Profile> = res.json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;

        profiles.into_iter().next()
            .ok_or_else(|| "Profile not found".to_string())
    }

    pub async fn update_profile(
        &self,
        user_id:      &str,
        display_name: Option<String>,
        bio:          Option<String>,
        website:      Option<String>,
        avatar_url:   Option<String>,
    ) -> Result<Profile, String> {
        let token = LocalStorage::get::<String>("mms_access_token")
            .map_err(|_| "Not authenticated".to_string())?;

        let body = UpdateProfileBody {
            display_name,
            bio,
            website,
            avatar_url,
        };

        let url = format!(
            "{}?id=eq.{}",
            self.rest_url("profiles"),
            user_id
        );

        let res = self.client
            .patch(&url)
            .header("apikey",        &self.anon_key)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type",  "application/json")
            .header("Prefer",        "return=representation")
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        if !res.status().is_success() {
            let text = res.text().await.unwrap_or_default();
            return Err(format!("Update failed: {}", text));
        }

        let profiles: Vec<Profile> = res.json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;

        profiles.into_iter().next()
            .ok_or_else(|| "No profile returned".to_string())
    }

    pub async fn get_all_profiles(&self) -> Result<Vec<Profile>, String> {
        let token = LocalStorage::get::<String>("mms_access_token")
            .map_err(|_| "Not authenticated".to_string())?;

        let url = format!(
            "{}?select=*&order=created_at.desc",
            self.rest_url("profiles")
        );

        let res = self.client
            .get(&url)
            .header("apikey",        &self.anon_key)
            .header("Authorization", format!("Bearer {}", token))
            .header("Accept",        "application/json")
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        if !res.status().is_success() {
            return Err("Failed to fetch profiles — admin role required".to_string());
        }

        res.json::<Vec<Profile>>()
            .await
            .map_err(|e| format!("Parse error: {}", e))
    }

    // ── Internal helpers ───────────────────────────────────────

    fn persist_session(session: &AuthSession) {
        let _ = LocalStorage::set("mms_access_token",  &session.access_token);
        let _ = LocalStorage::set("mms_refresh_token", &session.refresh_token);
        let _ = LocalStorage::set("mms_user_id",       &session.user.id);
    }
                }
