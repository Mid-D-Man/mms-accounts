use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;
use gloo_storage::Storage;
use crate::supabase::{SupabaseClient, Profile, Service, ServiceSubscription};
use crate::components::icons::{
    IconLayers, IconPackage, IconActivity, IconTrophy, IconCloud, IconLoader, IconCheck,
};

#[component]
pub fn ServicesView(profile: ReadSignal<Option<Profile>>) -> impl IntoView {
    let (services,      set_services)      = signal(Vec::<Service>::new());
    let (subscriptions, set_subscriptions) = signal(Vec::<ServiceSubscription>::new());
    let (loading,       set_loading)       = signal(true);
    let (error,         set_error)         = signal(String::new());
    let (toggling,      set_toggling)      = signal(Option::<String>::None);

    // Load on mount
    Effect::new(move |_| {
        let user_id = gloo_storage::LocalStorage::get::<String>("mms_user_id")
            .unwrap_or_default();
        let client = SupabaseClient::new();

        spawn_local(async move {
            let services_res = client.list_services().await;
            let subs_res     = client.get_user_subscriptions(&user_id).await;

            match (services_res, subs_res) {
                (Ok(svcs), Ok(subs)) => {
                    set_services.set(svcs);
                    set_subscriptions.set(subs);
                    set_loading.set(false);
                }
                (Err(e), _) | (_, Err(e)) => {
                    set_error.set(e);
                    set_loading.set(false);
                }
            }
        });
    });

    let handle_toggle = move |service: Service| {
        let user_id = gloo_storage::LocalStorage::get::<String>("mms_user_id")
            .unwrap_or_default();

        // Find existing subscription
        let existing = subscriptions.get()
            .into_iter()
            .find(|s| s.service_id == service.id);

        set_toggling.set(Some(service.id.clone()));

        let client = SupabaseClient::new();

        spawn_local(async move {
            match existing {
                Some(sub) => {
                    // Toggle active/inactive
                    let new_status = if sub.is_active() { "inactive" } else { "active" };
                    match client.update_subscription_status(&sub.id, &user_id, new_status).await {
                        Ok(()) => {
                            set_subscriptions.update(|subs| {
                                if let Some(s) = subs.iter_mut().find(|s| s.id == sub.id) {
                                    s.status = new_status.to_string();
                                }
                            });
                        }
                        Err(e) => set_error.set(e),
                    }
                }
                None => {
                    // New subscription
                    match client.subscribe_to_service(&user_id, &service.id).await {
                        Ok(sub) => set_subscriptions.update(|subs| subs.push(sub)),
                        Err(e)  => set_error.set(e),
                    }
                }
            }
            set_toggling.set(None);
        });
    };

    view! {
        <div class="services-view">
            <div class="services-header">
                <div class="services-header-icon">
                    <IconLayers class="icon-svg" />
                </div>
                <div>
                    <h1 class="services-title">"Platform Services"</h1>
                    <p class="services-subtitle">
                        "Enable MidManStudio services for your account. Each service "
                        "integrates with your MID ID and Credentials."
                    </p>
                </div>
            </div>

            {move || if !error.get().is_empty() {
                view! {
                    <div class="status-msg status-msg--error">{error.get()}</div>
                }.into_any()
            } else { view! { <span></span> }.into_any() }}

            {move || if loading.get() {
                view! {
                    <div class="services-loading">
                        <div class="spinner"></div>
                    </div>
                }.into_any()
            } else {
                let services_snap = services.get();
                let subs_snap     = subscriptions.get();

                view! {
                    <div class="services-grid">
                        {services_snap.into_iter().map(|svc| {
                            let sub = subs_snap.iter()
                                .find(|s| s.service_id == svc.id)
                                .cloned();
                            let is_subscribed = sub.as_ref().map(|s| s.is_active()).unwrap_or(false);
                            let svc_for_toggle = svc.clone();
                            let svc_id         = svc.id.clone();
                            let is_loading     = Signal::derive(move || {
                                toggling.get().as_deref() == Some(&svc_id)
                            });

                            view! {
                                <ServiceCard
                                    service=svc
                                    is_subscribed=is_subscribed
                                    is_loading=is_loading
                                    on_toggle=move || handle_toggle(svc_for_toggle.clone())
                                />
                            }
                        }).collect_view()}
                    </div>
                }.into_any()
            }}
        </div>
    }
}

