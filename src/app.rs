use leptos::prelude::*;
use wasm_bindgen::JsCast;

use crate::components::{
    landing::LandingPage,
    auth::AuthPage,
    dashboard::DashboardPage,
    admin::AdminPage,
};

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
    let (route, set_route) = signal(get_current_route());

    // Listen for hash changes
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
            {move || match route.get() {
                Route::Landing   => view! { <LandingPage /> }.into_any(),
                Route::Auth      => view! { <AuthPage /> }.into_any(),
                Route::Dashboard => view! { <DashboardPage /> }.into_any(),
                Route::Admin     => view! { <AdminPage /> }.into_any(),
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
