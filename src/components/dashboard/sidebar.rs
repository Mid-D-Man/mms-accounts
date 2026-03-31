use leptos::prelude::*;
use crate::supabase::Profile;
use crate::components::icons::{
    IconHome, IconUser, IconSettings, IconLogOut, IconShield, IconKey,
};
use super::DashView;

#[component]
pub fn DashboardSidebar(
    active_view: ReadSignal<DashView>,
    on_navigate: impl Fn(DashView) + 'static + Clone,
    profile:     ReadSignal<Option<Profile>>,
) -> impl IntoView {
    let nav = std::sync::Arc::new(on_navigate);

    let nav_overview     = { let n = nav.clone(); move |_| n(DashView::Overview)     };
    let nav_profile      = { let n = nav.clone(); move |_| n(DashView::Profile)      };
    let nav_credentials  = { let n = nav.clone(); move |_| n(DashView::Credentials)  };
    let nav_settings     = { let n = nav.clone(); move |_| n(DashView::Settings)     };

    let handle_signout = move |_| {
        let client = crate::supabase::SupabaseClient::new();
        wasm_bindgen_futures::spawn_local(async move {
            let _ = client.sign_out().await;
            if let Some(window) = web_sys::window() {
                let _ = window.location().set_hash("auth");
            }
        });
    };

    view! {
        <aside class="dash-sidebar">

            // ── Brand ──────────────────────────────────────────
            <div class="dash-sidebar-brand">
                <img
                    src="/logo.png"
                    alt="MidManStudio"
                    class="dash-sidebar-logo"
                    on:error=move |ev| {
                        use wasm_bindgen::JsCast;
                        if let Some(img) = ev.target()
                            .and_then(|t| t.dyn_into::<web_sys::HtmlImageElement>().ok())
                        {
                            // Use set_attribute to avoid conflict with Leptos's style() extension trait
                            let _ = img.set_attribute("style", "display:none");
                        }
                    }
                />
                <div class="dash-sidebar-brand-text">
                    <span class="brand-mms">"MmS"</span>
                    <span class="brand-accounts">"Accounts"</span>
                </div>
            </div>

            // ── User info ──────────────────────────────────────
            <div class="dash-sidebar-user">
                <div class="sidebar-avatar">
                    {move || {
                        let name = profile.get()
                            .as_ref()
                            .map(|p| p.display_name_or_email())
                            .unwrap_or_default();
                        let initial = name.chars().next()
                            .unwrap_or('?')
                            .to_uppercase()
                            .to_string();
                        view! { <span class="avatar-initial">{initial}</span> }
                    }}
                </div>
                <div class="sidebar-user-info">
                    <span class="sidebar-user-name">
                        {move || profile.get()
                            .as_ref()
                            .map(|p| p.display_name_or_email())
                            .unwrap_or_else(|| "Loading...".to_string())}
                    </span>
                    {move || if profile.get().as_ref().map(|p| p.is_admin()).unwrap_or(false) {
                        view! {
                            <span class="sidebar-role-badge">
                                <IconShield class="icon-svg icon-xs" />
                                "Admin"
                            </span>
                        }.into_any()
                    } else {
                        view! { <span class="sidebar-role-badge">"User"</span> }.into_any()
                    }}
                </div>
            </div>

            // ── Nav ────────────────────────────────────────────
            <nav class="dash-nav">
                <button
                    class=move || if active_view.get() == DashView::Overview {
                        "dash-nav-item dash-nav-item--active"
                    } else { "dash-nav-item" }
                    on:click=nav_overview
                >
                    <IconHome class="icon-svg icon-sm" />
                    <span>"Overview"</span>
                </button>

                <button
                    class=move || if active_view.get() == DashView::Profile {
                        "dash-nav-item dash-nav-item--active"
                    } else { "dash-nav-item" }
                    on:click=nav_profile
                >
                    <IconUser class="icon-svg icon-sm" />
                    <span>"Profile"</span>
                </button>

                <button
                    class=move || if active_view.get() == DashView::Credentials {
                        "dash-nav-item dash-nav-item--active"
                    } else { "dash-nav-item" }
                    on:click=nav_credentials
                >
                    <IconKey class="icon-svg icon-sm" />
                    <span>"Credentials"</span>
                </button>

                <button
                    class=move || if active_view.get() == DashView::Settings {
                        "dash-nav-item dash-nav-item--active"
                    } else { "dash-nav-item" }
                    on:click=nav_settings
                >
                    <IconSettings class="icon-svg icon-sm" />
                    <span>"Settings"</span>
                </button>
            </nav>

            // ── Bottom ─────────────────────────────────────────
            <div class="dash-sidebar-bottom">
                {move || if profile.get().as_ref().map(|p| p.is_admin()).unwrap_or(false) {
                    view! {
                        <a href="#admin" class="dash-nav-item dash-nav-item--admin">
                            <IconShield class="icon-svg icon-sm" />
                            <span>"Admin Panel"</span>
                        </a>
                    }.into_any()
                } else {
                    view! { <span></span> }.into_any()
                }}

                <button class="dash-nav-item dash-nav-item--signout" on:click=handle_signout>
                    <IconLogOut class="icon-svg icon-sm" />
                    <span>"Sign Out"</span>
                </button>
            </div>

        </aside>
    }
}
