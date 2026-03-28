use leptos::prelude::*;
use crate::supabase::client::SUPABASE_URL;

/// Redirects the user to Supabase's Google OAuth endpoint.
/// After auth, Google redirects back to the app with the session in the hash.
/// App.rs detects the access_token hash params and persists the session.
///
/// SETUP REQUIRED in Supabase Dashboard:
///   Authentication → Providers → Google → Enable
///   Set Client ID and Client Secret from Google Cloud Console
///   Authentication → URL Configuration → add your domain to Redirect URLs
///   e.g. https://mms-accounts.pages.dev and http://localhost:8080
fn start_google_oauth() {
    let Some(window) = web_sys::window() else { return };
    let origin = window.location().origin().unwrap_or_default();

    // redirect_to must be URL-encoded and must be in Supabase's allowed list
    let redirect_encoded = js_sys::encode_uri_component(&origin);

    let url = format!(
        "{}/auth/v1/authorize?provider=google&redirect_to={}",
        SUPABASE_URL,
        redirect_encoded
    );

    let _ = window.location().set_href(&url);
}

#[component]
pub fn GoogleSignInButton() -> impl IntoView {
    view! {
        <button
            class="btn btn-oauth btn-oauth--google"
            on:click=move |_| start_google_oauth()
            type="button"
        >
            // Google "G" icon as inline SVG — no emoji, no external resource
            <svg
                class="icon-svg icon-sm"
                viewBox="0 0 24 24"
                xmlns="http://www.w3.org/2000/svg"
                aria-hidden="true"
            >
                <path
                    d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z"
                    fill="#4285F4"
                />
                <path
                    d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z"
                    fill="#34A853"
                />
                <path
                    d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z"
                    fill="#FBBC05"
                />
                <path
                    d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z"
                    fill="#EA4335"
                />
            </svg>
            "Continue with Google"
        </button>
    }
}

/// Divider between OAuth and email/password forms
#[component]
pub fn OAuthDivider() -> impl IntoView {
    view! {
        <div class="oauth-divider">
            <span class="oauth-divider-line"></span>
            <span class="oauth-divider-text">"or"</span>
            <span class="oauth-divider-line"></span>
        </div>
    }
                                               }
