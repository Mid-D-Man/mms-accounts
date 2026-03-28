use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;
use crate::supabase::SupabaseClient;
use crate::components::icons::{IconMail, IconLoader, IconCheck};

#[component]
pub fn ForgotPasswordForm<F>(on_back: F) -> impl IntoView
where
    F: Fn() + 'static + Clone + Send + Sync,
{
    let (email,   set_email)   = signal(String::new());
    let (loading, set_loading) = signal(false);
    let (sent,    set_sent)    = signal(false);
    let (error,   set_error)   = signal(String::new());

    let on_back_clone = on_back.clone();

    let handle_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();

        let email_val = email.get();
        if email_val.is_empty() {
            set_error.set("Please enter your email address.".to_string());
            return;
        }

        set_loading.set(true);
        set_error.set(String::new());

        let client = SupabaseClient::new();

        spawn_local(async move {
            // Supabase always returns 200 here even if the email doesn't exist —
            // this is intentional to prevent email enumeration attacks.
            match client.send_password_recovery(&email_val).await {
                Ok(()) => {
                    set_sent.set(true);
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
        <div class="auth-form-wrap">
            {move || if sent.get() {
                // ── Sent confirmation ──────────────────────────
                view! {
                    <div class="signup-confirmed">
                        <div class="signup-confirmed-icon">
                            <IconCheck class="icon-svg icon-lg" />
                        </div>
                        <h3 class="signup-confirmed-title">"Check your email"</h3>
                        <p class="signup-confirmed-body">
                            "If an account exists for "
                            <strong>{email.get()}</strong>
                            ", a password reset link is on its way. It expires in 1 hour."
                        </p>
                        <p class="signup-confirmed-note">
                            "Didn't receive it? Check your spam folder or try again."
                        </p>
                        <button
                            class="btn btn-ghost btn-full"
                            on:click=move |_| {
                                set_sent.set(false);
                                set_email.set(String::new());
                            }
                        >
                            "Try a different email"
                        </button>
                        <button
                            class="btn btn-primary btn-full"
                            on:click=move |_| on_back_clone()
                        >
                            "Back to Sign In"
                        </button>
                    </div>
                }.into_any()
            } else {
                // ── Request form ───────────────────────────────
                let on_back_inner = on_back.clone();
                view! {
                    <div class="forgot-header">
                        <h3 class="forgot-title">"Forgot your password?"</h3>
                        <p class="forgot-sub">
                            "Enter your account email and we'll send a reset link."
                        </p>
                    </div>

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
                                    autofocus
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
                                    <span>"Sending..."</span>
                                }.into_any()
                            } else {
                                view! { <span>"Send Reset Link"</span> }.into_any()
                            }}
                        </button>

                    </form>

                    <div class="auth-switch">
                        <button class="link-btn" on:click=move |_| on_back_inner()>
                            "Back to Sign In"
                        </button>
                    </div>
                }.into_any()
            }}
        </div>
    }
}
