use super::client::*;
use serde::{Deserialize, Serialize};
use gloo_storage::{LocalStorage, Storage};
use gloo_net::http::Request;
use wasm_bindgen_futures::JsFuture;
use js_sys::{ArrayBuffer, Uint8Array};

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
struct RefreshBody {
    refresh_token: String,
}

#[derive(Serialize)]
struct ResendBody {
    #[serde(rename = "type")]
    kind:  String,
    email: String,
}

#[derive(Serialize)]
struct RecoverBody {
    email: String,
}

#[derive(Serialize)]
struct UpdatePasswordBody {
    password: String,
}

#[derive(Serialize)]
struct UpdateProfileBody {
    display_name: Option<String>,
    bio:          Option<String>,
    website:      Option<String>,
    avatar_url:   Option<String>,
}

#[derive(Serialize)]
struct CreateSecretBody {
    user_id:       String,
    mid_id:        String,
    secret_hash:   String,
    secret_prefix: String,
    label:         Option<String>,
}

#[derive(Serialize)]
struct RevokeSecretBody {
    is_active: bool,
}

#[derive(Serialize)]
struct SubscribeBody {
    user_id:    String,
    service_id: String,
    status:     String,
    tier:       String,
}

#[derive(Serialize)]
struct UpdateSubscriptionBody {
    status: String,
}

#[derive(Serialize)]
struct NewSubmissionBody {
    user_id:               String,
    mid_id:                String,
    filename:              String,
    category:              String,
    description:           String,
    tags:                  serde_json::Value,
    version:               String,
    supabase_storage_path: Option<String>,
    status:                String,
}

#[derive(Serialize)]
struct ApproveBody {
    submission_id: String,
}

#[derive(Serialize)]
struct RejectBody {
    submission_id: String,
    note:          String,
}

#[derive(Serialize)]
struct DeleteR2Body {
    key: String,
}

#[derive(Serialize)]
struct MoveR2Body {
    from_key: String,
    to_key:   String,
}

// ── Flexible signup response ───────────────────────────────────

#[derive(Debug, Deserialize)]
struct SignUpResponse {
    access_token:  Option<String>,
    refresh_token: Option<String>,
    token_type:    Option<String>,
    expires_in:    Option<u64>,
    user:          Option<User>,
}

// ── Outcome types ──────────────────────────────────────────────

pub enum SignUpOutcome {
    LoggedIn,
    ConfirmationRequired,
}

// ── Crypto helpers ─────────────────────────────────────────────

pub async fn generate_mid_secret() -> Result<(String, String, String), String> {
    let window = web_sys::window().ok_or("No window")?;
    let crypto = window.crypto().map_err(|_| "No crypto")?;

    let array = Uint8Array::new_with_length(32);
    crypto.get_random_values_with_js_u8_array(&array)
        .map_err(|_| "Failed to generate random bytes")?;

    let charset = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let bytes: Vec<u8> = array.to_vec();
    let body: String = bytes.iter()
        .map(|b| charset[(*b as usize) % charset.len()] as char)
        .collect();

    let full_secret = format!("mids_{}", body);
    let prefix      = format!("mids_{}", &body[..8]);
    let hash_hex    = sha256_hex(full_secret.as_bytes()).await?;

    Ok((full_secret, prefix, hash_hex))
}

pub async fn generate_path_id() -> Result<String, String> {
    let window = web_sys::window().ok_or("No window")?;
    let crypto = window.crypto().map_err(|_| "No crypto")?;

    let array = Uint8Array::new_with_length(8);
    crypto.get_random_values_with_js_u8_array(&array)
        .map_err(|_| "Failed to generate random bytes")?;

    let bytes: Vec<u8> = array.to_vec();
    let hex: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();
    let ts = js_sys::Date::now() as u64;
    Ok(format!("{}_{}", ts, hex))
}

