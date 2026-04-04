pub mod sidebar;
pub mod header;
pub mod overview;
pub mod profile;
pub mod settings;
pub mod credentials;
pub mod services;
pub mod registry;
pub mod admin_users;
pub mod admin_registry;
pub mod admin_r2;
pub mod admin_permissions;

pub use sidebar::*;
pub use header::*;
pub use overview::*;
pub use profile::*;
pub use settings::*;
pub use credentials::*;
pub use services::*;
pub use registry::*;
pub use admin_users::*;
pub use admin_registry::*;
pub use admin_r2::*;
pub use admin_permissions::*;

use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;
use gloo_storage::Storage;
use crate::supabase::{SupabaseClient, Profile};
use crate::components::icons::IconMenu;

#[derive(Clone, PartialEq)]
pub enum DashView {
    // ── User views ─────────────────────────────────────────────
    Overview,
    Profile,
    Settings,
    Credentials,
    Services,
    Registry,
    // ── Admin-only views ───────────────────────────────────────
    AdminUsers,
    AdminRegistry,
    AdminR2,
    AdminPermissions,
}

#[component]
pub fn DashboardPage() -> impl IntoView {
    if !SupabaseClient::is_logged_in() {
        if let Some(window) = web_sys::window() {
            let _ = window.location().set_hash("auth");
        }
        return view! { <div></div> }.into_any();
    }

    let (profile,         set_profile)         = signal(None::<Profile>);
    let (loading,         set_loading)         = signal(true);
    let (active_view,     set_active_view)     = signal(DashView::Overview);
    let (mobile_nav_open, set_mobile_nav_open) = signal(false);

    Effect::new(move |_| {
        let user_id = gloo_storage::LocalStorage::get::<String>("mms_user_id")
            .unwrap_or_default();
        if user_id.is_empty() { set_loading.set(false); return; }
        let client = SupabaseClient::new();
        spawn_local(async move {
            match client.get_profile(&user_id).await {
                Ok(p)  => { set_profile.set(Some(p)); set_loading.set(false); }
                Err(_) => {
                    SupabaseClient::clear_session();
                    if let Some(window) = web_sys::window() {
                        let _ = window.location().set_hash("auth");
                    }
                }
            }
        });
    });

    view! {
        <div class="dashboard-layout">
            <button
                class="mobile-hamburger"
                on:click=move |_| set_mobile_nav_open.set(true)
                aria-label="Open navigation"
            >
                <IconMenu class="icon-svg" />
            </button>

            <DashboardSidebar
                active_view=active_view
                on_navigate=move |v| set_active_view.set(v)
                profile=profile
                mobile_open=mobile_nav_open
                set_mobile_open=set_mobile_nav_open
            />

            <div class="dashboard-main">
                <DashboardHeader profile=profile />
                <main class="dashboard-content">
                    {move || if loading.get() {
                        view! {
                            <div class="dashboard-loading">
                                <div class="spinner-wrap"><div class="spinner"></div></div>
                            </div>
                        }.into_any()
                    } else {
                        match active_view.get() {
                            DashView::Overview    => view! { <OverviewView profile=profile /> }.into_any(),
                            DashView::Profile     => view! {
                                <ProfileView
                                    profile=profile
                                    on_updated=move |p| set_profile.set(Some(p))
                                />
                            }.into_any(),
                            DashView::Settings    => view! { <SettingsView /> }.into_any(),
                            DashView::Credentials => view! { <CredentialsView profile=profile /> }.into_any(),
                            DashView::Services    => view! { <ServicesView /> }.into_any(),
                            DashView::Registry    => view! { <RegistryView profile=profile /> }.into_any(),
                            // ── Admin-only ─────────────────────
                            DashView::AdminUsers =>
                                view! { <AdminUsersView profile=profile /> }.into_any(),
                            DashView::AdminRegistry =>
                                view! { <AdminRegistryView profile=profile /> }.into_any(),
                            DashView::AdminR2 =>
                                view! { <AdminR2View profile=profile /> }.into_any(),
                            DashView::AdminPermissions =>
                                view! { <AdminPermissionsView profile=profile /> }.into_any(),
                        }
                    }}
                </main>
            </div>
        </div>
    }.into_any()
}
