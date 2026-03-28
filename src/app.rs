use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use std::collections::HashMap;

use crate::components::{
    landing::LandingPage,
    auth::{AuthPage, ResetPasswordForm},
    dashboard::DashboardPage,
    admin::AdminPage,
};
use crate::supabase::{SupabaseClient, User};
use crate::supabase::client::{SUPABASE_URL, SUPABASE_ANON_KEY};

// ── Route enum ─────────────────────────────────────────────────

#[derive(Clone, PartialEq, Debug)]
pub enum Route {
    Landing,
    Auth,
    Dashboard,
    Admin,
}

impl Route {
    pub fn from_hash(hash: &str) -> Self {
        // Strip hash fragment params — Supabase OAuth/recovery URLs look like
        // #access_token=...&type=recovery, which starts with a token, not a route.
        // We only route on clean single-word hashes like #dashboard.
        let route_part = hash
            .trim_start_matches('#')
            .split('&')
            .next()
            .unwrap_or("");

        // If it looks like a token (very long) or contains '=', it's not a route
        if route_part.contains('=') || route_part.len() > 20 {
            return Self::Landing;
        }

        match route_part {
            "auth"      => Self::Auth,
            "dashboard" => Self::Dashboard,
            "admin"     => Self::Admin,
            _           => Self::Landing,
        }
    }
}

// ── Hash fragment parser ───────────────────────────────────────
// Supabase returns sessions and recovery tokens as URL hash params:
// #access_token=xxx&refresh_token=yyy&type=recovery

fn parse_hash_params() -> HashMap<String, String> {
    let hash = web_sys::window()
        .and_then(|w| w.location().hash().ok())
        .unwrap_or_default();

    let fragment = hash.trim_start_matches('#');
    let mut map = HashMap::new();

    for pair in fragment.split('&') {
        if let Some((k, v)) = pair.split_once('=') {
            map.insert(k.to_string(), v.to_string());
        }
    }
    map
}

// ── App root ───────────────────────────────────────────────────

#[component]
pub fn App() -> impl IntoView {
    let (route,          set_route)          = signal(get_current_route());
    let (booting,        set_booting)        = signal(true);
    // When Some, we show ResetPasswordForm instead of normal routing.
    // The String is the short-lived recovery access_token from the email link.
    let (recovery_token, set_recovery_token) = signal(None::<String>);

    // ── Boot: detect OAuth/recovery callbacks, then refresh ───
    Effect::new(move |_| {
        let params = parse_hash_params();

        // ── Case 1: Password recovery link from email ──────────
        // Supabase email links land here with type=recovery in the hash.
        if params.get("type").map(|s| s.as_str()) == Some("recovery") {
            if let Some(token) = params.get("access_token").cloned() {
                // Store the token for ResetPasswordForm — never write to
                // localStorage since it expires after one use.
                set_recovery_token.set(Some(token));
                // Replace the hash so the token isn't sitting in the address bar.
                if let Some(w) = web_sys::window() {
                    let _ = w.location().set_hash("auth");
                }
                set_booting.set(false);
                return;
            }
        }

        // ── Case 2: OAuth callback (Google, GitHub etc.) ───────
        // Supabase redirects back with access_token in the hash but no type.
        if let (Some(access_token), Some(refresh_token)) = (
            params.get("access_token").cloned(),
            params.get("refresh_token").cloned(),
        ) {
            let at = access_token.clone();
            let rt = refresh_token.clone();
            let anon = SUPABASE_ANON_KEY.to_string();
            let base = SUPABASE_URL.to_string();

            spawn_local(async move {
                // Persist the tokens immediately
                let _ = gloo_storage::LocalStorage::set("mms_access_token",  &at);
                let _ = gloo_storage::LocalStorage::set("mms_refresh_token", &rt);

                // Fetch the user ID — needed for profile queries
                let res = gloo_net::http::Request::get(&format!("{}/auth/v1/user", base))
                    .header("apikey",        &anon)
                    .header("Authorization", &format!("Bearer {}", at))
                    .send()
                    .await;

                if let Ok(r) = res {
                    if let Ok(user) = r.json::<User>().await {
                        let _ = gloo_storage::LocalStorage::set("mms_user_id", &user.id);
                    }
                }

                // Clear the OAuth params from the URL and route to dashboard
                if let Some(w) = web_sys::window() {
                    let _ = w.location().set_hash("dashboard");
                }
                set_route.set(Route::Dashboard);
                set_booting.set(false);
            });
            return;
        }

        // ── Case 3: Normal load — try to refresh existing session
        let client = SupabaseClient::new();
        spawn_local(async move {
            match client.try_refresh_session().await {
                Ok(true) => {
                    // Session renewed — stay on whatever hash says
                }
                Ok(false) => {
                    // No token in storage — first visit or logged out.
                    // Bounce protected routes back to auth.
                    let current = get_current_route();
                    if current == Route::Dashboard || current == Route::Admin {
                        set_route.set(Route::Auth);
                        if let Some(w) = web_sys::window() {
                            let _ = w.location().set_hash("auth");
                        }
                    }
                }
                Err(_) => {
                    // Refresh token expired/revoked
                    SupabaseClient::clear_session();
                    set_route.set(Route::Auth);
                    if let Some(w) = web_sys::window() {
                        let _ = w.location().set_hash("auth");
                    }
                }
            }
            set_booting.set(false);
        });
    });

    // ── Hash-change listener ───────────────────────────────────
    Effect::new(move |_| {
        if let Some(window) = web_sys::window() {
            let set_r = set_route.clone();
            let closure = wasm_bindgen::closure::Closure::wrap(
                Box::new(move || {
                    set_r.set(get_current_route());
                }) as Box<dyn Fn()>
            );
            let _ = window.add_event_listener_with_callback(
                "hashchange",
                closure.as_ref().unchecked_ref(),
            );
            closure.forget();
        }
    });

    view! {
        <div class="mms-app">
            {move || {
                if booting.get() {
                    // Keep the WASM loading screen visible during boot checks.
                    view! { <div></div> }.into_any()
                } else if let Some(token) = recovery_token.get() {
                    // Password recovery mode — overrides normal routing entirely.
                    // Once the user resets their password, on_done clears this.
                    view! {
                        <ResetPasswordForm
                            recovery_token=token
                            on_done=move || {
                                set_recovery_token.set(None);
                                if let Some(w) = web_sys::window() {
                                    let _ = w.location().set_hash("auth");
                                }
                                set_route.set(Route::Auth);
                            }
                        />
                    }.into_any()
                } else {
                    match route.get() {
                        Route::Landing   => view! { <LandingPage />   }.into_any(),
                        Route::Auth      => view! { <AuthPage />      }.into_any(),
                        Route::Dashboard => view! { <DashboardPage /> }.into_any(),
                        Route::Admin     => view! { <AdminPage />     }.into_any(),
                    }
                }
            }}
        </div>
    }
}

// ── Helpers ────────────────────────────────────────────────────

fn get_current_route() -> Route {
    let hash = web_sys::window()
        .and_then(|w| w.location().hash().ok())
        .unwrap_or_default();
    Route::from_hash(&hash)
}

pub fn navigate(hash: &str) {
    if let Some(window) = web_sys::window() {
        let _ = window.location().set_hash(hash);
    }
}
