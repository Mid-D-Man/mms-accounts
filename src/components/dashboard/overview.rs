use leptos::prelude::*;
use crate::supabase::Profile;
use crate::components::icons::{IconUser, IconShield, IconGlobe, IconEdit};

#[component]
pub fn OverviewView(profile: ReadSignal<Option<Profile>>) -> impl IntoView {
    view! {
        <div class="overview-view">

            // Account card
            <div class="overview-card">
                <div class="overview-card-head">
                    <div class="overview-avatar">
                        {move || {
                            let name = profile.get()
                                .as_ref()
                                .map(|p| p.display_name_or_email())
                                .unwrap_or_default();
                            let initial = name.chars().next()
                                .unwrap_or('?')
                                .to_uppercase()
                                .to_string();
                            view! { <span class="avatar-initial avatar-initial--lg">{initial}</span> }
                        }}
                    </div>
                    <div class="overview-identity">
                        <h2 class="overview-name">
                            {move || profile.get()
                                .as_ref()
                                .map(|p| p.display_name_or_email())
                                .unwrap_or_else(|| "—".to_string())}
                        </h2>
                        <p class="overview-email">
                            {move || profile.get()
                                .as_ref()
                                .map(|p| p.email.clone())
                                .unwrap_or_default()}
                        </p>
                        {move || {
                            let is_admin = profile.get()
                                .as_ref()
                                .map(|p| p.is_admin())
                                .unwrap_or(false);
                            if is_admin {
                                view! {
                                    <span class="badge badge--admin">
                                        <IconShield class="icon-svg icon-xs" />
                                        "Admin"
                                    </span>
                                }.into_any()
                            } else {
                                view! {
                                    <span class="badge">"User"</span>
                                }.into_any()
                            }
                        }}
                    </div>
                </div>

                <div class="overview-details">
                    <OverviewDetail
                        label="Display Name"
                        icon_type="user"
                        value=Signal::derive(move || {
                            profile.get()
                                .as_ref()
                                .and_then(|p| p.display_name.clone())
                                .unwrap_or_else(|| "Not set".to_string())
                        })
                    />
                    <OverviewDetail
                        label="Website"
                        icon_type="globe"
                        value=Signal::derive(move || {
                            profile.get()
                                .as_ref()
                                .and_then(|p| p.website.clone())
                                .unwrap_or_else(|| "Not set".to_string())
                        })
                    />
                    <OverviewDetail
                        label="Member Since"
                        icon_type="user"
                        value=Signal::derive(move || {
                            profile.get()
                                .as_ref()
                                .and_then(|p| p.created_at.clone())
                                .map(|d| d[..10].to_string())
                                .unwrap_or_else(|| "—".to_string())
                        })
                    />
                </div>
            </div>

            // Bio card
            {move || {
                let bio = profile.get()
                    .as_ref()
                    .and_then(|p| p.bio.clone())
                    .unwrap_or_default();
                if !bio.is_empty() {
                    view! {
                        <div class="overview-bio-card">
                            <h3 class="overview-bio-label">"Bio"</h3>
                            <p class="overview-bio-text">{bio}</p>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <div class="overview-bio-card overview-bio-card--empty">
                            <IconEdit class="icon-svg" />
                            <p>"No bio yet. Edit your profile to add one."</p>
                        </div>
                    }.into_any()
                }
            }}

        </div>
    }
}

#[component]
fn OverviewDetail(
    label:     &'static str,
    icon_type: &'static str,
    value:     Signal<String>,
) -> impl IntoView {
    let icon = match icon_type {
        "globe" => view! { <IconGlobe class="icon-svg icon-xs" /> }.into_any(),
        _       => view! { <IconUser  class="icon-svg icon-xs" /> }.into_any(),
    };

    view! {
        <div class="overview-detail-row">
            <div class="overview-detail-label">
                {icon}
                <span>{label}</span>
            </div>
            <span class="overview-detail-value">{move || value.get()}</span>
        </div>
    }
                              }
