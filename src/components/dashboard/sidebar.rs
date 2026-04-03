use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;
use crate::supabase::Profile;
use crate::components::icons::{
    IconHome, IconUser, IconSettings, IconLogOut, IconShield, IconKey,
    IconLayers, IconPackage, IconUsers, IconChevronsRight, IconChevronsLeft,
};
use super::DashView;

#[component]
pub fn DashboardSidebar(
    active_view:     ReadSignal<DashView>,
    on_navigate:     impl Fn(DashView) + 'static + Clone + Send + Sync,
    profile:         ReadSignal<Option<Profile>>,
    mobile_open:     ReadSignal<bool>,
    set_mobile_open: WriteSignal<bool>,
) -> impl IntoView {
    let (is_expanded, set_is_expanded) = signal(false);
    let (is_pinned,   set_is_pinned)   = signal(false);

    let handle_mouse_enter = move |_: web_sys::MouseEvent| {
        if !is_pinned.get() { set_is_expanded.set(true); }
    };
    let handle_mouse_leave = move |_: web_sys::MouseEvent| {
        if !is_pinned.get() { set_is_expanded.set(false); }
    };
    let toggle_pin = move |_: web_sys::MouseEvent| {
        let new_pinned = !is_pinned.get();
        set_is_pinned.set(new_pinned);
        if !new_pinned { set_is_expanded.set(false); }
    };

    let nav = std::sync::Arc::new(on_navigate);

    // ── One closure per destination ────────────────────────────
    let nav_overview       = { let n = nav.clone(); move |_: web_sys::MouseEvent| { n(DashView::Overview);       set_mobile_open.set(false); } };
    let nav_profile        = { let n = nav.clone(); move |_: web_sys::MouseEvent| { n(DashView::Profile);        set_mobile_open.set(false); } };
    let nav_creds          = { let n = nav.clone(); move |_: web_sys::MouseEvent| { n(DashView::Credentials);    set_mobile_open.set(false); } };
    let nav_services       = { let n = nav.clone(); move |_: web_sys::MouseEvent| { n(DashView::Services);       set_mobile_open.set(false); } };
    let nav_registry       = { let n = nav.clone(); move |_: web_sys::MouseEvent| { n(DashView::Registry);       set_mobile_open.set(false); } };
    let nav_settings       = { let n = nav.clone(); move |_: web_sys::MouseEvent| { n(DashView::Settings);       set_mobile_open.set(false); } };
    // Admin-only
    let nav_admin_users    = { let n = nav.clone(); move |_: web_sys::MouseEvent| { n(DashView::AdminUsers);     set_mobile_open.set(false); } };
    let nav_admin_registry = { let n = nav.clone(); move |_: web_sys::MouseEvent| { n(DashView::AdminRegistry);  set_mobile_open.set(false); } };
    // nav_admin_r2 and nav_admin_permissions added in response 3

    let handle_signout = move |_: web_sys::MouseEvent| {
        let client = crate::supabase::SupabaseClient::new();
        spawn_local(async move {
            let _ = client.sign_out().await;
            if let Some(window) = web_sys::window() {
                let _ = window.location().set_hash("auth");
            }
        });
    };

    let show_expanded = move || is_expanded.get() || is_pinned.get() || mobile_open.get();

    view! {
        <>
        {move || if mobile_open.get() {
            view! {
                <div class="sidebar-mobile-overlay"
                     on:click=move |_| set_mobile_open.set(false)></div>
            }.into_any()
        } else { view! { <span></span> }.into_any() }}

        <aside
            class=move || {
                let mut c = "dash-sidebar".to_string();
                if show_expanded()   { c.push_str(" expanded"); }
                if mobile_open.get() { c.push_str(" mobile-open"); }
                c
            }
            on:mouseenter=handle_mouse_enter
            on:mouseleave=handle_mouse_leave
        >
            // ── Header ────────────────────────────────────────
            <div class="sidebar-header">
                <div class="sidebar-brand">
                    <img
                        src="/logo.png" alt="MmS" class="sidebar-logo-img"
                        on:error=move |ev| {
                            use wasm_bindgen::JsCast;
                            if let Some(img) = ev.target()
                                .and_then(|t| t.dyn_into::<web_sys::HtmlImageElement>().ok())
                            { let _ = img.set_attribute("style", "display:none"); }
                        }
                    />
                    <div class="sidebar-brand-text">
                        <span class="brand-mms">"MmS"</span>
                        <span class="brand-accounts">"Accounts"</span>
                    </div>
                </div>
                <button class="sidebar-pin-btn" on:click=toggle_pin
                        title=move || if is_pinned.get() { "Unpin" } else { "Pin open" }>
                    {move || if is_pinned.get() {
                        view! { <IconChevronsLeft  class="icon-svg icon-sm" /> }.into_any()
                    } else {
                        view! { <IconChevronsRight class="icon-svg icon-sm" /> }.into_any()
                    }}
                </button>
            </div>

            // ── User info ──────────────────────────────────────
            <div class="sidebar-user">
                <div class="sidebar-avatar">
                    {move || {
                        let name = profile.get().as_ref()
                            .map(|p| p.display_name_or_email()).unwrap_or_default();
                        let initial = name.chars().next().unwrap_or('?')
                            .to_uppercase().to_string();
                        view! { <span class="avatar-initial">{initial}</span> }
                    }}
                </div>
                <div class="sidebar-user-info">
                    <span class="sidebar-user-name">
                        {move || profile.get().as_ref()
                            .map(|p| p.display_name_or_email())
                            .unwrap_or_else(|| "Loading...".to_string())}
                    </span>
                    {move || if profile.get().as_ref().map(|p| p.is_admin()).unwrap_or(false) {
                        view! {
                            <span class="sidebar-role-badge sidebar-role-badge--admin">
                                <IconShield class="icon-svg icon-xs" />"Admin"
                            </span>
                        }.into_any()
                    } else {
                        view! { <span class="sidebar-role-badge">"User"</span> }.into_any()
                    }}
                </div>
            </div>

            // ── Navigation ─────────────────────────────────────
            <nav class="sidebar-nav">
                // ── Account ─────────────────────────────────────
                <div class="sidebar-nav-section">
                    <SidebarItem
                        icon_slot=view! { <IconHome  class="icon-svg icon-sm" /> }.into_any()
                        label="Overview"
                        active=Signal::derive(move || active_view.get() == DashView::Overview)
                        on_click=nav_overview
                    />
                    <SidebarItem
                        icon_slot=view! { <IconUser  class="icon-svg icon-sm" /> }.into_any()
                        label="Profile"
                        active=Signal::derive(move || active_view.get() == DashView::Profile)
                        on_click=nav_profile
                    />
                    <SidebarItem
                        icon_slot=view! { <IconKey   class="icon-svg icon-sm" /> }.into_any()
                        label="Credentials"
                        active=Signal::derive(move || active_view.get() == DashView::Credentials)
                        on_click=nav_creds
                    />
                </div>

                <div class="sidebar-nav-divider"></div>

                // ── Platform ─────────────────────────────────────
                <div class="sidebar-nav-section">
                    <div class="sidebar-nav-label">"Platform"</div>
                    <SidebarItem
                        icon_slot=view! { <IconLayers  class="icon-svg icon-sm" /> }.into_any()
                        label="Services"
                        active=Signal::derive(move || active_view.get() == DashView::Services)
                        on_click=nav_services
                    />
                    <SidebarItem
                        icon_slot=view! { <IconPackage class="icon-svg icon-sm" /> }.into_any()
                        label="DixScript Registry"
                        active=Signal::derive(move || active_view.get() == DashView::Registry)
                        on_click=nav_registry
                    />
                </div>

                // ── Admin — CSS-hidden for non-admins ─────────────
                // Always rendered so closures are not inside reactive FnMut.
                <div class=move || {
                    if profile.get().as_ref().map(|p| p.is_admin()).unwrap_or(false) {
                        "sidebar-admin-section"
                    } else {
                        "sidebar-admin-section sidebar-admin-section--hidden"
                    }
                }>
                    <div class="sidebar-nav-divider"></div>
                    <div class="sidebar-nav-section">
                        <div class="sidebar-nav-label">"Administration"</div>
                        <SidebarItem
                            icon_slot=view! { <IconUsers   class="icon-svg icon-sm" /> }.into_any()
                            label="Users"
                            active=Signal::derive(move || active_view.get() == DashView::AdminUsers)
                            on_click=nav_admin_users
                        />
                        <SidebarItem
                            icon_slot=view! { <IconPackage class="icon-svg icon-sm" /> }.into_any()
                            label="Registry Review"
                            active=Signal::derive(move || active_view.get() == DashView::AdminRegistry)
                            on_click=nav_admin_registry
                        />
                        // R2 Files and Permissions items added in response 3
                    </div>
                </div>
            </nav>

            // ── Footer ─────────────────────────────────────────
            <div class="sidebar-footer">
                <SidebarItem
                    icon_slot=view! { <IconSettings class="icon-svg icon-sm" /> }.into_any()
                    label="Settings"
                    active=Signal::derive(move || active_view.get() == DashView::Settings)
                    on_click=nav_settings
                />
                <button class="sidebar-item sidebar-item--signout" on:click=handle_signout>
                    <span class="sidebar-item-icon"><IconLogOut class="icon-svg icon-sm" /></span>
                    <span class="sidebar-item-label">"Sign Out"</span>
                </button>
            </div>
        </aside>
        </>
    }
}

#[component]
fn SidebarItem(
    icon_slot: AnyView,
    label:     &'static str,
    active:    Signal<bool>,
    on_click:  impl Fn(web_sys::MouseEvent) + 'static,
) -> impl IntoView {
    view! {
        <button
            class=move || {
                if active.get() { "sidebar-item sidebar-item--active" } else { "sidebar-item" }
            }
            on:click=on_click
        >
            <span class="sidebar-item-icon">{icon_slot}</span>
            <span class="sidebar-item-label">{label}</span>
        </button>
    }
}
