use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;
use gloo_storage::Storage;
use crate::supabase::{SupabaseClient, Service, ServiceSubscription};
use crate::components::icons::{
    IconLayers, IconPackage, IconActivity, IconTrophy, IconCloud,
    IconLoader, IconCheck, IconX,
};

#[component]
pub fn ServicesView(_profile: ReadSignal<Option<Profile>>) -> impl IntoView {
    let (services,      set_services)      = signal(Vec::<Service>::new());
    let (subscriptions, set_subscriptions) = signal(Vec::<ServiceSubscription>::new());
    let (loading,       set_loading)       = signal(true);
    let (error,         set_error)         = signal(String::new());
    let (toggling,      set_toggling)      = signal(Option::<String>::None);
    let (modal_service, set_modal_service) = signal(Option::<Service>::None);

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

    let handle_toggle = move |service_id: String| {
        let user_id  = gloo_storage::LocalStorage::get::<String>("mms_user_id")
            .unwrap_or_default();
        let svc_id   = service_id.clone();
        let existing = subscriptions.get().into_iter().find(|s| s.service_id == svc_id);

        set_toggling.set(Some(service_id.clone()));
        let client = SupabaseClient::new();

        spawn_local(async move {
            match existing {
                Some(sub) => {
                    let new_status = if sub.is_active() { "inactive" } else { "active" };
                    match client.update_subscription_status(&sub.id, &user_id, new_status).await {
                        Ok(()) => set_subscriptions.update(|subs| {
                            if let Some(s) = subs.iter_mut().find(|s| s.id == sub.id) {
                                s.status = new_status.to_string();
                            }
                        }),
                        Err(e) => set_error.set(e),
                    }
                }
                None => {
                    match client.subscribe_to_service(&user_id, &service_id).await {
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
            {move || if let Some(svc) = modal_service.get() {
                let subs        = subscriptions.get();
                let is_sub      = subs.iter().any(|s| s.service_id == svc.id && s.is_active());
                let svc_id_tog  = svc.id.clone();
                let svc_id_load = svc.id.clone();
                let is_loading  = Signal::derive(move || {
                    toggling.get().as_deref() == Some(&svc_id_load)
                });

                view! {
                    <ServiceModal
                        service=svc.clone()
                        is_subscribed=is_sub
                        is_loading=is_loading
                        on_close=move || set_modal_service.set(None)
                        on_toggle=move || handle_toggle(svc_id_tog.clone())
                    />
                }.into_any()
            } else { view! { <span></span> }.into_any() }}

            <div class="services-header">
                <div class="services-header-icon"><IconLayers class="icon-svg" /></div>
                <div>
                    <h1 class="services-title">"Platform Services"</h1>
                    <p class="services-subtitle">
                        "Enable MidManStudio services for your account. "
                        "Click a card to learn more before enabling."
                    </p>
                </div>
            </div>

            {move || if !error.get().is_empty() {
                view! { <div class="status-msg status-msg--error">{error.get()}</div> }.into_any()
            } else { view! { <span></span> }.into_any() }}

            {move || if loading.get() {
                view! { <div class="services-loading"><div class="spinner"></div></div> }.into_any()
            } else {
                let services_snap = services.get();
                let subs_snap     = subscriptions.get();
                view! {
                    <div class="services-grid">
                        {services_snap.into_iter().map(|svc| {
                            let sub          = subs_snap.iter().find(|s| s.service_id == svc.id).cloned();
                            let is_subscribed = sub.as_ref().map(|s| s.is_active()).unwrap_or(false);
                            let svc_modal    = svc.clone();
                            let svc_id_tog   = svc.id.clone();
                            let svc_id_load  = svc.id.clone();
                            let is_loading   = Signal::derive(move || {
                                toggling.get().as_deref() == Some(&svc_id_load)
                            });

                            view! {
                                <ServiceCard
                                    service=svc
                                    is_subscribed=is_subscribed
                                    is_loading=is_loading
                                    on_click=move || set_modal_service.set(Some(svc_modal.clone()))
                                    on_toggle=move || handle_toggle(svc_id_tog.clone())
                                />
                            }
                        }).collect_view()}
                    </div>
                }.into_any()
            }}
        </div>
    }
}

#[component]
fn ServiceCard(
    service:       Service,
    is_subscribed: bool,
    is_loading:    Signal<bool>,
    on_click:      impl Fn() + 'static,
    on_toggle:     impl Fn() + 'static,
) -> impl IntoView {
    let icon          = service_icon(&service.slug);
    let name          = service.name.clone();
    let full_desc     = service.description.clone().unwrap_or_default();
    let short_desc    = if full_desc.len() > 90 {
        format!("{}...", &full_desc[..90])
    } else {
        full_desc.clone()
    };
    let is_active_svc = service.is_active;
    let is_free       = service.is_free;

    view! {
        <div
            class=move || {
                let mut c = "service-card".to_string();
                if is_subscribed  { c.push_str(" service-card--enabled"); }
                if !is_active_svc { c.push_str(" service-card--coming-soon"); }
                c
            }
            on:click=move |_| on_click()
            role="button"
            tabindex="0"
        >
            {if !is_active_svc {
                view! {
                    <div class="service-coming-soon">
                        <span class="service-coming-soon-label">"Coming Soon"</span>
                    </div>
                }.into_any()
            } else { view! { <span></span> }.into_any() }}

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
                    } else { view! { <span></span> }.into_any() }}
                </div>
            </div>

            <h3 class="service-card-title">{name}</h3>
            <p class="service-card-desc">{short_desc}</p>
            <div class="service-slug"><code>{service.slug.clone()}</code></div>

            {if is_active_svc {
                view! {
                    <button
                        class=move || {
                            if is_subscribed { "btn btn-ghost btn-sm service-toggle-btn" }
                            else             { "btn btn-primary btn-sm service-toggle-btn" }
                        }
                        disabled=move || is_loading.get()
                        on:click=move |ev| {
                            use wasm_bindgen::JsCast;
                            if let Some(me) = ev.dyn_ref::<web_sys::MouseEvent>() {
                                me.stop_propagation();
                            }
                            on_toggle();
                        }
                    >
                        {move || if is_loading.get() {
                            view! { <IconLoader class="icon-svg spin" /><span>"Updating..."</span> }.into_any()
                        } else if is_subscribed {
                            view! { <span>"Disable"</span> }.into_any()
                        } else {
                            view! { <IconCheck class="icon-svg icon-xs" /><span>"Enable"</span> }.into_any()
                        }}
                    </button>
                }.into_any()
            } else { view! { <span></span> }.into_any() }}
        </div>
    }
}

#[component]
fn ServiceModal(
    service:       Service,
    is_subscribed: bool,
    is_loading:    Signal<bool>,
    on_close:      impl Fn() + 'static + Clone,
    on_toggle:     impl Fn() + 'static,
) -> impl IntoView {
    let icon     = service_icon(&service.slug);
    let name     = service.name.clone();
    let slug     = service.slug.clone();
    let is_free  = service.is_free;
    let is_active = service.is_active;

    // Owned String — no lifetime issues
    let extended_desc: String = match service.slug.as_str() {
        "dixscript-registry" => "Submit and manage .mdix packages for the DixScript cloud registry. \
            Your packages become publicly importable by any DixScript file using from_cloud. \
            Each submission is reviewed before going live. Requires your MID ID for attribution.".to_string(),
        "game-analytics" => "Track player events, sessions, and custom metrics across all MmS games. \
            Send events via your MID Secret and view aggregated dashboards here. \
            Supports custom event schemas, funnels, and retention analysis.".to_string(),
        "leaderboards" => "Per-game configurable leaderboards with public read access via MID ID. \
            Create boards with custom scoring, time windows (daily/weekly/all-time), \
            and player limit controls. Read access requires no authentication.".to_string(),
        "cloud-saves" => "Key-value save data storage scoped per user and game. \
            Each game authenticates with its MID Secret, users authenticate with their MID ID. \
            Supports versioned saves, conflict resolution, and up to 1MB per save slot.".to_string(),
        _ => service.description.clone().unwrap_or_default(),
    };

    let on_close_bg  = on_close.clone();
    let on_close_btn = on_close.clone();
    let on_close_ftr = on_close.clone();

    view! {
        <div class="service-modal-backdrop" on:click=move |_| on_close_bg()></div>
        <div class="service-modal-panel" role="dialog" aria-modal="true">

            <div class="service-modal-header">
                <div class="service-modal-icon-wrap">{icon}</div>
                <div class="service-modal-title-block">
                    <div class="service-modal-badges">
                        {if is_free {
                            view! { <span class="service-badge service-badge--free">"Free"</span> }.into_any()
                        } else {
                            view! { <span class="service-badge service-badge--pro">"Pro"</span> }.into_any()
                        }}
                        {if !is_active {
                            view! { <span class="service-badge">"Coming Soon"</span> }.into_any()
                        } else if is_subscribed {
                            view! { <span class="service-badge service-badge--active">"Active"</span> }.into_any()
                        } else { view! { <span></span> }.into_any() }}
                    </div>
                    <h2 class="service-modal-title">{name}</h2>
                    <code class="service-modal-slug">{slug}</code>
                </div>
                <button class="service-modal-close"
                        on:click=move |_| on_close_btn()
                        aria-label="Close">
                    <IconX class="icon-svg icon-sm" />
                </button>
            </div>

            <div class="service-modal-body">
                <div class="service-modal-section">
                    <h3 class="service-modal-section-title">"What is this?"</h3>
                    <p class="service-modal-desc">{extended_desc}</p>
                </div>

                <div class="service-modal-section">
                    <h3 class="service-modal-section-title">"Pricing"</h3>
                    <div class="service-pricing-card">
                        {if is_free {
                            view! {
                                <div class="service-price-amount">"Free"</div>
                                <div class="service-price-desc">
                                    "No cost, no limits within fair use. "
                                    "Available to all MmS account holders."
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <div class="service-price-amount">"Pro"</div>
                                <div class="service-price-desc">
                                    "Requires a Pro subscription. "
                                    "Contact MidManStudio for access."
                                </div>
                            }.into_any()
                        }}
                    </div>
                </div>

                <div class="service-modal-section">
                    <h3 class="service-modal-section-title">"Authentication"</h3>
                    <p class="service-modal-desc">
                        "Use your MID ID and a MID Secret to authenticate API calls. "
                        "Generate secrets from the Credentials tab."
                    </p>
                    <div class="service-auth-example">
                        <code>"X-MMS-MID-ID: your_mid_id"</code>
                        <code>"X-MMS-SECRET: mids_your_secret"</code>
                    </div>
                </div>
            </div>

            <div class="service-modal-footer">
                <button class="btn btn-ghost btn-sm"
                        on:click=move |_| on_close_ftr()>"Close"</button>
                {if is_active {
                    view! {
                        <button
                            class=move || {
                                if is_subscribed { "btn btn-ghost btn-sm" }
                                else             { "btn btn-primary btn-sm" }
                            }
                            disabled=move || is_loading.get()
                            on:click=move |_| on_toggle()
                        >
                            {move || if is_loading.get() {
                                view! { <IconLoader class="icon-svg spin" /><span>"Updating..."</span> }.into_any()
                            } else if is_subscribed {
                                view! { <span>"Disable Service"</span> }.into_any()
                            } else {
                                view! { <IconCheck class="icon-svg icon-xs" /><span>"Enable Service"</span> }.into_any()
                            }}
                        </button>
                    }.into_any()
                } else { view! { <span></span> }.into_any() }}
            </div>
        </div>
    }
}

fn service_icon(slug: &str) -> AnyView {
    match slug {
        "dixscript-registry" => view! { <IconPackage  class="icon-svg" /> }.into_any(),
        "game-analytics"     => view! { <IconActivity class="icon-svg" /> }.into_any(),
        "leaderboards"       => view! { <IconTrophy   class="icon-svg" /> }.into_any(),
        "cloud-saves"        => view! { <IconCloud    class="icon-svg" /> }.into_any(),
        _                    => view! { <IconLayers   class="icon-svg" /> }.into_any(),
    }
}
