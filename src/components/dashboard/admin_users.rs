// src/components/dashboard/admin_users.rs
// User management table — admin-only.
// Shows all registered accounts with search, stats, and role badges.
// Role changes are handled in AdminPermissionsView (response 3).

use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;
use crate::supabase::{SupabaseClient, Profile};
use crate::components::icons::{IconShield, IconUser, IconUsers};

#[component]
pub fn AdminUsersView(profile: ReadSignal<Option<Profile>>) -> impl IntoView {
    let is_admin = move || profile.get().as_ref().map(|p| p.is_admin()).unwrap_or(false);

    let (profiles, set_profiles) = signal(Vec::<Profile>::new());
    let (loading,  set_loading)  = signal(true);
    let (error,    set_error)    = signal(String::new());
    let (search,   set_search)   = signal(String::new());

    Effect::new(move |_| {
        if !is_admin() { set_loading.set(false); return; }
        let client = SupabaseClient::new();
        spawn_local(async move {
            match client.get_all_profiles().await {
                Ok(all) => { set_profiles.set(all); set_loading.set(false); }
                Err(e)  => { set_error.set(e);       set_loading.set(false); }
            }
        });
    });

    view! {
        <div class="admin-users-view">
            {move || if !is_admin() {
                view! {
                    <div class="admin-gate">
                        <IconShield class="icon-svg admin-gate-icon" />
                        <p class="admin-gate-text">"Admin access required."</p>
                    </div>
                }.into_any()
            } else {
                view! {
                    <div class="admin-users-content">

                        <div class="admin-section-header">
                            <div class="admin-section-header-icon">
                                <IconUsers class="icon-svg" />
                            </div>
                            <div>
                                <h1 class="admin-section-title">"User Management"</h1>
                                <p class="admin-section-subtitle">
                                    "All registered MidManStudio accounts. "
                                    "To change roles, use the Permissions view."
                                </p>
                            </div>
                        </div>

                        {move || {
                            let total  = profiles.get().len();
                            let admins = profiles.get().iter().filter(|p| p.is_admin()).count();
                            let users  = total - admins;
                            view! {
                                <div class="admin-stats-row">
                                    <AdminStatCard label="Total Accounts" value=total.to_string()  />
                                    <AdminStatCard label="Admins"         value=admins.to_string() />
                                    <AdminStatCard label="Standard Users" value=users.to_string()  />
                                </div>
                            }
                        }}

                        {move || if !error.get().is_empty() {
                            view! {
                                <div class="status-msg status-msg--error">{error.get()}</div>
                            }.into_any()
                        } else { view! { <span></span> }.into_any() }}

                        <div class="admin-table-card">
                            <div class="admin-table-toolbar">
                                <h2 class="admin-table-title">"All Accounts"</h2>
                                <input
                                    class="form-input admin-search"
                                    type="search"
                                    placeholder="Search by name or email..."
                                    prop:value=move || search.get()
                                    on:input=move |ev| set_search.set(event_target_value(&ev))
                                />
                            </div>

                            {move || if loading.get() {
                                view! {
                                    <div class="admin-table-loading">
                                        <div class="spinner"></div>
                                    </div>
                                }.into_any()
                            } else {
                                let q    = search.get().to_lowercase();
                                let rows = profiles.get().into_iter()
                                    .filter(|p| {
                                        q.is_empty()
                                        || p.email.to_lowercase().contains(&q)
                                        || p.display_name.as_deref()
                                            .unwrap_or("").to_lowercase().contains(&q)
                                        || p.mid_id.to_lowercase().contains(&q)
                                    })
                                    .collect::<Vec<_>>();

                                view! {
                                    <div class="admin-table-scroll">
                                        <table class="admin-table">
                                            <thead>
                                                <tr>
                                                    <th>"User"</th>
                                                    <th>"Email"</th>
                                                    <th>"Role"</th>
                                                    <th>"Joined"</th>
                                                </tr>
                                            </thead>
                                            <tbody>
                                                {rows.into_iter().map(|p| {
                                                    let name    = p.display_name_or_email();
                                                    let initial = name.chars().next()
                                                        .unwrap_or('?').to_uppercase().to_string();
                                                    let joined  = p.created_at.as_deref()
                                                        .and_then(|d| d.get(..10))
                                                        .unwrap_or("—").to_string();
                                                    let is_admin_user = p.is_admin();
                                                    view! {
                                                        <tr class="admin-table-row">
                                                            <td class="admin-cell admin-cell--name">
                                                                <span class="avatar-initial avatar-initial--sm">
                                                                    {initial}
                                                                </span>
                                                                <span>{name}</span>
                                                            </td>
                                                            <td class="admin-cell admin-cell--email">
                                                                {p.email}
                                                            </td>
                                                            <td class="admin-cell">
                                                                {if is_admin_user {
                                                                    view! {
                                                                        <span class="badge badge--admin">
                                                                            <IconShield class="icon-svg icon-xs" />
                                                                            "Admin"
                                                                        </span>
                                                                    }.into_any()
                                                                } else {
                                                                    view! {
                                                                        <span class="badge">
                                                                            <IconUser class="icon-svg icon-xs" />
                                                                            "User"
                                                                        </span>
                                                                    }.into_any()
                                                                }}
                                                            </td>
                                                            <td class="admin-cell admin-cell--date">
                                                                {joined}
                                                            </td>
                                                        </tr>
                                                    }
                                                }).collect_view()}
                                            </tbody>
                                        </table>
                                    </div>
                                }.into_any()
                            }}
                        </div>
                    </div>
                }.into_any()
            }}
        </div>
    }
}

// ── Shared stat card — re-exported for use in other admin components ──

#[component]
pub fn AdminStatCard(label: &'static str, value: String) -> impl IntoView {
    view! {
        <div class="admin-stat-card">
            <div class="admin-stat-value">{value}</div>
            <div class="admin-stat-label">{label}</div>
        </div>
    }
}
