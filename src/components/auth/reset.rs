use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;
use crate::supabase::SupabaseClient;
use crate::components::icons::{IconLock, IconLoader, IconCheck, IconShield};

#[component]
pub fn ResetPasswordForm<F>(
    recovery_token: String,
    on_done:        F,
) -> impl IntoView
where
    F: Fn() + 'static + Clone + Send + Sync,
{
    let (password,  set_password)  = signal(String::new());
    let (confirm,   set_confirm)   = signal(String::new());
    let (loading,   set_loading)   = signal(false);
    let (success,   set_success)   = signal(false);
    let (error,     set_error)     = signal(String::new());

    // Keep the token in a reactive value so the submit closure can read it.
    let token = recovery_token.clone();
    let on_done_clone = on_done.clone();

    let handle_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();

        let password_val = password.get();
        let confirm_val  = confirm.get();

        if password_val.is_empty() {
            set_error.set("Please enter a new password.".to_string());
            return;
        }

        if password_val != confirm_val {
            set_error.set("Passwords do not match.".to_string());
            return;
        }

        if password_val.len() < 8 {
            set_error.set("Password must be at least 8 characters.".to_string());
            return;
        }

        set_loading.set(true);
        set_error.set(String::new());

        let client    = SupabaseClient::new();
        let token_val = token.clone();

        spawn_local(async move {
            match client.reset_password_with_token(&token_val, &password_val).await {
                Ok(()) => {
                    set_success.set(true);
                    set_loading.set(false);
                }
                Err(e) => {
                    set_error.set(e);
                    set_loading.set(false);
                }
            }
        });
    };

    view! {
        // Full-page wrapper — this replaces the entire app during recovery
        <div class="auth-page">
            <div class="auth-panel">

                <div class="auth-brand">
                    <div class="auth-brand-logo">
                        <IconShield class="icon-svg icon-lg" />
                    </div>
                    <div class="auth-brand-text">
                        <span class="auth-brand-mms">"MmS"</span>
                        <span class="auth-brand-accounts">"Accounts"</span>
                    </div>
                </div>

                {move || if success.get() {
                    // ── Success state ──────────────────────────
                    view! {
                        <div class="signup-confirmed">
                            <div class="signup-confirmed-icon">
                                <IconCheck class="icon-svg icon-lg" />
                            </div>
                            <h3 class="signup-confirmed-title">"Password updated"</h3>
                            <p class="signup-confirmed-body">
                                "Your password has been reset. You can now sign in with your new password."
                            </p>
                            <button
                                class="btn btn-primary btn-full"
                                on:click=move |_| on_done_clone()
                            >
                                "Go to Sign In"
                            </button>
                        </div>
                    }.into_any()
                } else {
                    // ── Reset form ─────────────────────────────
                    view! {
                        <div class="reset-header">
                            <h3 class="reset-title">"Set new password"</h3>
                            <p class="reset-sub">
                                "Choose a strong password for your MmS account."
                            </p>
                        </div>

                        <form class="auth-form" on:submit=handle_submit>

                            <div class="form-group">
                                <label class="form-label">"New Password"</label>
                                <div class="input-with-icon">
                                    <IconLock class="input-icon icon-svg" />
                                    <input
                                        class="form-input form-input--icon"
                                        type="password"
                                        placeholder="Min. 8 characters"
                                        prop:value=move || password.get()
                                        on:input=move |ev| set_password.set(event_target_value(&ev))
                                        required
                                        autofocus
                                    />
                                </div>
                            </div>

                            <div class="form-group">
                                <label class="form-label">"Confirm New Password"</label>
                                <div class="input-with-icon">
                                    <IconLock class="input-icon icon-svg" />
                                    <input
                                        class="form-input form-input--icon"
                                        type="password"
                                        placeholder="Repeat password"
                                        prop:value=move || confirm.get()
                                        on:input=move |ev| set_confirm.set(event_target_value(&ev))
                                        required
                                    />
                                </div>
                            </div>

                            {move || if !error.get().is_empty() {
                                view! {
                                    <div class="status-msg status-msg--error">{error.get()}</div>
                                }.into_any()
                            } else {
                                view! { <span></span> }.into_any()
                            }}

                            <button
                                type="submit"
                                class="btn btn-primary btn-full"
                                disabled=move || loading.get()
                            >
                                {move || if loading.get() {
                                    view! {
                                        <IconLoader class="icon-svg spin" />
                                        <span>"Updating..."</span>
                                    }.into_any()
                                } else {
                                    view! { <span>"Set New Password"</span> }.into_any()
                                }}
                            </button>

                        </form>
                    }.into_any()
                }}

            </div>
        </div>
    }
}
