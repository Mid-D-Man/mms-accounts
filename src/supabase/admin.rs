// src/supabase/admin.rs
// Admin-specific SupabaseClient methods.
// Multiple impl blocks for the same type are valid in Rust — auth.rs holds
// the auth/user operations; this file holds admin-only operations.

use super::client::*;
use serde::Serialize;
use gloo_storage::{LocalStorage, Storage};
use gloo_net::http::Request;

#[derive(Serialize)]
struct UpdateRoleBody {
    role: String,
}

impl SupabaseClient {
    // ── Role management ────────────────────────────────────────

    /// Change any user's role (user ↔ admin).
    /// Requires the "admin_update_any_profile" RLS policy.
    /// Client-side self-change guard is a UX convenience — RLS is the
    /// authoritative check server-side.
    pub async fn update_user_role(
        &self,
        target_user_id: &str,
        new_role:        &str,
    ) -> Result<(), String> {
        let token = LocalStorage::get::<String>("mms_access_token")
            .map_err(|_| "Not authenticated".to_string())?;

        // Prevent admin from demoting/changing themselves in the UI
        let my_id = LocalStorage::get::<String>("mms_user_id").unwrap_or_default();
        if my_id == target_user_id {
            return Err("Cannot change your own role.".to_string());
        }

        let body = UpdateRoleBody { role: new_role.to_string() };
        let url  = format!("{}?id=eq.{}", self.rest_url("profiles"), target_user_id);

        let res = Request::patch(&url)
            .header("apikey",        &self.anon_key)
            .header("Authorization", &format!("Bearer {}", token))
            .header("Content-Type",  "application/json")
            .json(&body).map_err(|e| format!("Request build error: {}", e))?
            .send().await.map_err(|e| format!("Network error: {}", e))?;

        if !res.ok() {
            let text = res.text().await.unwrap_or_default();
            return Err(format!("Role update failed: {}", text));
        }
        Ok(())
    }

    // ── Submission file preview ────────────────────────────────

    /// Download the raw .mdix content of a pending submission from
    /// Supabase Storage for admin preview.
    ///
    /// `supabase_storage_path` is the full bucket/key stored on the
    /// RegistrySubmission row, e.g.
    ///   "registry-pending/{mid_id}/{timestamp}_{file}.mdix"
    pub async fn download_submission_file(
        &self,
        supabase_storage_path: &str,
    ) -> Result<String, String> {
        let token = LocalStorage::get::<String>("mms_access_token")
            .map_err(|_| "Not authenticated".to_string())?;

        // The storage URL includes the bucket name as part of the path
        let url = format!(
            "{}/storage/v1/object/{}",
            self.url, supabase_storage_path
        );

        let res = Request::get(&url)
            .header("apikey",        &self.anon_key)
            .header("Authorization", &format!("Bearer {}", token))
            .send().await.map_err(|e| format!("Network error: {}", e))?;

        if !res.ok() {
            let status = res.status();
            let body   = res.text().await.unwrap_or_default();
            return Err(format!("Download failed ({}): {}", status, body));
        }

        res.text().await.map_err(|e| format!("Read error: {}", e))
    }
}
