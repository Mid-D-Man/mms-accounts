pub mod login;
pub mod signup;

pub use login::*;
pub use signup::*;

use leptos::prelude::*;
use crate::components::icons::IconShield;

#[component]
pub fn AuthPage() -> impl IntoView {
    let (mode, set_mode) = signal("signin");

    view! {
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

                <div class="auth-tab-row">
                    <button
                        class=move || {
                            if mode.get() == "signin" {
                                "auth-tab auth-tab--active"
                            } else {
                                "auth-tab"
                            }
                        }
                        on:click=move |_| set_mode.set("signin")
                    >
                        "Sign In"
                    </button>
                    <button
                        class=move || {
                            if mode.get() == "signup" {
                                "auth-tab auth-tab--active"
                            } else {
                                "auth-tab"
                            }
                        }
                        on:click=move |_| set_mode.set("signup")
                    >
                        "Create Account"
                    </button>
                </div>

                {move || if mode.get() == "signin" {
                    view! {
                        <LoginForm on_switch=move || set_mode.set("signup") />
                    }.into_any()
                } else {
                    view! {
                        <SignupForm on_switch=move || set_mode.set("signin") />
                    }.into_any()
                }}

                <div class="auth-footer-link">
                    <a href="#" class="auth-back-home">"Back to home"</a>
                </div>
            </div>
        </div>
    }
}
