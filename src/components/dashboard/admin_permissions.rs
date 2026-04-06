// src/components/dashboard/admin_permissions.rs
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;
use std::sync::Arc;
use gloo_storage::Storage;
use crate::supabase::{SupabaseClient, Profile};
use crate::components::icons::{
    IconDatabase, IconShield, IconUser, IconCheck, IconX, IconLoader, IconAlertTriangle,
};

#[component]
pub fn AdminPermissionsView(profile: ReadSignal<Option<Profile>>) -> impl IntoView {
    let is_admin = move || profile.get().as_ref().map(|p| p.is_admin()).unwrap_or(false);

    let (profiles,       set_profiles)       = signal(Vec::<Profile>::new());
    let (loading,        set_loading)        = signal(true);
    let (error,          set_error)          = signal(String::new());
    let (search,         set_search)         = signal(String::new());
    let (action_error,   set_action_error)   = signal(String::new());
    let (action_success, set_action_success) = signal(String::new());
    let (confirm_id,     set_confirm_id)     = signal(None::<String>);
    let (confirm_action, set_confirm_action) = signal(String::new());
    let (processing_id,  set_processing_id)  = signal(None::<String>);

    let my_id = Arc::new(
        gloo_storage::LocalStorage::get::<String>("mms_user_id").unwrap_or_default()
    );

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

    let handle_role_change: Arc<dyn Fn(String, String) + Send + Sync + 'static> = Arc::new(
        move |target_id: String, new_role: String| {
            if processing_id.get().is_some() { return; }
            set_processing_id.set(Some(target_id.clone()));
            set_action_error.set(String::new());
            set_action_success.set(String::new());
            set_confirm_id.set(None);
            set_confirm_action.set(String::new());
            let client    = SupabaseClient::new();
            let new_role2 = new_role.clone();
            spawn_local(async move {
                match client.update_user_role(&target_id, &new_role).await {
                    Ok(()) => {
                        set_profiles.update(|ps| {
                            if let Some(p) = ps.iter_mut().find(|p| p.id == target_id) {
                                p.role = new_role2.clone();
                            }
                        });
                        set_processing_id.set(None);
                        let verb = if new_role2 == "admin" { "promoted to admin" } else { "demoted to user" };
                        set_action_success.set(format!("User {}.", verb));
                    }
                    Err(e) => {
                        set_action_error.set(e);
                        set_processing_id.set(None);
                    }
                }
            });
        }
    );

    view! {
        <div class="admin-permissions-view">
            {
                let hrc_outer   = handle_role_change.clone();
                let my_id_outer = my_id.clone();

                move || if !is_admin() {
                    view! {
                        <div class="admin-gate">
                            <IconShield class="icon-svg admin-gate-icon" />
                            <p class="admin-gate-text">"Admin access required."</p>
                        </div>
                    }.into_any()
                } else {
                    let hrc_for_table = hrc_outer.clone();
                    let my_id_table   = my_id_outer.clone();

                    view! {
                        <div class="admin-permissions-content">

                            <div class="admin-section-header">
                                <div class="admin-section-header-icon">
                                    <IconDatabase class="icon-svg" />
                                </div>
                                <div>
                                    <h1 class="admin-section-title">"Permissions"</h1>
                                    <p class="admin-section-subtitle">
                                        "Manage account roles. Promote users to admin or demote admins "
                                        "back to user. You cannot change your own role."
                                    </p>
                                </div>
                            </div>

                            {move || {
                                let total  = profiles.get().len();
                                let admins = profiles.get().iter().filter(|p| p.is_admin()).count();
                                let users  = total - admins;
                                view! {
                                    <div class="admin-stats-row">
                                        <AdminPermStatCard label="Total Accounts" value=total.to_string()  />
                                        <AdminPermStatCard label="Admins"         value=admins.to_string() />
                                        <AdminPermStatCard label="Standard Users" value=users.to_string()  />
                                    </div>
                                }
                            }}

                            {move || if !action_error.get().is_empty() {
                                view! {
                                    <div class="status-msg status-msg--error">
                                        <IconAlertTriangle class="icon-svg icon-sm" />
                                        {action_error.get()}
                                    </div>
                                }.into_any()
                            } else { view! { <span></span> }.into_any() }}

                            {move || if !action_success.get().is_empty() {
                                view! {
                                    <div class="status-msg status-msg--success">
                                        <IconCheck class="icon-svg icon-sm" />
                                        {action_success.get()}
                                    </div>
                                }.into_any()
                            } else { view! { <span></span> }.into_any() }}

                            {move || if !error.get().is_empty() {
                                view! { <div class="status-msg status-msg--error">{error.get()}</div> }.into_any()
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

                                {
                                    move || if loading.get() {
                                        view! {
                                            <div class="admin-table-loading"><div class="spinner"></div></div>
                                        }.into_any()
                                    } else {
                                        let q    = search.get().to_lowercase();
                                        let rows = profiles.get().into_iter()
                                            .filter(|p| {
                                                q.is_empty()
                                                || p.email.to_lowercase().contains(&q)
                                                || p.display_name.as_deref().unwrap_or("").to_lowercase().contains(&q)
                                                || p.mid_id.to_lowercase().contains(&q)
                                            })
                                            .collect::<Vec<_>>();

                                        let hrc       = hrc_for_table.clone();
                                        let my_id_tbl = my_id_table.clone();

                                        view! {
                                            <div class="admin-table-scroll">
                                                <table class="admin-table">
                                                    <thead>
                                                        <tr>
                                                            <th>"User"</th>
                                                            <th>"Email"</th>
                                                            <th>"Current Role"</th>
                                                            <th>"Joined"</th>
                                                            <th>"Actions"</th>
                                                        </tr>
                                                    </thead>
                                                    <tbody>
                                                        {rows.into_iter().map(|p| {
                                                            let hrc       = hrc.clone();
                                                            let my_id_row = my_id_tbl.clone();

                                                            let pid            = Arc::new(p.id.clone());
                                                            let pid_promote    = pid.clone();
                                                            let pid_demote     = pid.clone();
                                                            let pid_cfm        = pid.clone();
                                                            let pid_cancel     = pid.clone();
                                                            let pid_cfm_chk    = pid.clone();
                                                            let pid_proc_chk   = pid.clone();

                                                            let name    = p.display_name_or_email();
                                                            let initial = name.chars().next().unwrap_or('?').to_uppercase().to_string();
                                                            let joined  = p.created_at.as_deref().and_then(|d| d.get(..10)).unwrap_or("—").to_string();
                                                            let is_admin_user = p.is_admin();
                                                            let is_self       = p.id == *my_id_row;

                                                            view! {
                                                                <tr class="admin-table-row">
                                                                    <td class="admin-cell admin-cell--name">
                                                                        <span class="avatar-initial avatar-initial--sm">{initial}</span>
                                                                        <span>{name}</span>
                                                                        {if is_self {
                                                                            view! { <span class="perm-self-badge">"(you)"</span> }.into_any()
                                                                        } else { view! { <span></span> }.into_any() }}
                                                                    </td>
                                                                    <td class="admin-cell admin-cell--email">{p.email.clone()}</td>
                                                                    <td class="admin-cell">
                                                                        {if is_admin_user {
                                                                            view! {
                                                                                <span class="badge badge--admin">
                                                                                    <IconShield class="icon-svg icon-xs" />"Admin"
                                                                                </span>
                                                                            }.into_any()
                                                                        } else {
                                                                            view! {
                                                                                <span class="badge">
                                                                                    <IconUser class="icon-svg icon-xs" />"User"
                                                                                </span>
                                                                            }.into_any()
                                                                        }}
                                                                    </td>
                                                                    <td class="admin-cell admin-cell--date">{joined}</td>
                                                                    <td class="admin-cell">
                                                                        {
                                                                            move || {
                                                                                let is_confirming = confirm_id.get().as_deref() == Some(pid_cfm_chk.as_str());
                                                                                let is_processing = processing_id.get().as_deref() == Some(pid_proc_chk.as_str());

                                                                                if is_self {
                                                                                    view! { <span class="perm-self-note">"Cannot change own role"</span> }.into_any()
                                                                                } else if is_processing {
                                                                                    view! {
                                                                                        <div class="perm-processing">
                                                                                            <IconLoader class="icon-svg spin" />
                                                                                            <span>"Updating..."</span>
                                                                                        </div>
                                                                                    }.into_any()
                                                                                } else if is_confirming {
                                                                                    let action = confirm_action.get();
                                                                                    let action_label = if action == "promote" { "Promote to Admin?" } else { "Demote to User?" };
                                                                                    let new_role = if action == "promote" { "admin".to_string() } else { "user".to_string() };
                                                                                    let hrc2    = hrc.clone();
                                                                                    let k_cfm   = pid_cfm.clone();
                                                                                    let k_cancel = pid_cancel.clone();
                                                                                    view! {
                                                                                        <div class="perm-confirm-row">
                                                                                            <span class="perm-confirm-label">{action_label}</span>
                                                                                            <button
                                                                                                class="btn btn-primary btn-sm"
                                                                                                on:click=move |_| hrc2((*k_cfm).clone(), new_role.clone())
                                                                                            >
                                                                                                <IconCheck class="icon-svg icon-xs" />
                                                                                                "Confirm"
                                                                                            </button>
                                                                                            <button
                                                                                                class="btn btn-ghost btn-sm"
                                                                                                on:click=move |_| {
                                                                                                    if confirm_id.get().as_deref() == Some(k_cancel.as_str()) {
                                                                                                        set_confirm_id.set(None);
                                                                                                        set_confirm_action.set(String::new());
                                                                                                    }
                                                                                                }
                                                                                            >
                                                                                                <IconX class="icon-svg icon-xs" />
                                                                                                "Cancel"
                                                                                            </button>
                                                                                        </div>
                                                                                    }.into_any()
                                                                                } else if is_admin_user {
                                                                                    let k = pid_demote.clone();
                                                                                    view! {
                                                                                        <button
                                                                                            class="btn btn-ghost btn-sm perm-btn--demote"
                                                                                            disabled=move || processing_id.get().is_some()
                                                                                            on:click=move |_| {
                                                                                                set_confirm_id.set(Some((*k).clone()));
                                                                                                set_confirm_action.set("demote".to_string());
                                                                                            }
                                                                                        >
                                                                                            "Demote to User"
                                                                                        </button>
                                                                                    }.into_any()
                                                                                } else {
                                                                                    let k = pid_promote.clone();
                                                                                    view! {
                                                                                        <button
                                                                                            class="btn btn-primary btn-sm perm-btn--promote"
                                                                                            disabled=move || processing_id.get().is_some()
                                                                                            on:click=move |_| {
                                                                                                set_confirm_id.set(Some((*k).clone()));
                                                                                                set_confirm_action.set("promote".to_string());
                                                                                            }
                                                                                        >
                                                                                            <IconShield class="icon-svg icon-xs" />
                                                                                            "Promote to Admin"
                                                                                        </button>
                                                                                    }.into_any()
                                                                                }
                                                                            }
                                                                        }
                                                                    </td>
                                                                </tr>
                                                            }.into_any()
                                                        }).collect_view()}
                                                    </tbody>
                                                </table>
                                            </div>
                                        }.into_any()
                                    }
                                }
                            </div>
                        </div>
                    }.into_any()
                }
            }
        </div>
    }
}

#[component]
fn AdminPermStatCard(label: &'static str, value: String) -> impl IntoView {
    view! {
        <div class="admin-stat-card">
            <div class="admin-stat-value">{value}</div>
            <div class="admin-stat-label">{label}</div>
        </div>
    }
}
