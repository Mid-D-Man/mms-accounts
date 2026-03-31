// src/components/dashboard/credentials.rs
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;
use gloo_storage::Storage;
use std::sync::Arc;
use crate::supabase::{SupabaseClient, MidSecret, Profile, generate_mid_secret, copy_to_clipboard};
use crate::components::icons::{
    IconKey, IconCopy, IconCheck, IconLoader, IconPlus, IconTrash, IconAlertTriangle,
};

#[component]
pub fn CredentialsView(profile: ReadSignal<Option<Profile>>) -> impl IntoView {
    let (secrets,          set_secrets)          = signal(Vec::<MidSecret>::new());
    let (loading,          set_loading)          = signal(true);
    let (error,            set_error)            = signal(String::new());
    let (show_form,        set_show_form)        = signal(false);
    let (new_label,        set_new_label)        = signal(String::new());
    let (generating,       set_generating)       = signal(false);
    let (gen_error,        set_gen_error)        = signal(String::new());
    let (revealed,         set_revealed)         = signal(None::<String>);
    let (secret_copied,    set_secret_copied)    = signal(false);
    let (mid_id_copied,    set_mid_id_copied)    = signal(false);
    let (revoking_id,      set_revoking_id)      = signal(None::<String>);
    let (confirm_revoke,   set_confirm_revoke)   = signal(None::<String>);

    // ── Load secrets on mount ──────────────────────────────────
    Effect::new(move |_| {
        let user_id = gloo_storage::LocalStorage::get::<String>("mms_user_id")
            .unwrap_or_default();
        if user_id.is_empty() {
            set_loading.set(false);
            return;
        }
        let client = SupabaseClient::new();
        spawn_local(async move {
            match client.list_secrets(&user_id).await {
                Ok(s)  => { set_secrets.set(s); set_loading.set(false); }
                Err(e) => { set_error.set(e);   set_loading.set(false); }
            }
        });
    });

    // ── Copy MID ID ────────────────────────────────────────────
    let handle_copy_mid_id = move |_| {
        let mid_id = profile.get()
            .as_ref()
            .map(|p| p.mid_id.clone())
            .unwrap_or_default();
        spawn_local(async move {
            if copy_to_clipboard(&mid_id).await.is_ok() {
                set_mid_id_copied.set(true);
                let _ = gloo_timers::future::TimeoutFuture::new(2000).await;
                set_mid_id_copied.set(false);
            }
        });
    };

    // ── Copy revealed secret ───────────────────────────────────
    let handle_copy_secret = move |_| {
        let secret = revealed.get().unwrap_or_default();
        spawn_local(async move {
            if copy_to_clipboard(&secret).await.is_ok() {
                set_secret_copied.set(true);
                let _ = gloo_timers::future::TimeoutFuture::new(2000).await;
                set_secret_copied.set(false);
            }
        });
    };

    // ── Generate new secret ────────────────────────────────────
    let handle_generate = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let label   = new_label.get();
        let user_id = gloo_storage::LocalStorage::get::<String>("mms_user_id")
            .unwrap_or_default();
        let mid_id  = profile.get()
            .as_ref()
            .map(|p| p.mid_id.clone())
            .unwrap_or_default();

        if user_id.is_empty() || mid_id.is_empty() {
            set_gen_error.set("Profile not loaded. Try refreshing.".to_string());
            return;
        }

        set_generating.set(true);
        set_gen_error.set(String::new());

        spawn_local(async move {
            match generate_mid_secret().await {
                Err(e) => {
                    set_gen_error.set(format!("Failed to generate secret: {}", e));
                    set_generating.set(false);
                }
                Ok((full_secret, prefix, hash)) => {
                    let label_opt = if label.trim().is_empty() {
                        None
                    } else {
                        Some(label.trim().to_string())
                    };
                    let client = SupabaseClient::new();
                    match client.create_secret(
                        &user_id, &mid_id, label_opt, &hash, &prefix,
                    ).await {
                        Ok(secret) => {
                            set_secrets.update(|s| s.insert(0, secret));
                            set_revealed.set(Some(full_secret));
                            set_show_form.set(false);
                            set_new_label.set(String::new());
                            set_generating.set(false);
                        }
                        Err(e) => {
                            set_gen_error.set(e);
                            set_generating.set(false);
                        }
                    }
                }
            }
        });
    };

    // ── Revoke secret ──────────────────────────────────────────
    let handle_revoke = move |secret_id: String| {
        let user_id = gloo_storage::LocalStorage::get::<String>("mms_user_id")
            .unwrap_or_default();
        set_revoking_id.set(Some(secret_id.clone()));
        set_confirm_revoke.set(None);
        let client = SupabaseClient::new();
        spawn_local(async move {
            match client.revoke_secret(&secret_id, &user_id).await {
                Ok(()) => {
                    set_secrets.update(|s| s.retain(|sec| sec.id != secret_id));
                    set_revoking_id.set(None);
                }
                Err(e) => {
                    set_error.set(e);
                    set_revoking_id.set(None);
                }
            }
        });
    };

    view! {
        <div class="credentials-view">

            // ── Page header ────────────────────────────────────
            <div class="credentials-header">
                <div class="credentials-header-icon">
                    <IconKey class="icon-svg" />
                </div>
                <div>
                    <h1 class="credentials-title">"Developer Credentials"</h1>
                    <p class="credentials-subtitle">
                        "Use your MID ID and a MID Secret to authenticate your games and "
                        "tools with MidManStudio services."
                    </p>
                </div>
            </div>

            // ── MID ID card ────────────────────────────────────
            <div class="cred-card">
                <div class="cred-card-head">
                    <div>
                        <h2 class="cred-card-title">"MID ID"</h2>
                        <p class="cred-card-desc">
                            "Your public developer identifier. Safe to include in client-side code."
                        </p>
                    </div>
                </div>
                <div class="mid-id-display">
                    <code class="mid-id-value">
                        {move || profile.get()
                            .as_ref()
                            .map(|p| p.mid_id.clone())
                            .unwrap_or_else(|| "Loading...".to_string())}
                    </code>
                    <button
                        class="copy-btn"
                        on:click=handle_copy_mid_id
                        title="Copy MID ID"
                    >
                        {move || if mid_id_copied.get() {
                            view! {
                                <IconCheck class="icon-svg icon-xs" />
                                <span>"Copied"</span>
                            }.into_any()
                        } else {
                            view! {
                                <IconCopy class="icon-svg icon-xs" />
                                <span>"Copy"</span>
                            }.into_any()
                        }}
                    </button>
                </div>
            </div>

            // ── Revealed secret banner — shown ONCE ────────────
            {move || if let Some(secret) = revealed.get() {
                view! {
                    <div class="revealed-banner">
                        <div class="revealed-banner-head">
                            <div class="revealed-banner-icon">
                                <IconAlertTriangle class="icon-svg icon-sm" />
                            </div>
                            <div>
                                <p class="revealed-banner-title">"Save your secret now"</p>
                                <p class="revealed-banner-sub">
                                    "This secret will never be shown again. Copy it somewhere safe before dismissing."
                                </p>
                            </div>
                        </div>
                        <div class="revealed-secret-row">
                            <code class="revealed-secret-value">{secret.clone()}</code>
                            <button
                                class="copy-btn copy-btn--large"
                                on:click=handle_copy_secret
                            >
                                {move || if secret_copied.get() {
                                    view! {
                                        <IconCheck class="icon-svg icon-xs" />
                                        <span>"Copied!"</span>
                                    }.into_any()
                                } else {
                                    view! {
                                        <IconCopy class="icon-svg icon-xs" />
                                        <span>"Copy Secret"</span>
                                    }.into_any()
                                }}
                            </button>
                        </div>
                        <button
                            class="revealed-dismiss"
                            on:click=move |_| set_revealed.set(None)
                        >
                            "I have saved my secret — dismiss"
                        </button>
                    </div>
                }.into_any()
            } else {
                view! { <span></span> }.into_any()
            }}

            // ── Secrets list card ──────────────────────────────
            <div class="cred-card">
                <div class="cred-card-head">
                    <div>
                        <h2 class="cred-card-title">"MID Secrets"</h2>
                        <p class="cred-card-desc">
                            "Secret tokens used alongside your MID ID. "
                            "Create one per project. Revoke old ones when rotating."
                        </p>
                    </div>
                    {move || if !show_form.get() {
                        view! {
                            <button
                                class="btn btn-primary btn-sm"
                                on:click=move |_| {
                                    set_show_form.set(true);
                                    set_gen_error.set(String::new());
                                }
                            >
                                <IconPlus class="icon-svg icon-xs" />
                                "Generate Secret"
                            </button>
                        }.into_any()
                    } else {
                        view! { <span></span> }.into_any()
                    }}
                </div>

                // ── Generate form ──────────────────────────────
                {move || if show_form.get() {
                    view! {
                        <div class="gen-form-wrap">
                            <form class="gen-form" on:submit=handle_generate>
                                <div class="form-group">
                                    <label class="form-label">
                                        "Label "
                                        <span class="form-label-optional">"(optional)"</span>
                                    </label>
                                    <input
                                        class="form-input"
                                        type="text"
                                        placeholder="e.g. Game Client, CI Pipeline, Unity SDK"
                                        prop:value=move || new_label.get()
                                        on:input=move |ev| set_new_label.set(event_target_value(&ev))
                                        maxlength="64"
                                    />
                                </div>

                                {move || if !gen_error.get().is_empty() {
                                    view! {
                                        <div class="status-msg status-msg--error">
                                            {gen_error.get()}
                                        </div>
                                    }.into_any()
                                } else { view! { <span></span> }.into_any() }}

                                <div class="gen-form-actions">
                                    <button
                                        type="button"
                                        class="btn btn-ghost btn-sm"
                                        on:click=move |_| {
                                            set_show_form.set(false);
                                            set_new_label.set(String::new());
                                            set_gen_error.set(String::new());
                                        }
                                        disabled=move || generating.get()
                                    >
                                        "Cancel"
                                    </button>
                                    <button
                                        type="submit"
                                        class="btn btn-primary btn-sm"
                                        disabled=move || generating.get()
                                    >
                                        {move || if generating.get() {
                                            view! {
                                                <IconLoader class="icon-svg spin" />
                                                <span>"Generating..."</span>
                                            }.into_any()
                                        } else {
                                            view! {
                                                <IconKey class="icon-svg icon-xs" />
                                                <span>"Generate"</span>
                                            }.into_any()
                                        }}
                                    </button>
                                </div>
                            </form>
                        </div>
                    }.into_any()
                } else {
                    view! { <span></span> }.into_any()
                }}

                // ── Global error ───────────────────────────────
                {move || if !error.get().is_empty() {
                    view! {
                        <div class="status-msg status-msg--error">{error.get()}</div>
                    }.into_any()
                } else { view! { <span></span> }.into_any() }}

                // ── List ───────────────────────────────────────
                {move || if loading.get() {
                    view! {
                        <div class="secrets-empty">
                            <div class="spinner"></div>
                        </div>
                    }.into_any()
                } else if secrets.get().is_empty() {
                    view! {
                        <div class="secrets-empty">
                            <IconKey class="icon-svg secrets-empty-icon" />
                            <p class="secrets-empty-title">"No secrets yet"</p>
                            <p class="secrets-empty-sub">
                                "Generate your first secret to start authenticating "
                                "your projects with MmS services."
                            </p>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <div class="secrets-list">
                            {move || {
                                // Snapshot the handle_revoke closure in an Arc so
                                // each per-row reactive closure can clone it cheaply
                                // without capturing a non-Send Rc.
                                let revoke = Arc::new(handle_revoke.clone());

                                secrets.get().into_iter().map(|secret| {
                                    // Arc<String> is Send — safe to capture in
                                    // a Leptos reactive closure.
                                    let sid: Arc<String> = Arc::new(secret.id.clone());
                                    let revoke = revoke.clone();

                                    view! {
                                        <div class="secret-row">
                                            <div class="secret-row-left">
                                                <div class="secret-row-icon">
                                                    <IconKey class="icon-svg icon-xs" />
                                                </div>
                                                <div class="secret-row-info">
                                                    <span class="secret-label">
                                                        {secret.display_label()}
                                                    </span>
                                                    <code class="secret-prefix">
                                                        {secret.display_prefix()}
                                                    </code>
                                                </div>
                                            </div>
                                            <div class="secret-row-meta">
                                                <span class="secret-meta-item">
                                                    <span class="secret-meta-label">"Created"</span>
                                                    <span class="secret-meta-value">
                                                        {secret.formatted_created()}
                                                    </span>
                                                </span>
                                                <span class="secret-meta-item">
                                                    <span class="secret-meta-label">"Last used"</span>
                                                    <span class="secret-meta-value">
                                                        {secret.formatted_last_used()}
                                                    </span>
                                                </span>
                                            </div>
                                            <div class="secret-row-actions">
                                                {
                                                    let sid = sid.clone();
                                                    let revoke = revoke.clone();
                                                    move || {
                                                        let is_confirming   = confirm_revoke.get().as_deref() == Some(sid.as_str());
                                                        let is_revoking_now = revoking_id.get().as_deref()    == Some(sid.as_str());

                                                        if is_confirming {
                                                            let sid_confirm = (*sid).clone();
                                                            let sid_cancel  = (*sid).clone();
                                                            let revoke      = revoke.clone();

                                                            view! {
                                                                <span class="revoke-confirm-label">
                                                                    "Revoke this secret?"
                                                                </span>
                                                                <button
                                                                    class="btn btn-danger btn-sm"
                                                                    disabled=is_revoking_now
                                                                    on:click=move |_| revoke(sid_confirm.clone())
                                                                >
                                                                    {if is_revoking_now {
                                                                        view! {
                                                                            <IconLoader class="icon-svg spin" />
                                                                            <span>"Revoking..."</span>
                                                                        }.into_any()
                                                                    } else {
                                                                        view! {
                                                                            <span>"Yes, revoke"</span>
                                                                        }.into_any()
                                                                    }}
                                                                </button>
                                                                <button
                                                                    class="btn btn-ghost btn-sm"
                                                                    on:click=move |_| {
                                                                        if confirm_revoke.get().as_deref() == Some(sid_cancel.as_str()) {
                                                                            set_confirm_revoke.set(None);
                                                                        }
                                                                    }
                                                                >
                                                                    "Cancel"
                                                                </button>
                                                            }.into_any()
                                                        } else {
                                                            let sid_click = (*sid).clone();
                                                            view! {
                                                                <button
                                                                    class="revoke-btn"
                                                                    title="Revoke secret"
                                                                    on:click=move |_| {
                                                                        set_confirm_revoke.set(Some(sid_click.clone()))
                                                                    }
                                                                >
                                                                    <IconTrash class="icon-svg icon-xs" />
                                                                    "Revoke"
                                                                </button>
                                                            }.into_any()
                                                        }
                                                    }
                                                }
                                            </div>
                                        </div>
                                    }
                                }).collect_view()
                            }}
                        </div>
                    }.into_any()
                }}
            </div>

            // ── Usage guide card ───────────────────────────────
            <div class="cred-card cred-card--info">
                <h2 class="cred-card-title">"How to use"</h2>
                <div class="usage-steps">
                    <div class="usage-step">
                        <div class="usage-step-num">"1"</div>
                        <div>
                            <p class="usage-step-title">"Authenticate"</p>
                            <p class="usage-step-desc">
                                "Send your MID ID and MID Secret in the request headers to any MmS API endpoint."
                            </p>
                            <pre class="usage-code">
                                "X-MMS-MID-ID: your_mid_id\nX-MMS-SECRET: mids_your_secret"
                            </pre>
                        </div>
                    </div>
                    <div class="usage-step">
                        <div class="usage-step-num">"2"</div>
                        <div>
                            <p class="usage-step-title">"Keep secrets server-side"</p>
                            <p class="usage-step-desc">
                                "Never expose your MID Secret in client-side or public code. "
                                "Store it in environment variables or a secrets manager."
                            </p>
                        </div>
                    </div>
                    <div class="usage-step">
                        <div class="usage-step-num">"3"</div>
                        <div>
                            <p class="usage-step-title">"Rotate regularly"</p>
                            <p class="usage-step-desc">
                                "Generate a new secret, update your app, then revoke the old one. "
                                "Each project should have its own secret."
                            </p>
                        </div>
                    </div>
                </div>
            </div>

        </div>
    }
                    }
