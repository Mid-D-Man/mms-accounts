use leptos::prelude::*;
use std::rc::Rc;
use wasm_bindgen_futures::spawn_local;
use crate::supabase::{SupabaseClient, SignUpOutcome};
use crate::components::icons::{IconMail, IconLock, IconUser, IconLoader, IconCheck};

#[component]
pub fn SignupForm<F>(on_switch: F) -> impl IntoView
where
    F: Fn() + 'static + Clone,
{
    let (display_name, set_display_name) = signal(String::new());
    let (email,        set_email)        = signal(String::new());
    let (password,     set_password)     = signal(String::new());
    let (confirm,      set_confirm)      = signal(String::new());
    let (error,        set_error)        = signal(String::new());
    let (loading,      set_loading)      = signal(false);
    let (confirmed,    set_confirmed)    = signal(false);

    // Wrap in Rc so the reactive closure stays FnMut — cloning an Rc is
    // cheap and does not move the underlying value out of scope.
    let on_switch = Rc::new(on_switch);

    let handle_submit = {
        let on_switch = Rc::clone(&on_switch);
        move |ev: web_sys::SubmitEvent| {
            let _ = on_switch; // keep clone alive; not actually used here
            ev.prevent_default();

            let name_val     = display_name.get();
            let email_val    = email.get();
            let password_val = password.get();
            let confirm_val  = confirm.get();

            if name_val.is_empty() || email_val.is_empty() || password_val.is_empty() {
                set_error.set("Please fill in all fields.".to_string());
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

            let client   = SupabaseClient::new();
            let metadata = serde_json::json!({ "display_name": name_val });

            spawn_local(async move {
                match client.sign_up(&email_val, &password_val, Some(metadata)).await {
                    Ok(SignUpOutcome::LoggedIn) => {
                        if let Some(window) = web_sys::window() {
                            let _ = window.location().set_hash("dashboard");
                        }
                    }
                    Ok(SignUpOutcome::ConfirmationRequired) => {
                        set_confirmed.set(true);
                        set_loading.set(false);
                    }
                    Err(e) => {
                        set_error.set(e);
                        set_loading.set(false);
                    }
                }
            });
        }
    };

    view! {
        <div class="auth-form-wrap">

            {
                // Clone the Rc once per reactive evaluation — this keeps the
                // closure FnMut because Rc::clone never moves on_switch itself.
                let on_switch = Rc::clone(&on_switch);
                move || {
                    let on_switch = Rc::clone(&on_switch);
                    if confirmed.get() {
                        let os = Rc::clone(&on_switch);
                        view! {
                            <div class="signup-confirmed">
                                <div class="signup-confirmed-icon">
                                    <IconCheck class="icon-svg icon-lg" />
                                </div>
                                <h3 class="signup-confirmed-title">"Check your email"</h3>
                                <p class="signup-confirmed-body">
                                    "A confirmation link has been sent to "
                                    <strong>{email.get()}</strong>
                                    ". Click it to activate your account, then sign in."
                                </p>
                                <button
                                    class="btn btn-primary btn-full"
                                    on:click=move |_| os()
                                >
                                    "Go to Sign In"
                                </button>
                            </div>
                        }.into_any()
                    } else {
                        let os = Rc::clone(&on_switch);
                        view! {
                            <form class="auth-form" on:submit=handle_submit.clone()>

                                <div class="form-group">
                                    <label class="form-label">"Display Name"</label>
                                    <div class="input-with-icon">
                                        <IconUser class="input-icon icon-svg" />
                                        <input
                                            class="form-input form-input--icon"
                                            type="text"
                                            placeholder="Mid-D-Man"
                                            prop:value=move || display_name.get()
                                            on:input=move |ev| set_display_name.set(event_target_value(&ev))
                                            required
                                        />
                                    </div>
                                </div>

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
                                            placeholder="Min. 8 characters"
                                            prop:value=move || password.get()
                                            on:input=move |ev| set_password.set(event_target_value(&ev))
                                            required
                                        />
                                    </div>
                                </div>

                                <div class="form-group">
                                    <label class="form-label">"Confirm Password"</label>
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
                                            <span>"Creating account..."</span>
                                        }.into_any()
                                    } else {
                                        view! { <span>"Create Account"</span> }.into_any()
                                    }}
                                </button>

                            </form>

                            <div class="auth-switch">
                                "Already have an account? "
                                <button class="link-btn" on:click=move |_| os()>
                                    "Sign in"
                                </button>
                            </div>
                        }.into_any()
                    }
                }
            }

        </div>
    }
                }
