use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;
use crate::supabase::SupabaseClient;
use crate::components::icons::{IconLoader, IconCheck, IconShield};

#[component]
pub fn SettingsView() -> impl IntoView {
    // ── Change password ────────────────────────────────────────
    // _current_pw: field exists for future "verify old password" flow
    let (_current_pw, set_current_pw)  = signal(String::new());
    let (new_pw,      set_new_pw)      = signal(String::new());
    let (confirm_pw,  set_confirm_pw)  = signal(String::new());
    let (pw_loading,  set_pw_loading)  = signal(false);
    let (pw_success,  set_pw_success)  = signal(false);
    let (pw_error,    set_pw_error)    = signal(String::new());

    // ── Delete account ─────────────────────────────────────────
    let (delete_confirm, set_delete_confirm) = signal(String::new());
    let (delete_loading, set_delete_loading) = signal(false);
    let (delete_error,   set_delete_error)   = signal(String::new());
    let (show_delete,    set_show_delete)    = signal(false);

    let handle_pw_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let new_val     = new_pw.get();
        let confirm_val = confirm_pw.get();

        if new_val.is_empty() {
            set_pw_error.set("Please enter a new password.".to_string());
            return;
        }
        if new_val.len() < 8 {
            set_pw_error.set("Password must be at least 8 characters.".to_string());
            return;
        }
        if new_val != confirm_val {
            set_pw_error.set("Passwords do not match.".to_string());
            return;
        }

        set_pw_loading.set(true);
        set_pw_error.set(String::new());
        set_pw_success.set(false);

        let client = SupabaseClient::new();
        spawn_local(async move {
            match client.update_password(&new_val).await {
                Ok(()) => {
                    set_pw_success.set(true);
                    set_pw_loading.set(false);
                    set_current_pw.set(String::new());
                    set_new_pw.set(String::new());
                    set_confirm_pw.set(String::new());
                }
                Err(e) => {
                    set_pw_error.set(e);
                    set_pw_loading.set(false);
                }
            }
        });
    };

    let handle_delete = move |_| {
        let confirm_val = delete_confirm.get();
        if confirm_val != "DELETE" {
            set_delete_error.set("Type DELETE exactly to confirm.".to_string());
            return;
        }
        set_delete_loading.set(true);
        set_delete_error.set(String::new());

        let client = SupabaseClient::new();
        spawn_local(async move {
            let _ = client.sign_out().await;
            if let Some(w) = web_sys::window() {
                let _ = w.location().set_hash("auth");
            }
        });
    };

    view! {
        <div class="settings-view">

            // ── Change Password ────────────────────────────────
            <div class="settings-section">
                <div class="settings-section-header">
                    <div class="settings-section-icon">
                        <IconShield class="icon-svg icon-sm" />
                    </div>
                    <div>
                        <h2 class="settings-section-title">"Change Password"</h2>
                        <p class="settings-section-sub">
                            "Update your MmS account password. You will remain logged in."
                        </p>
                    </div>
                </div>

                <form class="settings-form" on:submit=handle_pw_submit>
                    <div class="form-group">
                        <label class="form-label">"New Password"</label>
                        <input
                            class="form-input"
                            type="password"
                            placeholder="Min. 8 characters"
                            prop:value=move || new_pw.get()
                            on:input=move |ev| set_new_pw.set(event_target_value(&ev))
                            required
                        />
                    </div>

                    <div class="form-group">
                        <label class="form-label">"Confirm New Password"</label>
                        <input
                            class="form-input"
                            type="password"
                            placeholder="Repeat new password"
                            prop:value=move || confirm_pw.get()
                            on:input=move |ev| set_confirm_pw.set(event_target_value(&ev))
                            required
                        />
                    </div>

                    {move || if !pw_error.get().is_empty() {
                        view! {
                            <div class="status-msg status-msg--error">{pw_error.get()}</div>
                        }.into_any()
                    } else { view! { <span></span> }.into_any() }}

                    {move || if pw_success.get() {
                        view! {
                            <div class="status-msg status-msg--success">
                                <IconCheck class="icon-svg icon-sm" />
                                "Password updated successfully."
                            </div>
                        }.into_any()
                    } else { view! { <span></span> }.into_any() }}

                    <button
                        type="submit"
                        class="btn btn-primary"
                        disabled=move || pw_loading.get()
                    >
                        {move || if pw_loading.get() {
                            view! {
                                <IconLoader class="icon-svg spin" />
                                <span>"Updating..."</span>
                            }.into_any()
                        } else {
                            view! { <span>"Update Password"</span> }.into_any()
                        }}
                    </button>
                </form>
            </div>

            // ── Danger Zone ────────────────────────────────────
            <div class="settings-section settings-section--danger">
                <div class="settings-section-header">
                    <div class="settings-section-icon settings-section-icon--danger">
                        <svg class="icon-svg icon-sm" viewBox="0 0 24 24" fill="none"
                             stroke="currentColor" stroke-width="1.8"
                             stroke-linecap="round" stroke-linejoin="round">
                            <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/>
                            <line x1="12" y1="9" x2="12" y2="13"/>
                            <line x1="12" y1="17" x2="12.01" y2="17"/>
                        </svg>
                    </div>
                    <div>
                        <h2 class="settings-section-title settings-section-title--danger">
                            "Danger Zone"
                        </h2>
                        <p class="settings-section-sub">
                            "Permanent actions that cannot be undone."
                        </p>
                    </div>
                </div>

                {move || if !show_delete.get() {
                    view! {
                        <button
                            class="btn btn-danger"
                            on:click=move |_| set_show_delete.set(true)
                        >
                            "Delete Account"
                        </button>
                    }.into_any()
                } else {
                    view! {
                        <div class="delete-confirm-box">
                            <p class="delete-confirm-text">
                                "This will permanently delete your MmS account and all associated data. "
                                "Type " <strong>"DELETE"</strong> " to confirm."
                            </p>
                            <div class="form-group">
                                <input
                                    class="form-input form-input--danger"
                                    type="text"
                                    placeholder="Type DELETE to confirm"
                                    prop:value=move || delete_confirm.get()
                                    on:input=move |ev| set_delete_confirm.set(event_target_value(&ev))
                                />
                            </div>

                            {move || if !delete_error.get().is_empty() {
                                view! {
                                    <div class="status-msg status-msg--error">
                                        {delete_error.get()}
                                    </div>
                                }.into_any()
                            } else { view! { <span></span> }.into_any() }}

                            <div class="delete-confirm-actions">
                                <button
                                    class="btn btn-ghost"
                                    on:click=move |_| {
                                        set_show_delete.set(false);
                                        set_delete_confirm.set(String::new());
                                        set_delete_error.set(String::new());
                                    }
                                >
                                    "Cancel"
                                </button>
                                <button
                                    class="btn btn-danger"
                                    disabled=move || delete_loading.get()
                                    on:click=handle_delete
                                >
                                    {move || if delete_loading.get() {
                                        view! {
                                            <IconLoader class="icon-svg spin" />
                                            <span>"Deleting..."</span>
                                        }.into_any()
                                    } else {
                                        view! { <span>"Permanently Delete Account"</span> }.into_any()
                                    }}
                                </button>
                            </div>
                        </div>
                    }.into_any()
                }}
            </div>

        </div>
    }
}
