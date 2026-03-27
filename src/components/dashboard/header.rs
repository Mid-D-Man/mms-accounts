use leptos::prelude::*;
use crate::supabase::Profile;

#[component]
pub fn DashboardHeader(profile: ReadSignal<Option<Profile>>) -> impl IntoView {
    view! {
        <header class="dash-header">
            <div class="dash-header-inner">
                <div class="dash-header-left">
                    <h1 class="dash-header-title">
                        {move || {
                            let name = profile.get()
                                .as_ref()
                                .map(|p| p.display_name_or_email())
                                .unwrap_or_default();
                            if name.is_empty() {
                                "Dashboard".to_string()
                            } else {
                                format!("Welcome back, {}", name)
                            }
                        }}
                    </h1>
                    <p class="dash-header-sub">"MidManStudio Accounts"</p>
                </div>
                <div class="dash-header-right">
                    <div class="dash-header-status">
                        <span class="status-dot status-dot--active"></span>
                        <span class="status-label">"Active"</span>
                    </div>
                </div>
            </div>
        </header>
    }
      }
