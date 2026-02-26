pub mod login;
pub mod signup;

use leptos::prelude::*;
use wasm_bindgen::JsCast;

pub use login::*;
pub use signup::*;

#[component]
pub fn AuthPage() -> impl IntoView {
    let (mode, set_mode) = signal("signin");

    view! {
        <div class="auth-page">
            <div class="auth-container">
                <div class="auth-header">
                    <h1>"MidManStudio"</h1>
                    <p>"Accounts"</p>
                </div>

                {move || if mode.get() == "signin" {
                    view! { <LoginForm on_switch=move || set_mode.set("signup") /> }.into_any()
                } else {
                    view! { <SignupForm on_switch=move || set_mode.set("signin") /> }.into_any()
                }}
            </div>
        </div>
    }
}
