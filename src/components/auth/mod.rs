pub mod login;
pub mod signup;
pub mod forgot;
pub mod reset;
pub mod oauth_buttons;

pub use login::*;
pub use signup::*;
pub use forgot::*;
pub use reset::*;
pub use oauth_buttons::*;

use leptos::prelude::*;
use crate::components::icons::IconShield;

// Auth page modes
#[derive(Clone, PartialEq)]
enum AuthMode {
    SignIn,
    SignUp,
    Forgot,
}

#[component]
pub fn AuthPage() -> impl IntoView {
    let (mode, set_mode) = signal(AuthMode::SignIn);

    view! {
        <div class="auth-page">
            <div class="auth-panel">

                // Brand header
                <div class="auth-brand">
                    <div class="auth-brand-logo">
                        <IconShield class="icon-svg icon-lg" />
                    </div>
                    <div class="auth-brand-text">
                        <span class="auth-brand-mms">"MmS"</span>
                        <span class="auth-brand-accounts">"Accounts"</span>
                    </div>
                </div>

                // Tab row — only show on SignIn/SignUp, not Forgot
                {move || if mode.get() != AuthMode::Forgot {
                    view! {
                        <div class="auth-tab-row">
                            <button
                                class=move || {
                                    if mode.get() == AuthMode::SignIn {
                                        "auth-tab auth-tab--active"
                                    } else {
                                        "auth-tab"
                                    }
                                }
                                on:click=move |_| set_mode.set(AuthMode::SignIn)
                            >
                                "Sign In"
                            </button>
                            <button
                                class=move || {
                                    if mode.get() == AuthMode::SignUp {
                                        "auth-tab auth-tab--active"
                                    } else {
                                        "auth-tab"
                                    }
                                }
                                on:click=move |_| set_mode.set(AuthMode::SignUp)
                            >
                                "Create Account"
                            </button>
                        </div>
                    }.into_any()
                } else {
                    view! { <span></span> }.into_any()
                }}

                // Google OAuth — show on SignIn and SignUp tabs
                {move || if mode.get() != AuthMode::Forgot {
                    view! {
                        <div class="oauth-section">
                            <GoogleSignInButton />
                            <OAuthDivider />
                        </div>
                    }.into_any()
                } else {
                    view! { <span></span> }.into_any()
                }}

                // Main content area
                {move || match mode.get() {
                    AuthMode::SignIn => view! {
                        <LoginForm
                            on_switch=move || set_mode.set(AuthMode::SignUp)
                            on_forgot=move || set_mode.set(AuthMode::Forgot)
                        />
                    }.into_any(),
                    AuthMode::SignUp => view! {
                        <SignupForm
                            on_switch=move || set_mode.set(AuthMode::SignIn)
                        />
                    }.into_any(),
                    AuthMode::Forgot => view! {
                        <ForgotPasswordForm
                            on_back=move || set_mode.set(AuthMode::SignIn)
                        />
                    }.into_any(),
                }}

                <div class="auth-footer-link">
                    <a href="#" class="auth-back-home">"Back to home"</a>
                </div>

            </div>
        </div>
    }
}