async fn sha256_hex(data: &[u8]) -> Result<String, String> {
    let window = web_sys::window().ok_or("No window")?;
    let crypto = window.crypto().map_err(|_| "No crypto")?;
    let subtle = crypto.subtle();

    let data_array = Uint8Array::from(data);
    let promise = subtle.digest_with_str_and_buffer_source("SHA-256", &data_array)
        .map_err(|_| "SubtleCrypto.digest failed")?;
    let result = JsFuture::from(promise).await
        .map_err(|_| "SHA-256 promise rejected")?;

    let hash_buffer = ArrayBuffer::from(result);
    let hash_bytes  = Uint8Array::new(&hash_buffer);
    let bytes: Vec<u8> = hash_bytes.to_vec();

    Ok(bytes.iter().map(|b| format!("{:02x}", b)).collect())
}

pub async fn copy_to_clipboard(text: &str) -> Result<(), String> {
    let window    = web_sys::window().ok_or("No window")?;
    let navigator = window.navigator();
    let clipboard = navigator.clipboard();
    let promise   = clipboard.write_text(text);
    JsFuture::from(promise).await.map_err(|_| "Clipboard write failed")?;
    Ok(())
}

// ── Helper: parse error from dixscript-docs response ──────────

async fn parse_docs_error(res: gloo_net::http::Response) -> String {
    let text = res.text().await.unwrap_or_default();
    serde_json::from_str::<serde_json::Value>(&text)
        .ok()
        .and_then(|v| {
            v.get("error")
                .or_else(|| v.get("message"))
                .and_then(|e| e.as_str())
                .map(String::from)
        })
        .unwrap_or(text)
}

// ── SupabaseClient impl ────────────────────────────────────────

impl SupabaseClient {

    // ── Sign up ────────────────────────────────────────────────

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

        let res = Request::post(&self.auth_url("/signup"))
            .header("apikey",       &self.anon_key)
            .header("Content-Type", "application/json")
            .json(&body).map_err(|e| format!("Request build error: {}", e))?
            .send().await.map_err(|e| format!("Network error: {}", e))?;

        if !res.ok() {
            let text = res.text().await.unwrap_or_default();
            let msg = serde_json::from_str::<SupabaseError>(&text)
                .map(|e| e.message).unwrap_or(text);
            return Err(msg);
        }

        let payload: SignUpResponse = res.json().await
            .map_err(|e| format!("Parse error: {}", e))?;

        if let (Some(access_token), Some(refresh_token), Some(user)) = (
            payload.access_token, payload.refresh_token, payload.user,
        ) {
            let session = AuthSession {
                access_token, refresh_token,
                token_type: payload.token_type.unwrap_or_else(|| "bearer".into()),
                expires_in: payload.expires_in, user,
            };
            Self::persist_session(&session);
            return Ok(SignUpOutcome::LoggedIn);
        }

