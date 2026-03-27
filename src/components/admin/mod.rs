pub mod stats;
pub mod user_table;

pub use stats::*;
pub use user_table::*;

use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;
use gloo_storage::Storage;
use crate::supabase::{SupabaseClient, Profile};
use crate::components::icons::{IconShield, IconLogOut};

#[component]
pub fn AdminPage() -> impl IntoView {
    if !SupabaseClient::is_logged_in() {
        if let Some(window) = web_sys::window() {
            let _ = window.location().set_hash("auth");
        }
        return view! { <div></div> }.into_any();
    }

    let (profiles,  set_profiles)  = signal(Vec::<Profile>::new());
    let (loading,   set_loading)   = signal(true);
    let (error,     set_error)     = signal(String::new());
    let (is_admin,  set_is_admin)  = signal(false);

    Effect::new(move |_| {
        let user_id = gloo_storage::LocalStorage::get::<String>("mms_user_id")
            .unwrap_or_default();
        let client  = SupabaseClient::new();

        spawn_local(async move {
            match client.get_profile(&user_id).await {
                Ok(p) if p.is_admin() => {
                    set_is_admin.set(true);
                    match client.get_all_profiles().await {
                        Ok(all) => {
                            set_profiles.set(all);
                            set_loading.set(false);
                        }
                        Err(e) => {
                            set_error.set(e);
                            set_loading.set(false);
                        }
                    }
                }
                Ok(_) => {
                    if let Some(window) = web_sys::window() {
                        let _ = window.location().set_hash("dashboard");
                    }
                }
                Err(_) => {
                    SupabaseClient::clear_session();
                    if let Some(window) = web_sys::window() {
                        let _ = window.location().set_hash("auth");
                    }
                }
            }
        });
    });

    let handle_signout = move |_| {
        let client = SupabaseClient::new();
        spawn_local(async move {
            let _ = client.sign_out().await;
            if let Some(window) = web_sys::window() {
                let _ = window.location().set_hash("auth");
            }
        });
    };

    view! {
        <div class="admin-layout">
            <header class="admin-header">
                <div class="admin-header-inner">
                    <div class="admin-header-brand">
                        <IconShield class="icon-svg icon-sm" />
                        <span class="admin-header-title">"MmS Admin"</span>
                    </div>
                    <div class="admin-header-actions">
                        <a href="#dashboard" class="btn btn-ghost btn-sm">
                            "Dashboard"
                        </a>
                        <button class="btn btn-ghost btn-sm" on:click=handle_signout>
                            <IconLogOut class="icon-svg icon-sm" />
                            "Sign Out"
                        </button>
                    </div>
                </div>
            </header>

            <main class="admin-main">
                {move || if loading.get() {
                    view! {
                        <div class="admin-loading">
                            <div class="spinner"></div>
                        </div>
                    }.into_any()
                } else if !error.get().is_empty() {
                    view! {
                        <div class="admin-error">
                            <div class="status-msg status-msg--error">{error.get()}</div>
                        </div>
                    }.into_any()
                } else if !is_admin.get() {
                    view! {
                        <div class="admin-error">
                            <div class="status-msg status-msg--error">
                                "Access denied. Admin role required."
                            </div>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <div class="admin-content">
                            <div class="admin-page-header">
                                <h1 class="admin-page-title">"Admin Dashboard"</h1>
                                <p class="admin-page-sub">"Manage MidManStudio accounts."</p>
                            </div>
                            <AdminStats profiles=profiles />
                            <UserTable profiles=profiles />
                        </div>
                    }.into_any()
                }}
            </main>
        </div>
    }.into_any()
}
