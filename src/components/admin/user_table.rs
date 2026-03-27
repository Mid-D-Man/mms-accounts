use leptos::prelude::*;
use crate::supabase::Profile;
use crate::components::icons::{IconShield, IconUser};

#[component]
pub fn UserTable(profiles: ReadSignal<Vec<Profile>>) -> impl IntoView {
    let (search, set_search) = signal(String::new());

    view! {
        <div class="user-table-wrap">
            <div class="user-table-toolbar">
                <h2 class="user-table-title">"All Accounts"</h2>
                <input
                    class="form-input user-table-search"
                    type="search"
                    placeholder="Search by name or email..."
                    prop:value=move || search.get()
                    on:input=move |ev| set_search.set(event_target_value(&ev))
                />
            </div>

            <div class="user-table-scroll">
                <table class="user-table">
                    <thead>
                        <tr>
                            <th>"User"</th>
                            <th>"Email"</th>
                            <th>"Role"</th>
                            <th>"Joined"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {move || {
                            let q = search.get().to_lowercase();
                            profiles.get()
                                .into_iter()
                                .filter(|p| {
                                    q.is_empty()
                                    || p.email.to_lowercase().contains(&q)
                                    || p.display_name.as_deref()
                                        .unwrap_or("")
                                        .to_lowercase()
                                        .contains(&q)
                                })
                                .map(|p| {
                                    let name = p.display_name_or_email();
                                    let initial = name.chars().next()
                                        .unwrap_or('?')
                                        .to_uppercase()
                                        .to_string();
                                    let joined = p.created_at.as_deref()
                                        .map(|d| &d[..10])
                                        .unwrap_or("—")
                                        .to_string();
                                    let is_admin = p.is_admin();

                                    view! {
                                        <tr class="user-table-row">
                                            <td class="user-table-cell user-cell-name">
                                                <div class="user-cell-avatar">
                                                    <span class="avatar-initial">{initial}</span>
                                                </div>
                                                <span>{name}</span>
                                            </td>
                                            <td class="user-table-cell">{p.email.clone()}</td>
                                            <td class="user-table-cell">
                                                {if is_admin {
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
                                            <td class="user-table-cell user-cell-date">{joined}</td>
                                        </tr>
                                    }
                                })
                                .collect_view()
                        }}
                    </tbody>
                </table>
            </div>
        </div>
    }
                                     }
