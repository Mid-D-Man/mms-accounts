pub mod profile;

use leptos::prelude::*;
use crate::supabase::SupabaseClient;

pub use profile::*;

#[component]
pub fn Dashboard() -> impl IntoView {
    let (user, set_user) = signal(None::<String>);
    let (loading, set_loading) = signal(true);

    let client = SupabaseClient::new();

    Effect::new(move |_| {
        let client_clone = client.clone();
        spawn_local(async move {
            match client_clone.get_user().await {
                Ok(user_data) => {
                    set_user.set(Some(user_data.email));
                    set_loading.set(false);
                }
                Err(_) => {
                    if let Some(window) = web_sys::window() {
                        let _ = window.location().set_hash("#auth");
                    }
                }
            }
        });
    });

    let handle_signout = move |_| {
        client.sign_out();
        if let Some(window) = web_sys::window() {
            let _ = window.location().set_hash("#auth");
        }
    };

    view! {
        <div class="dashboard">
            {move || if loading.get() {
                view! { <p>"Loading..."</p> }.into_any()
            } else {
                view! {
                    <div class="dashboard-content">
                        <h1>"Welcome, " {user.get().unwrap_or_default()}</h1>
                        <button class="btn-secondary" on:click=handle_signout>
                            "Sign Out"
                        </button>
                    </div>
                }.into_any()
            }}
        </div>
    }
}
