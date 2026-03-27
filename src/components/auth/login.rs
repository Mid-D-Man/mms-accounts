use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;
use crate::supabase::SupabaseClient;
use crate::components::icons::{IconMail, IconLock, IconLoader};

#[component]
pub fn LoginForm<F>(on_switch: F) -> impl IntoView
where
    F: Fn() + 'static + Clone,
{
    let (email,    set_email)    = signal(String::new());
    let (password, set_password) = signal(String::new());
    let (error,    set_error)    = signal(String::new());
    let (loading,  set_loading)  = signal(false);

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

        let client = SupabaseClient::new();

        spawn_local(async move {
            match client.sign_in(&email_val, &password_val).await {
                Ok(_) => {
                    if let Some(window) = web_sys::window() {
                        let _ = window.location().set_hash("dashboard");
                    }
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
        </div>
    }
}