        Ok(SignUpOutcome::ConfirmationRequired)
    }

    // ── Sign in ────────────────────────────────────────────────

    pub async fn sign_in(&self, email: &str, password: &str) -> Result<AuthSession, String> {
        let body = SignInBody { email: email.to_string(), password: password.to_string() };

        let res = Request::post(&self.auth_url("/token?grant_type=password"))
            .header("apikey",       &self.anon_key)
            .header("Content-Type", "application/json")
            .json(&body).map_err(|e| format!("Request build error: {}", e))?
            .send().await.map_err(|e| format!("Network error: {}", e))?;

        if !res.ok() {
            let text = res.text().await.unwrap_or_default();
            let msg = serde_json::from_str::<SupabaseError>(&text)
                .map(|e| e.message).unwrap_or(text);
            return Err(msg);
        }

        let session: AuthSession = res.json().await
            .map_err(|e| format!("Parse error: {}", e))?;

        Self::persist_session(&session);
        Ok(session)
    }

    // ── Sign out ───────────────────────────────────────────────

    pub async fn sign_out(&self) -> Result<(), String> {
        let token = LocalStorage::get::<String>("mms_access_token")
            .map_err(|_| "No session")?;

        let _ = Request::post(&self.auth_url("/logout"))
            .header("apikey",        &self.anon_key)
            .header("Authorization", &format!("Bearer {}", token))
            .send().await;

        Self::clear_session();
        Ok(())
    }

    // ── Resend verification ────────────────────────────────────

    pub async fn resend_verification(&self, email: &str) -> Result<(), String> {
        let body = ResendBody { kind: "signup".to_string(), email: email.to_string() };

        let res = Request::post(&self.auth_url("/resend"))
            .header("apikey",       &self.anon_key)
            .header("Content-Type", "application/json")
            .json(&body).map_err(|e| format!("Request build error: {}", e))?
            .send().await.map_err(|e| format!("Network error: {}", e))?;

        if !res.ok() {
            let text = res.text().await.unwrap_or_default();
            let msg = serde_json::from_str::<SupabaseError>(&text)
                .map(|e| e.message).unwrap_or(text);
            return Err(msg);
        }
        Ok(())
    }

    // ── Password recovery ──────────────────────────────────────

    pub async fn send_password_recovery(&self, email: &str) -> Result<(), String> {
        let origin = web_sys::window()
            .and_then(|w| w.location().origin().ok())
            .unwrap_or_default();

        let body = RecoverBody { email: email.to_string() };
        let url  = format!("{}?redirect_to={}",
            self.auth_url("/recover"),
            js_sys::encode_uri_component(&origin));

        let res = Request::post(&url)
            .header("apikey",       &self.anon_key)
            .header("Content-Type", "application/json")
            .json(&body).map_err(|e| format!("Request build error: {}", e))?
            .send().await.map_err(|e| format!("Network error: {}", e))?;

        if !res.ok() {
            let text = res.text().await.unwrap_or_default();
            let msg = serde_json::from_str::<SupabaseError>(&text)
                .map(|e| e.message).unwrap_or(text);
            return Err(msg);
        }
        Ok(())
    }

    // ── Reset password with token ──────────────────────────────

    pub async fn reset_password_with_token(
        &self, recovery_token: &str, new_password: &str,
    ) -> Result<(), String> {
        let body = UpdatePasswordBody { password: new_password.to_string() };

        let res = Request::put(&self.auth_url("/user"))
            .header("apikey",        &self.anon_key)
            .header("Authorization", &format!("Bearer {}", recovery_token))
            .header("Content-Type",  "application/json")
            .json(&body).map_err(|e| format!("Request build error: {}", e))?
            .send().await.map_err(|e| format!("Network error: {}", e))?;

        if !res.ok() {
            let text = res.text().await.unwrap_or_default();
            let msg = serde_json::from_str::<SupabaseError>(&text)
                .map(|e| e.message).unwrap_or(text);
            return Err(msg);
        }
        Ok(())
    }

    // ── Update password ────────────────────────────────────────

    pub async fn update_password(&self, new_password: &str) -> Result<(), String> {
        let token = LocalStorage::get::<String>("mms_access_token")
            .map_err(|_| "Not authenticated".to_string())?;

        let body = UpdatePasswordBody { password: new_password.to_string() };

        let res = Request::put(&self.auth_url("/user"))
            .header("apikey",        &self.anon_key)
            .header("Authorization", &format!("Bearer {}", token))
            .header("Content-Type",  "application/json")
            .json(&body).map_err(|e| format!("Request build error: {}", e))?
            .send().await.map_err(|e| format!("Network error: {}", e))?;

        if !res.ok() {
            let text = res.text().await.unwrap_or_default();
            let msg = serde_json::from_str::<SupabaseError>(&text)
                .map(|e| e.message).unwrap_or(text);
            return Err(msg);
        }
        Ok(())
    }

    // ── Session refresh ────────────────────────────────────────

    pub async fn try_refresh_session(&self) -> Result<bool, String> {
        let refresh_token = match LocalStorage::get::<String>("mms_refresh_token") {
            Ok(t) if !t.is_empty() => t,
            _ => return Ok(false),
        };

        let body = RefreshBody { refresh_token };

        let res = Request::post(&self.auth_url("/token?grant_type=refresh_token"))
            .header("apikey",       &self.anon_key)
            .header("Content-Type", "application/json")
            .json(&body).map_err(|e| format!("Request build error: {}", e))?
            .send().await.map_err(|e| format!("Network error: {}", e))?;

        if !res.ok() {
            Self::clear_session();
            return Err("Session expired. Please sign in again.".to_string());
        }

        let session: AuthSession = res.json().await
            .map_err(|e| format!("Parse error: {}", e))?;

        Self::persist_session(&session);
        Ok(true)
    }

    // ── Profile methods ────────────────────────────────────────

    pub async fn get_user(&self) -> Result<User, String> {
        let token = LocalStorage::get::<String>("mms_access_token")
            .map_err(|_| "Not authenticated".to_string())?;

        let res = Request::get(&self.auth_url("/user"))
            .header("apikey",        &self.anon_key)
            .header("Authorization", &format!("Bearer {}", token))
            .send().await.map_err(|e| format!("Network error: {}", e))?;

        if !res.ok() {
            Self::clear_session();
            return Err("Session expired".to_string());
        }

        res.json::<User>().await.map_err(|e| format!("Parse error: {}", e))
    }

    pub async fn get_profile(&self, user_id: &str) -> Result<Profile, String> {
        let token = LocalStorage::get::<String>("mms_access_token")
            .map_err(|_| "Not authenticated".to_string())?;

        let url = format!("{}?id=eq.{}&select=*", self.rest_url("profiles"), user_id);

        let res = Request::get(&url)
            .header("apikey",        &self.anon_key)
            .header("Authorization", &format!("Bearer {}", token))
            .header("Accept",        "application/json")
            .send().await.map_err(|e| format!("Network error: {}", e))?;

        if !res.ok() {
            return Err("Failed to fetch profile".to_string());
        }

        let profiles: Vec<Profile> = res.json().await
            .map_err(|e| format!("Parse error: {}", e))?;

        profiles.into_iter().next().ok_or_else(|| "Profile not found".to_string())
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

        let body = UpdateProfileBody { display_name, bio, website, avatar_url };
        let url  = format!("{}?id=eq.{}", self.rest_url("profiles"), user_id);

        let res = Request::patch(&url)
            .header("apikey",        &self.anon_key)
            .header("Authorization", &format!("Bearer {}", token))
            .header("Prefer",        "return=representation")
            .json(&body).map_err(|e| format!("Request build error: {}", e))?
            .send().await.map_err(|e| format!("Network error: {}", e))?;

        if !res.ok() {
            let text = res.text().await.unwrap_or_default();
            return Err(format!("Update failed: {}", text));
        }

        let profiles: Vec<Profile> = res.json().await
            .map_err(|e| format!("Parse error: {}", e))?;

        profiles.into_iter().next().ok_or_else(|| "No profile returned".to_string())
    }

    pub async fn get_all_profiles(&self) -> Result<Vec<Profile>, String> {
        let token = LocalStorage::get::<String>("mms_access_token")
            .map_err(|_| "Not authenticated".to_string())?;

        let url = format!("{}?select=*&order=created_at.desc", self.rest_url("profiles"));

        let res = Request::get(&url)
            .header("apikey",        &self.anon_key)
            .header("Authorization", &format!("Bearer {}", token))
            .header("Accept",        "application/json")
            .send().await.map_err(|e| format!("Network error: {}", e))?;

        if !res.ok() {
            return Err("Failed to fetch profiles — admin role required".to_string());
        }

        res.json::<Vec<Profile>>().await.map_err(|e| format!("Parse error: {}", e))
    }

    // ── MID Secret methods ─────────────────────────────────────

    pub async fn list_secrets(&self, user_id: &str) -> Result<Vec<MidSecret>, String> {
        let token = LocalStorage::get::<String>("mms_access_token")
            .map_err(|_| "Not authenticated".to_string())?;

        let url = format!(
            "{}?user_id=eq.{}&is_active=eq.true&order=created_at.desc&select=*",
            self.rest_url("mid_secrets"), user_id
        );

        let res = Request::get(&url)
            .header("apikey",        &self.anon_key)
            .header("Authorization", &format!("Bearer {}", token))
            .header("Accept",        "application/json")
            .send().await.map_err(|e| format!("Network error: {}", e))?;

        if !res.ok() {
            let text = res.text().await.unwrap_or_default();
            return Err(format!("Failed to fetch secrets: {}", text));
        }

        res.json::<Vec<MidSecret>>().await.map_err(|e| format!("Parse error: {}", e))
    }

    pub async fn create_secret(
        &self,
        user_id:       &str,
        mid_id:        &str,
        label:         Option<String>,
        secret_hash:   &str,
        secret_prefix: &str,
    ) -> Result<MidSecret, String> {
        let token = LocalStorage::get::<String>("mms_access_token")
            .map_err(|_| "Not authenticated".to_string())?;

        let body = CreateSecretBody {
            user_id:       user_id.to_string(),
            mid_id:        mid_id.to_string(),
            secret_hash:   secret_hash.to_string(),
            secret_prefix: secret_prefix.to_string(),
            label,
        };

        let res = Request::post(&self.rest_url("mid_secrets"))
            .header("apikey",        &self.anon_key)
            .header("Authorization", &format!("Bearer {}", token))
            .header("Prefer",        "return=representation")
            .json(&body).map_err(|e| format!("Request build error: {}", e))?
            .send().await.map_err(|e| format!("Network error: {}", e))?;

        if !res.ok() {
            let text = res.text().await.unwrap_or_default();
            return Err(format!("Failed to create secret: {}", text));
        }

        let secrets: Vec<MidSecret> = res.json().await
            .map_err(|e| format!("Parse error: {}", e))?;

        secrets.into_iter().next().ok_or_else(|| "No secret returned".to_string())
    }

    pub async fn revoke_secret(&self, secret_id: &str, user_id: &str) -> Result<(), String> {
        let token = LocalStorage::get::<String>("mms_access_token")
            .map_err(|_| "Not authenticated".to_string())?;

        let body = RevokeSecretBody { is_active: false };
        let url  = format!(
            "{}?id=eq.{}&user_id=eq.{}",
            self.rest_url("mid_secrets"), secret_id, user_id
        );

        let res = Request::patch(&url)
            .header("apikey",        &self.anon_key)
            .header("Authorization", &format!("Bearer {}", token))
            .json(&body).map_err(|e| format!("Request build error: {}", e))?
            .send().await.map_err(|e| format!("Network error: {}", e))?;

        if !res.ok() {
            let text = res.text().await.unwrap_or_default();
            return Err(format!("Failed to revoke secret: {}", text));
        }
        Ok(())
    }

    // ── Service methods ────────────────────────────────────────

    pub async fn list_services(&self) -> Result<Vec<Service>, String> {
        let token = LocalStorage::get::<String>("mms_access_token")
            .map_err(|_| "Not authenticated".to_string())?;

        let url = format!("{}?select=*&order=name.asc", self.rest_url("services"));

        let res = Request::get(&url)
            .header("apikey",        &self.anon_key)
            .header("Authorization", &format!("Bearer {}", token))
            .header("Accept",        "application/json")
            .send().await.map_err(|e| format!("Network error: {}", e))?;

        if !res.ok() {
            return Err("Failed to fetch services".to_string());
        }

        res.json::<Vec<Service>>().await.map_err(|e| format!("Parse error: {}", e))
    }

    pub async fn get_user_subscriptions(&self, user_id: &str) -> Result<Vec<ServiceSubscription>, String> {
        let token = LocalStorage::get::<String>("mms_access_token")
            .map_err(|_| "Not authenticated".to_string())?;

        let url = format!(
            "{}?user_id=eq.{}&select=*",
            self.rest_url("service_subscriptions"), user_id
        );

        let res = Request::get(&url)
            .header("apikey",        &self.anon_key)
            .header("Authorization", &format!("Bearer {}", token))
            .header("Accept",        "application/json")
            .send().await.map_err(|e| format!("Network error: {}", e))?;

        if !res.ok() {
            return Err("Failed to fetch subscriptions".to_string());
        }

        res.json::<Vec<ServiceSubscription>>().await.map_err(|e| format!("Parse error: {}", e))
    }

    pub async fn subscribe_to_service(
        &self, user_id: &str, service_id: &str,
    ) -> Result<ServiceSubscription, String> {
        let token = LocalStorage::get::<String>("mms_access_token")
            .map_err(|_| "Not authenticated".to_string())?;

        let body = SubscribeBody {
            user_id:    user_id.to_string(),
            service_id: service_id.to_string(),
            status:     "active".to_string(),
            tier:       "free".to_string(),
        };

        let res = Request::post(&self.rest_url("service_subscriptions"))
            .header("apikey",        &self.anon_key)
            .header("Authorization", &format!("Bearer {}", token))
            .header("Prefer",        "return=representation")
            .json(&body).map_err(|e| format!("Request build error: {}", e))?
            .send().await.map_err(|e| format!("Network error: {}", e))?;

        if !res.ok() {
            let text = res.text().await.unwrap_or_default();
            return Err(format!("Failed to subscribe: {}", text));
        }

        let subs: Vec<ServiceSubscription> = res.json().await
            .map_err(|e| format!("Parse error: {}", e))?;

        subs.into_iter().next().ok_or_else(|| "No subscription returned".to_string())
    }

    pub async fn update_subscription_status(
        &self, sub_id: &str, user_id: &str, status: &str,
    ) -> Result<(), String> {
        let token = LocalStorage::get::<String>("mms_access_token")
            .map_err(|_| "Not authenticated".to_string())?;

        let body = UpdateSubscriptionBody { status: status.to_string() };
        let url  = format!(
            "{}?id=eq.{}&user_id=eq.{}",
            self.rest_url("service_subscriptions"), sub_id, user_id
        );

        let res = Request::patch(&url)
            .header("apikey",        &self.anon_key)
            .header("Authorization", &format!("Bearer {}", token))
            .json(&body).map_err(|e| format!("Request build error: {}", e))?
            .send().await.map_err(|e| format!("Network error: {}", e))?;

        if !res.ok() {
            let text = res.text().await.unwrap_or_default();
            return Err(format!("Failed to update subscription: {}", text));
        }
        Ok(())
    }

    // ── Registry — user submission methods ─────────────────────

    pub async fn list_user_submissions(&self, user_id: &str) -> Result<Vec<RegistrySubmission>, String> {
        let token = LocalStorage::get::<String>("mms_access_token")
            .map_err(|_| "Not authenticated".to_string())?;

        let url = format!(
            "{}?user_id=eq.{}&order=submitted_at.desc&select=*",
            self.rest_url("registry_submissions"), user_id
        );

        let res = Request::get(&url)
            .header("apikey",        &self.anon_key)
            .header("Authorization", &format!("Bearer {}", token))
            .header("Accept",        "application/json")
            .send().await.map_err(|e| format!("Network error: {}", e))?;

        if !res.ok() {
            return Err("Failed to fetch submissions".to_string());
        }

        res.json::<Vec<RegistrySubmission>>().await.map_err(|e| format!("Parse error: {}", e))
    }

    pub async fn upload_registry_file(
        &self,
        bucket:    &str,
        file_path: &str,
        data:      Vec<u8>,
    ) -> Result<(), String> {
        let token = LocalStorage::get::<String>("mms_access_token")
            .map_err(|_| "Not authenticated".to_string())?;

        let url = self.storage_url(bucket, file_path);
        let arr = Uint8Array::from(data.as_slice());

        let res = Request::post(&url)
            .header("apikey",        &self.anon_key)
            .header("Authorization", &format!("Bearer {}", token))
            .header("Content-Type",  "application/octet-stream")
            .body(arr)
            .map_err(|e| format!("Request build error: {}", e))?
            .send().await.map_err(|e| format!("Network error: {}", e))?;

        if !res.ok() {
            let text = res.text().await.unwrap_or_default();
            return Err(format!("Upload failed: {}", text));
        }
        Ok(())
    }

    pub async fn create_registry_submission(
        &self,
        user_id:      &str,
        mid_id:       &str,
        filename:     &str,
        category:     &str,
        description:  &str,
        tags:         Vec<String>,
        version:      &str,
        storage_path: Option<String>,
    ) -> Result<RegistrySubmission, String> {
        let token = LocalStorage::get::<String>("mms_access_token")
            .map_err(|_| "Not authenticated".to_string())?;

        let tags_json = serde_json::Value::Array(
            tags.into_iter().map(serde_json::Value::String).collect()
        );

        let body = NewSubmissionBody {
            user_id:               user_id.to_string(),
            mid_id:                mid_id.to_string(),
            filename:              filename.to_string(),
            category:              category.to_string(),
            description:           description.to_string(),
            tags:                  tags_json,
            version:               version.to_string(),
            supabase_storage_path: storage_path,
            status:                "pending".to_string(),
        };

        let res = Request::post(&self.rest_url("registry_submissions"))
            .header("apikey",        &self.anon_key)
            .header("Authorization", &format!("Bearer {}", token))
            .header("Prefer",        "return=representation")
            .json(&body).map_err(|e| format!("Request build error: {}", e))?
            .send().await.map_err(|e| format!("Network error: {}", e))?;

        if !res.ok() {
            let text = res.text().await.unwrap_or_default();
            return Err(format!("Failed to submit: {}", text));
        }

        let submissions: Vec<RegistrySubmission> = res.json().await
            .map_err(|e| format!("Parse error: {}", e))?;

        submissions.into_iter().next().ok_or_else(|| "No submission returned".to_string())
    }

    // ── Registry — admin review methods ───────────────────────
    // These use the admin's Supabase JWT directly against the DB.
    // The RLS policy "admin_read_all_submissions" must be created
    // in Supabase for this to return results.

    pub async fn get_pending_submissions(&self) -> Result<Vec<RegistrySubmission>, String> {
        let token = LocalStorage::get::<String>("mms_access_token")
            .map_err(|_| "Not authenticated".to_string())?;

        let url = format!(
            "{}?status=eq.pending&order=submitted_at.desc&select=*",
            self.rest_url("registry_submissions")
        );

        let res = Request::get(&url)
            .header("apikey",        &self.anon_key)
            .header("Authorization", &format!("Bearer {}", token))
            .header("Accept",        "application/json")
            .send().await.map_err(|e| format!("Network error: {}", e))?;

        if !res.ok() {
            return Err(
                "Failed to fetch pending submissions. Ensure admin RLS policy is applied.".to_string()
            );
        }

        res.json::<Vec<RegistrySubmission>>().await
            .map_err(|e| format!("Parse error: {}", e))
    }

    /// Approve a pending submission.
    /// Calls dixscript-docs which fetches the file from Supabase Storage,
    /// copies it to R2 packages/, creates the .meta.json sidecar,
    /// then deletes the file from Supabase Storage and updates the DB record.
    pub async fn approve_submission(&self, submission_id: &str) -> Result<(), String> {
        let token = LocalStorage::get::<String>("mms_access_token")
            .map_err(|_| "Not authenticated".to_string())?;

        let url  = format!("{}/api/registry/approve", DIXSCRIPT_DOCS_URL);
        let body = ApproveBody { submission_id: submission_id.to_string() };

        let res = Request::post(&url)
            .header("Authorization", &format!("Bearer {}", token))
            .header("Content-Type",  "application/json")
            .json(&body).map_err(|e| format!("Request build error: {}", e))?
            .send().await.map_err(|e| format!("Network error: {}", e))?;

        if !res.ok() {
            return Err(parse_docs_error(res).await);
        }
        Ok(())
    }

    /// Reject a pending submission with an optional admin note.
    /// Calls dixscript-docs which deletes the file from Supabase Storage
    /// and updates the DB record with status = rejected.
    pub async fn reject_submission(&self, submission_id: &str, note: &str) -> Result<(), String> {
        let token = LocalStorage::get::<String>("mms_access_token")
            .map_err(|_| "Not authenticated".to_string())?;

        let url  = format!("{}/api/registry/reject", DIXSCRIPT_DOCS_URL);
        let body = RejectBody {
            submission_id: submission_id.to_string(),
            note:          note.to_string(),
        };

        let res = Request::post(&url)
            .header("Authorization", &format!("Bearer {}", token))
            .header("Content-Type",  "application/json")
            .json(&body).map_err(|e| format!("Request build error: {}", e))?
            .send().await.map_err(|e| format!("Network error: {}", e))?;

        if !res.ok() {
            return Err(parse_docs_error(res).await);
        }
        Ok(())
    }

    // ── R2 file management methods ─────────────────────────────
    // These call dixscript-docs which performs operations on the
    // Cloudflare R2 bucket. All require admin JWT.

    pub async fn list_r2_files(&self, prefix: &str) -> Result<Vec<R2FileInfo>, String> {
        let token = LocalStorage::get::<String>("mms_access_token")
            .map_err(|_| "Not authenticated".to_string())?;

        let prefix_enc = js_sys::encode_uri_component(prefix)
            .as_string()
            .unwrap_or_else(|| prefix.to_string());

        let url = format!(
            "{}/api/admin/r2/list?prefix={}",
            DIXSCRIPT_DOCS_URL, prefix_enc
        );

        let res = Request::get(&url)
            .header("Authorization", &format!("Bearer {}", token))
            .send().await.map_err(|e| format!("Network error: {}", e))?;

        if !res.ok() {
            return Err(parse_docs_error(res).await);
        }

        #[derive(Deserialize)]
        struct ListResp { files: Vec<R2FileInfo> }

        let resp: ListResp = res.json().await
            .map_err(|e| format!("Parse error: {}", e))?;

        Ok(resp.files)
    }

    /// Delete a file from R2 by its full key (e.g. "packages/game/base.mdix").
    /// Also deletes the .meta.json sidecar if present.
    pub async fn delete_r2_file(&self, key: &str) -> Result<(), String> {
        let token = LocalStorage::get::<String>("mms_access_token")
            .map_err(|_| "Not authenticated".to_string())?;

        let url  = format!("{}/api/admin/r2/delete", DIXSCRIPT_DOCS_URL);
        let body = DeleteR2Body { key: key.to_string() };

        let res = Request::post(&url)
            .header("Authorization", &format!("Bearer {}", token))
            .header("Content-Type",  "application/json")
            .json(&body).map_err(|e| format!("Request build error: {}", e))?
            .send().await.map_err(|e| format!("Network error: {}", e))?;

        if !res.ok() {
            return Err(parse_docs_error(res).await);
        }
        Ok(())
    }

    /// Move/rename a file in R2.
    /// Copies content to the new key then deletes the old key.
    /// Also moves the .meta.json sidecar if present.
    pub async fn move_r2_file(&self, from_key: &str, to_key: &str) -> Result<(), String> {
        let token = LocalStorage::get::<String>("mms_access_token")
            .map_err(|_| "Not authenticated".to_string())?;

        let url  = format!("{}/api/admin/r2/move", DIXSCRIPT_DOCS_URL);
        let body = MoveR2Body {
            from_key: from_key.to_string(),
            to_key:   to_key.to_string(),
        };

        let res = Request::post(&url)
            .header("Authorization", &format!("Bearer {}", token))
            .header("Content-Type",  "application/json")
            .json(&body).map_err(|e| format!("Request build error: {}", e))?
            .send().await.map_err(|e| format!("Network error: {}", e))?;

        if !res.ok() {
            return Err(parse_docs_error(res).await);
        }
        Ok(())
    }

    // ── Session helpers ────────────────────────────────────────

    fn persist_session(session: &AuthSession) {
        let _ = LocalStorage::set("mms_access_token",  &session.access_token);
        let _ = LocalStorage::set("mms_refresh_token", &session.refresh_token);
        let _ = LocalStorage::set("mms_user_id",       &session.user.id);
    }
}
