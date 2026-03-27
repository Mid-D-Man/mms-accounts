use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;
use gloo_storage::Storage;
use std::rc::Rc;
use crate::supabase::{SupabaseClient, Profile};
use crate::components::icons::{IconLoader, IconCheck};

#[component]
pub fn ProfileView(
    profile:    ReadSignal<Option<Profile>>,
    on_updated: impl Fn(Profile) + 'static,
) -> impl IntoView {
    let (display_name, set_display_name) = signal(String::new());
    let (bio,          set_bio)          = signal(String::new());
    let (website,      set_website)      = signal(String::new());
    let (loading,      set_loading)      = signal(false);
    let (success,      set_success)      = signal(false);
    let (error,        set_error)        = signal(String::new());

    // Wrap in Rc so the closure stays FnMut — each submit clones the Rc,
    // never moves out of the outer closure.
    let on_updated = Rc::new(on_updated);

    Effect::new(move |_| {
        if let Some(p) = profile.get() {
            set_display_name.set(p.display_name.unwrap_or_default());
            set_bio.set(p.bio.unwrap_or_default());
            set_website.set(p.website.unwrap_or_default());
        }
    });

    let handle_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();

        let user_id = match gloo_storage::LocalStorage::get::<String>("mms_user_id") {
            Ok(id) => id,
            Err(_) => {
                set_error.set("Not authenticated.".to_string());
                return;
            }
        };

        set_loading.set(true);
        set_error.set(String::new());
        set_success.set(false);

        let name_val    = display_name.get();
        let bio_val     = bio.get();
        let website_val = website.get();
        let client      = SupabaseClient::new();

        // Clone the Rc for this particular spawn — outer closure is untouched.
        let on_updated = Rc::clone(&on_updated);

        spawn_local(async move {
            match client.update_profile(
                &user_id,
                Some(name_val).filter(|s| !s.is_empty()),
                Some(bio_val).filter(|s| !s.is_empty()),
                Some(website_val).filter(|s| !s.is_empty()),
                None,
            ).await {
                Ok(updated) => {
                    on_updated(updated);
                    set_success.set(true);
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
        <div class="profile-view">
            <div class="profile-view-header">
                <h2 class="profile-view-title">"Edit Profile"</h2>
                <p class="profile-view-sub">
                    "Update your MidManStudio identity."
                </p>
            </div>

            <form class="profile-form" on:submit=handle_submit>

                <div class="form-section">
                    <h3 class="form-section-label">"Public Info"</h3>

                    <div class="form-group">
                        <label class="form-label">"Display Name"</label>
                        <input
                            class="form-input"
                            type="text"
                            placeholder="How you appear to others"
                            prop:value=move || display_name.get()
                            on:input=move |ev| set_display_name.set(event_target_value(&ev))
                        />
                    </div>

                    <div class="form-group">
                        <label class="form-label">"Bio"</label>
                        <textarea
                            class="form-textarea"
                            placeholder="Tell the studio a bit about yourself..."
                            prop:value=move || bio.get()
                            on:input=move |ev| set_bio.set(event_target_value(&ev))
                            rows="4"
                        ></textarea>
                    </div>

                    <div class="form-group">
                        <label class="form-label">"Website"</label>
                        <input
                            class="form-input"
                            type="url"
                            placeholder="https://your-site.com"
                            prop:value=move || website.get()
                            on:input=move |ev| set_website.set(event_target_value(&ev))
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

                {move || if success.get() {
                    view! {
                        <div class="status-msg status-msg--success">
                            <IconCheck class="icon-svg icon-sm" />
                            "Profile updated successfully."
                        </div>
                    }.into_any()
                } else {
                    view! { <span></span> }.into_any()
                }}

                <button
                    type="submit"
                    class="btn btn-primary"
                    disabled=move || loading.get()
                >
                    {move || if loading.get() {
                        view! {
                            <IconLoader class="icon-svg spin" />
                            <span>"Saving..."</span>
                        }.into_any()
                    } else {
                        view! { <span>"Save Changes"</span> }.into_any()
                    }}
                </button>

            </form>
        </div>
    }
}
