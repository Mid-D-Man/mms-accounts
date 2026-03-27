use leptos::prelude::*;
use crate::supabase::Profile;
use crate::components::icons::{IconUsers, IconShield, IconUser};

#[component]
pub fn AdminStats(profiles: ReadSignal<Vec<Profile>>) -> impl IntoView {
    view! {
        <div class="admin-stats-grid">
            <StatCard
                label="Total Users"
                icon_type="users"
                value=Signal::derive(move || profiles.get().len().to_string())
            />
            <StatCard
                label="Admins"
                icon_type="shield"
                value=Signal::derive(move || {
                    profiles.get()
                        .iter()
                        .filter(|p| p.is_admin())
                        .count()
                        .to_string()
                })
            />
            <StatCard
                label="Regular Users"
                icon_type="user"
                value=Signal::derive(move || {
                    profiles.get()
                        .iter()
                        .filter(|p| !p.is_admin())
                        .count()
                        .to_string()
                })
            />
        </div>
    }
}

#[component]
fn StatCard(
    label:     &'static str,
    icon_type: &'static str,
    value:     Signal<String>,
) -> impl IntoView {
    let icon = match icon_type {
        "shield" => view! { <IconShield class="icon-svg" /> }.into_any(),
        "user"   => view! { <IconUser   class="icon-svg" /> }.into_any(),
        _        => view! { <IconUsers  class="icon-svg" /> }.into_any(),
    };

    view! {
        <div class="admin-stat-card">
            <div class="admin-stat-icon">{icon}</div>
            <div class="admin-stat-body">
                <span class="admin-stat-value">{move || value.get()}</span>
                <span class="admin-stat-label">{label}</span>
            </div>
        </div>
    }
  }
