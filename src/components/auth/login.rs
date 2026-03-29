// src/components/auth/login.rs
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;
use crate::supabase::SupabaseClient;
use crate::components::icons::{IconMail, IconLock, IconLoader, IconCheck};

#[derive(Clone, PartialEq)]
enum LoginState {
    Idle,
    Unverified,
    Resent,
}

#[component]
pub fn LoginForm<F, G>(on_switch: F, on_forgot: G) -> impl IntoView
where
    F: Fn() + 'static + Clone + Send + Sync,
    G: Fn() + 'static + Clone + Send + Sync,
{
    let (email,       set_email)       = signal(String::new());
    let (password,    set_password)    = signal(String::new());
    let (error,       set_error)       = signal(String::new());
    let (loading,     set_loading)     = signal(false);
    let (login_state, set_login_state) = signal(LoginState::Idle);

    // ── Sign-in ────────────────────────────────────────────────

    let handle_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let email_val    = email.get();
        let password_val = password.get();

        if email_val.is_empty() || password_val.is_empty() {
            set_error.set("Please fill in all fields.".to_string());
            return;
        }

        set_loading.set(true);
        set_error.set(String::new());
        set_login_state.set(LoginState::Idle);

        let client = SupabaseClient::new();
        spawn_local(async move {
            match client.sign_in(&email_val, &password_val).await {
                Ok(_) => {
                    if let Some(w) = web_sys::window() {
                        let _ = w.location().set_hash("dashboard");
                    }
                }
                Err(e) => {
                    set_loading.set(false);
                    if e.to_lowercase().contains("email not confirmed") {
                        set_login_state.set(LoginState::Unverified);
                    } else {
                        set_error.set(e);
                    }
                }
            }
        });
    };

    // ── Resend verification ────────────────────────────────────

    let handle_resend = move |_| {
        let email_val = email.get();
        if email_val.is_empty() {
            set_error.set("Enter your email address above first.".to_string());
            return;
        }
        set_loading.set(true);
        set_error.set(String::new());
        let client = SupabaseClient::new();
        spawn_local(async move {
            match client.resend_verification(&email_val).await {
                Ok(())  => { set_login_state.set(LoginState::Resent); set_loading.set(false); }
                Err(e)  => { set_error.set(e); set_loading.set(false); }
            }
        });
    };

    // ── Wrap callbacks in Arc so reactive closures stay FnMut ──

    let on_switch = std::sync::Arc::new(on_switch);
    let on_forgot = std::sync::Arc::new(on_forgot);

    view! {
        <div class="auth-form-wrap">
            {move || {
                let on_switch = on_switch.clone();
                let on_forgot = on_forgot.clone();

                if login_state.get() == LoginState::Resent {
                    view! {
                        <div class="signup-confirmed">
                            <div class="signup-confirmed-icon">
                                <IconCheck class="icon-svg icon-lg" />
                            </div>
                            <h3 class="signup-confirmed-title">"Verification email sent"</h3>
                            <p class="signup-confirmed-body">
                                "Check your inbox for "
                                <strong>{email.get()}</strong>
                                " and click the confirmation link, then sign in."
                            </p>
                            <button
                                class="btn btn-ghost btn-full"
                                on:click=move |_| set_login_state.set(LoginState::Idle)
                            >
                                "Back to Sign In"
                            </button>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <form class="auth-form" on:submit=handle_submit>

                            <div class="form-group">
                                <label class="form-label">"Email"</label>
                                <div class="input-with-icon">
                                    <IconMail class="input-icon icon-svg" />
                                    <input
                                        class="form-input form-input--icon"
                                        type="email"
                                        placeholder="you@example.com"
                                        prop:value=move || email.get()
                                        on:input=move |ev| set_email.set(event_target_value(&ev))
                                        required
                                    />
                                </div>
                            </div>

                            <div class="form-group">
                                <label class="form-label">"Password"</label>
                                <div class="input-with-icon">
                                    <IconLock class="input-icon icon-svg" />
                                    <input
                                        class="form-input form-input--icon"
                                        type="password"
                                        placeholder="••••••••"
                                        prop:value=move || password.get()
                                        on:input=move |ev| set_password.set(event_target_value(&ev))
                                        required
                                    />
                                </div>
                                <div class="forgot-link-row">
                                    <button
                                        type="button"
                                        class="link-btn link-btn--muted"
                                        on:click=move |_| on_forgot()
                                    >
                                        "Forgot password?"
                                    </button>
                                </div>
                            </div>

                            {move || if !error.get().is_empty() {
                                view! {
                                    <div class="status-msg status-msg--error">{error.get()}</div>
                                }.into_any()
                            } else {
                                view! { <span></span> }.into_any()
                            }}

                            {move || if login_state.get() == LoginState::Unverified {
                                view! {
                                    <div class="verify-banner">
                                        <div class="verify-banner-text">
                                            <strong>"Email not verified."</strong>
                                            " Check your inbox for the confirmation link."
                                        </div>
                                        <button
                                            type="button"
                                            class="btn btn-ghost btn-sm verify-resend-btn"
                                            disabled=move || loading.get()
                                            on:click=handle_resend
                                        >
                                            {move || if loading.get() {
                                                view! {
                                                    <IconLoader class="icon-svg spin" />
                                                    <span>"Sending..."</span>
                                                }.into_any()
                                            } else {
                                                view! { <span>"Resend verification email"</span> }.into_any()
                                            }}
                                        </button>
                                    </div>
                                }.into_any()
                            } else {
                                view! { <span></span> }.into_any()
                            }}

                            <button
                                type="submit"
                                class="btn btn-primary btn-full"
                                disabled=move || loading.get()
                            >
                                {move || if loading.get() && login_state.get() != LoginState::Unverified {
                                    view! {
                                        <IconLoader class="icon-svg spin" />
                                        <span>"Signing in..."</span>
                                    }.into_any()
                                } else {
                                    view! { <span>"Sign In"</span> }.into_any()
                                }}
                            </button>

                        </form>

                        <div class="auth-switch">
                            "Don't have an account? "
                            <button class="link-btn" on:click=move |_| on_switch()>
                                "Create one"
                            </button>
                        </div>
                    }.into_any()
                }
            }}
        </div>
    }
      }
