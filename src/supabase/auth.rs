use super::client::*;
use serde::Serialize;
use gloo_storage::{LocalStorage, Storage};

#[derive(Serialize)]
struct SignUpRequest {
    email: String,
    password: String,
    data: serde_json::Value,
}

#[derive(Serialize)]
struct SignInRequest {
    email: String,
    password: String,
}

impl SupabaseClient {
    pub async fn sign_up(
        &self,
        email: &str,
        password: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<AuthSession, String> {
        let body = SignUpRequest {
            email: email.to_string(),
            password: password.to_string(),
            data: metadata.unwrap_or(serde_json::json!({})),
        };

        let res = self.client
            .post(&self.auth_url("/signup"))
            .header("apikey", &self.anon_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        if !res.status().is_success() {
            let err_text = res.text().await.unwrap_or_default();
            return Err(format!("Signup failed: {}", err_text));
        }

        let session: AuthSession = res.json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;

        let _ = LocalStorage::set("supabase_token", &session.access_token);
        
        Ok(session)
    }

    pub async fn sign_in(&self, email: &str, password: &str) -> Result<AuthSession, String> {
        let body = SignInRequest {
            email: email.to_string(),
            password: password.to_string(),
        };

        let res = self.client
            .post(&self.auth_url("/token?grant_type=password"))
            .header("apikey", &self.anon_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        if !res.status().is_success() {
            let err_text = res.text().await.unwrap_or_default();
            return Err(format!("Login failed: {}", err_text));
        }

        let session: AuthSession = res.json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;

        let _ = LocalStorage::set("supabase_token", &session.access_token);
        
        Ok(session)
    }

    pub fn sign_out(&self) {
        let _ = LocalStorage::delete("supabase_token");
    }

    pub async fn get_user(&self) -> Result<User, String> {
        let token = LocalStorage::get::<String>("supabase_token")
            .map_err(|_| "No token found")?;

        let res = self.client
            .get(&self.auth_url("/user"))
            .header("apikey", &self.anon_key)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        if !res.status().is_success() {
            return Err("Not authenticated".to_string());
        }

        let user: User = res.json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;

        Ok(user)
    }
}
