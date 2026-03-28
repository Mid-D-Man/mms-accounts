use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;

use crate::components::{
    landing::LandingPage,
    auth::AuthPage,
    dashboard::DashboardPage,
    admin::AdminPage,
};
use crate::supabase::SupabaseClient;

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
        match hash.trim_start_matches('#') {
            "auth"      => Self::Auth,
            "dashboard" => Self::Dashboard,
            "admin"     => Self::Admin,
            _           => Self::Landing,
        }
    }
}

// ── App root ───────────────────────────────────────────────────

#[component]
pub fn App() -> impl IntoView {
    let (route,   set_route)   = signal(get_current_route());
    // true while we are attempting the initial session refresh
    let (booting, set_booting) = signal(true);

    // ── Session bootstrap ──────────────────────────────────────
    // On every cold load / refresh: if a refresh token exists in
    // localStorage, hit Supabase to get a fresh access token before
    // rendering any protected page. This means the user stays logged
    // in across browser restarts and tab closes.
    Effect::new(move |_| {
        let client = SupabaseClient::new();
        spawn_local(async move {
            match client.try_refresh_session().await {
                Ok(true) => {
                    // Refreshed successfully — stay on whatever route the
                    // hash says (dashboard / admin / landing).
                }
                Ok(false) => {
                    // No refresh token — first visit or explicitly logged out.
                    // If the hash points at a protected page, bounce to auth.
                    let current = get_current_route();
                    if current == Route::Dashboard || current == Route::Admin {
                        set_route.set(Route::Auth);
                        if let Some(w) = web_sys::window() {
                            let _ = w.location().set_hash("auth");
                        }
                    }
                }
                Err(_) => {
                    // Refresh token was expired/revoked — clear and bounce.
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
            {move || if booting.get() {
                // Keep the existing WASM loading screen visible while we
                // verify the session — no flash of wrong content.
                view! { <div></div> }.into_any()
            } else {
                match route.get() {
                    Route::Landing   => view! { <LandingPage /> }.into_any(),
                    Route::Auth      => view! { <AuthPage /> }.into_any(),
                    Route::Dashboard => view! { <DashboardPage /> }.into_any(),
                    Route::Admin     => view! { <AdminPage /> }.into_any(),
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