// ── Service Card ───────────────────────────────────────────────

#[component]
fn ServiceCard(
    service:       Service,
    is_subscribed: bool,
    is_loading:    Signal<bool>,
    on_toggle:     impl Fn() + 'static,
) -> impl IntoView {
    let icon = service_icon(&service.slug);
    let name = service.name.clone();
    let desc = service.description.clone().unwrap_or_default();
    let is_active_service = service.is_active;
    let is_free  = service.is_free;
    let slug     = service.slug.clone();

    view! {
        <div class=move || {
            let mut c = "service-card".to_string();
            if is_subscribed { c.push_str(" service-card--enabled"); }
            if !is_active_service { c.push_str(" service-card--coming-soon"); }
            c
        }>
            // Coming soon overlay
            {if !is_active_service {
                view! {
                    <div class="service-coming-soon">
                        <span class="service-coming-soon-label">"Coming Soon"</span>
                    </div>
                }.into_any()
            } else {
                view! { <span></span> }.into_any()
            }}

            <div class="service-card-top">
                <div class=move || {
                    let mut c = "service-icon-wrap".to_string();
                    if is_subscribed { c.push_str(" service-icon-wrap--active"); }
                    c
                }>
                    {icon}
                </div>
                <div class="service-card-badges">
                    {if is_free {
                        view! { <span class="service-badge service-badge--free">"Free"</span> }.into_any()
                    } else {
                        view! { <span class="service-badge service-badge--pro">"Pro"</span> }.into_any()
                    }}
                    {if is_subscribed {
                        view! { <span class="service-badge service-badge--active">"Active"</span> }.into_any()
                    } else {
                        view! { <span></span> }.into_any()
                    }}
                </div>
            </div>

            <h3 class="service-card-title">{name}</h3>
            <p class="service-card-desc">{desc}</p>

            // Slug label
            <div class="service-slug">
                <code>{slug}</code>
            </div>

            // Toggle button
            {if is_active_service {
                view! {
                    <button
                        class=move || {
                            if is_subscribed {
                                "btn btn-ghost btn-sm service-toggle-btn"
                            } else {
                                "btn btn-primary btn-sm service-toggle-btn"
                            }
                        }
                        disabled=move || is_loading.get()
                        on:click=move |_| on_toggle()
                    >
                        {move || if is_loading.get() {
                            view! {
                                <IconLoader class="icon-svg spin" />
                                <span>"Updating..."</span>
                            }.into_any()
                        } else if is_subscribed {
                            view! {
                                <span>"Disable"</span>
                            }.into_any()
                        } else {
                            view! {
                                <IconCheck class="icon-svg icon-xs" />
                                <span>"Enable"</span>
                            }.into_any()
                        }}
                    </button>
                }.into_any()
            } else {
                view! { <span></span> }.into_any()
            }}
        </div>
    }
}

/// Maps a service slug to an icon AnyView.
fn service_icon(slug: &str) -> AnyView {
    match slug {
        "dixscript-registry" => view! { <IconPackage  class="icon-svg" /> }.into_any(),
        "game-analytics"     => view! { <IconActivity class="icon-svg" /> }.into_any(),
        "leaderboards"       => view! { <IconTrophy   class="icon-svg" /> }.into_any(),
        "cloud-saves"        => view! { <IconCloud    class="icon-svg" /> }.into_any(),
        _                    => view! { <IconLayers   class="icon-svg" /> }.into_any(),
    }
                    }
