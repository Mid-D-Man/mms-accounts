pub mod sidebar;
pub mod header;
pub mod overview;
pub mod profile;

pub use sidebar::*;
pub use header::*;
pub use overview::*;
pub use profile::*;

use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;
use gloo_storage::Storage;
use crate::supabase::{SupabaseClient, Profile};

#[derive(Clone, PartialEq)]
pub enum DashView {
    Overview,
    Profile,
    Settings,
}

#[component]
pub fn DashboardPage() -> impl IntoView {
    if !SupabaseClient::is_logged_in() {
        if let Some(window) = web_sys::window() {
            let _ = window.location().set_hash("auth");
        }
        return view! { <div></div> }.into_any();
    }

    let (profile,      set_profile)      = signal(None::<Profile>);
    let (loading,      set_loading)      = signal(true);
    let (active_view,  set_active_view)  = signal(DashView::Overview);

    Effect::new(move |_| {
        let user_id = gloo_storage::LocalStorage::get::<String>("mms_user_id")
            .unwrap_or_default();

        if user_id.is_empty() {
            set_loading.set(false);
            return;
        }

        let client = SupabaseClient::new();

        spawn_local(async move {
            match client.get_profile(&user_id).await {
                Ok(p)  => {
                    set_profile.set(Some(p));
                    set_loading.set(false);
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

    view! {
        <div class="dashboard-layout">
            <DashboardSidebar
                active_view=active_view
                on_navigate=move |v| set_active_view.set(v)
                profile=profile
            />

            <div class="dashboard-main">
                <DashboardHeader profile=profile />

                <main class="dashboard-content">
                    {move || if loading.get() {
                        view! {
                            <div class="dashboard-loading">
                                <div class="spinner-wrap">
                                    <div class="spinner"></div>
                                </div>
                            </div>
                        }.into_any()
                    } else {
                        match active_view.get() {
                            DashView::Overview => view! {
                                <OverviewView profile=profile />
                            }.into_any(),
                            DashView::Profile | DashView::Settings => view! {
                                <ProfileView
                                    profile=profile
                                    on_updated=move |p| set_profile.set(Some(p))
                                />
                            }.into_any(),
                        }
                    }}
                </main>
            </div>
        </div>
    }.into_any()
                        }
