use leptos::prelude::*;
use wasm_bindgen::JsCast;
use crate::components::{AuthPage, Dashboard};

#[component]
pub fn App() -> impl IntoView {
    let (current_page, set_current_page) = signal(get_current_route());
    
    Effect::new(move |_| {
        if let Some(window) = web_sys::window() {
            let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
                set_current_page.set(get_current_route());
            }) as Box<dyn Fn()>);
            
            let _ = window.add_event_listener_with_callback(
                "hashchange",
                closure.as_ref().unchecked_ref(),
            );
            closure.forget();
        }
    });

    view! {
        <div class="mms-app">
            {move || match current_page.get().as_str() {
                "auth" => view! { <AuthPage /> }.into_any(),
                "dashboard" => view! { <Dashboard /> }.into_any(),
                _ => view! {
                    <div class="landing">
                        <h1>"MidManStudio Accounts"</h1>
                        <p>"Manage your MmS projects in one place"</p>
                        <a href="#auth" class="btn-primary">"Get Started"</a>
                    </div>
                }.into_any(),
            }}
        </div>
    }
}

fn get_current_route() -> String {
    web_sys::window()
        .and_then(|w| w.location().hash().ok())
        .unwrap_or_default()
        .trim_start_matches('#')
        .to_string()
}
