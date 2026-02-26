use leptos::prelude::*;
use wasm_bindgen::JsCast;
use crate::supabase::SupabaseClient;

#[component]
pub fn SignupForm<F>(on_switch: F) -> impl IntoView
where
    F: Fn() + 'static + Clone,
{
    let (email, set_email) = signal(String::new());
    let (password, set_password) = signal(String::new());
    let (display_name, set_display_name) = signal(String::new());
    let (status, set_status) = signal(String::new());
    let (loading, set_loading) = signal(false);

    let client = SupabaseClient::new();

    let handle_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        set_loading.set(true);
        set_status.set(String::new());

        let email_val = email.get();
        let password_val = password.get();
        let display_name_val = display_name.get();
        let client_clone = client.clone();

        spawn_local(async move {
            let metadata = serde_json::json!({
                "display_name": display_name_val
            });

            match client_clone.sign_up(&email_val, &password_val, Some(metadata)).await {
                Ok(_) => {
                    set_status.set("✓ Account created!".to_string());
                    if let Some(window) = web_sys::window() {
                        let _ = window.location().set_hash("#dashboard");
                    }
                }
                Err(e) => {
                    set_status.set(format!("✗ {}", e));
                }
            }
            set_loading.set(false);
        });
    };

    view! {
        <div class="auth-form-wrapper">
            <h2>"Create Account"</h2>
            
            <form class="auth-form" on:submit=handle_submit>
                <div class="form-group">
                    <label>"Display Name"</label>
                    <input
                        type="text"
                        placeholder="Mid-D-Man"
                        prop:value=move || display_name.get()
                        on:input=move |ev| set_display_name.set(event_target_value(&ev))
                        required
                    />
                </div>

                <div class="form-group">
                    <label>"Email"</label>
                    <input
                        type="email"
                        placeholder="you@example.com"
                        prop:value=move || email.get()
                        on:input=move |ev| set_email.set(event_target_value(&ev))
                        required
                    />
                </div>

                <div class="form-group">
                    <label>"Password"</label>
                    <input
                        type="password"
                        placeholder="••••••••"
                        prop:value=move || password.get()
                        on:input=move |ev| set_password.set(event_target_value(&ev))
                        required
                    />
                </div>

                {move || if !status.get().is_empty() {
                    view! {
                        <div class=move || {
                            if status.get().starts_with('✓') {
                                "auth-status success"
                            } else {
                                "auth-status error"
                            }
                        }>
                            {status.get()}
                        </div>
                    }.into_any()
                } else {
                    view! { <div></div> }.into_any()
                }}

                <button
                    type="submit"
                    class="btn-primary"
                    prop:disabled=move || loading.get()
                >
                    {move || if loading.get() { "Creating..." } else { "Create Account" }}
                </button>
            </form>

            <div class="auth-toggle">
                <p>
                    "Already have an account? "
                    <button class="link-btn" on:click=move |_| on_switch()>
                        "Sign in"
                    </button>
                </p>
            </div>
        </div>
    }
}
