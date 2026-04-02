// src/components/dashboard/registry.rs
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;
use gloo_storage::Storage;
use js_sys::Uint8Array;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use crate::supabase::{SupabaseClient, Profile, RegistrySubmission, generate_path_id};
use crate::components::icons::{
    IconPackage, IconUpload, IconFileText, IconClock, IconCheck,
    IconAlertTriangle, IconLoader, IconEdit,
};

#[derive(Clone, PartialEq)]
enum SubmitTab {
    Upload,
    Write,
}

const CATEGORIES: &[(&str, &str)] = &[
    ("script",  "Script"),
    ("module",  "Module"),
    ("library", "Library"),
    ("tool",    "Tool"),
];

#[component]
pub fn RegistryView(profile: ReadSignal<Option<Profile>>) -> impl IntoView {
    let (submissions,  set_submissions)  = signal(Vec::<RegistrySubmission>::new());
    let (loading,      set_loading)      = signal(true);
    let (error,        set_error)        = signal(String::new());
    let (show_form,    set_show_form)    = signal(false);
    let (submitting,   set_submitting)   = signal(false);
    let (submit_error, set_submit_error) = signal(String::new());
    let (submit_tab,   set_submit_tab)   = signal(SubmitTab::Upload);

    // ── Upload form state ──────────────────────────────────────
    let (f_category,    set_f_category)    = signal("script".to_string());
    let (f_version,     set_f_version)     = signal(String::new());
    let (f_description, set_f_description) = signal(String::new());
    let (f_tags,        set_f_tags)        = signal(String::new());
    let (f_file,        set_f_file)        = signal(Option::<web_sys::File>::None);
    let (f_filename,    set_f_filename)    = signal(String::new());

    // ── Write form state ───────────────────────────────────────
    let (w_name,        set_w_name)        = signal(String::new());
    let (w_category,    set_w_category)    = signal("script".to_string());
    let (w_version,     set_w_version)     = signal(String::new());
    let (w_description, set_w_description) = signal(String::new());
    let (w_tags,        set_w_tags)        = signal(String::new());
    let (w_content,     set_w_content)     = signal(String::new());

    // ── Subscription check ─────────────────────────────────────
    let (is_subscribed, set_is_subscribed) = signal(false);
    let (checking_sub,  set_checking_sub)  = signal(true);

    // ── Load on mount ──────────────────────────────────────────
    Effect::new(move |_| {
        let user_id = gloo_storage::LocalStorage::get::<String>("mms_user_id")
            .unwrap_or_default();
        let client = SupabaseClient::new();
        spawn_local(async move {
            if let Ok(subs) = client.get_user_subscriptions(&user_id).await {
                if let Ok(services) = client.list_services().await {
                    if let Some(svc) = services.iter().find(|s| s.slug == "dixscript-registry") {
                        let active = subs.iter().any(|s| s.service_id == svc.id && s.is_active());
                        set_is_subscribed.set(active);
                    }
                }
            }
            set_checking_sub.set(false);

            match client.list_user_submissions(&user_id).await {
                Ok(s)  => { set_submissions.set(s);  set_loading.set(false); }
                Err(e) => { set_error.set(e);         set_loading.set(false); }
            }
        });
    });

    // ── File picker handler ────────────────────────────────────
    let handle_file_change = move |ev: web_sys::Event| {
        let input = ev.target()
            .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok());
        if let Some(input) = input {
            if let Some(files) = input.files() {
                if let Some(file) = files.get(0) {
                    set_f_filename.set(file.name());
                    set_f_file.set(Some(file));
                }
            }
        }
    };

    view! {
        <div class="registry-view">

            <div class="registry-header">
                <div class="registry-header-icon">
                    <IconPackage class="icon-svg" />
                </div>
                <div>
                    <h1 class="registry-title">"DixScript Registry"</h1>
                    <p class="registry-subtitle">
                        "Submit .mdix packages to the MmS cloud registry. "
                        "Approved packages are publicly accessible via MID ID."
                    </p>
                </div>
            </div>

            {move || if !error.get().is_empty() {
                view! {
                    <div class="status-msg status-msg--error">{error.get()}</div>
                }.into_any()
            } else { view! { <span></span> }.into_any() }}

            {move || if checking_sub.get() {
                view! {
                    <div class="registry-loading"><div class="spinner"></div></div>
                }.into_any()
            } else if !is_subscribed.get() {
                view! {
                    <div class="registry-gate">
                        <div class="registry-gate-icon">
                            <IconPackage class="icon-svg" />
                        </div>
                        <h2 class="registry-gate-title">"Enable DixScript Registry"</h2>
                        <p class="registry-gate-desc">
                            "You need to enable the DixScript Registry service before submitting packages. "
                            "Head to the Services tab to activate it."
                        </p>
                        <p class="registry-gate-note">"It's free."</p>
                    </div>
                }.into_any()
            } else {
                view! {
                    <div class="registry-content">

                        // ── Submit card ────────────────────────
                        <div class="registry-card">
                            <div class="registry-card-head">
                                <div>
                                    <h2 class="registry-card-title">"Submit a Package"</h2>
                                    <p class="registry-card-desc">
                                        "Packages are reviewed before becoming publicly available. "
                                        "Upload a .mdix file or write one directly."
                                    </p>
                                </div>
                                {move || if !show_form.get() {
                                    view! {
                                        <button
                                            class="btn btn-primary btn-sm"
                                            on:click=move |_| {
                                                set_show_form.set(true);
                                                set_submit_error.set(String::new());
                                                set_submit_tab.set(SubmitTab::Upload);
                                            }
                                        >
                                            <IconUpload class="icon-svg icon-xs" />
                                            "New Submission"
                                        </button>
                                    }.into_any()
                                } else {
                                    view! { <span></span> }.into_any()
                                }}
                            </div>

                            // ── Form area ──────────────────────────
                            // Inline on:submit handler avoids move/clone issues
                            // with reactive closures re-executing.
                            {move || if show_form.get() {
                                view! {
                                    <div class="registry-form-wrap">

                                        // Tab switcher
                                        <div class="submit-tabs">
                                            <button
                                                type="button"
                                                class=move || if submit_tab.get() == SubmitTab::Upload {
                                                    "submit-tab submit-tab--active"
                                                } else { "submit-tab" }
                                                on:click=move |_| {
                                                    set_submit_tab.set(SubmitTab::Upload);
                                                    set_submit_error.set(String::new());
                                                }
                                            >
                                                <IconUpload class="icon-svg icon-xs" />
                                                "Upload File"
                                            </button>
                                            <button
                                                type="button"
                                                class=move || if submit_tab.get() == SubmitTab::Write {
                                                    "submit-tab submit-tab--active"
                                                } else { "submit-tab" }
                                                on:click=move |_| {
                                                    set_submit_tab.set(SubmitTab::Write);
                                                    set_submit_error.set(String::new());
                                                }
                                            >
                                                <IconEdit class="icon-svg icon-xs" />
                                                "Write File"
                                            </button>
                                        </div>

                                        // Single form — inline handler reads submit_tab
                                        // to decide which path to take. Both sections are
                                        // always in the DOM; CSS show/hide toggles them.
                                        <form
                                            class="registry-form"
                                            on:submit=move |ev: web_sys::SubmitEvent| {
                                                ev.prevent_default();

                                                if submit_tab.get() == SubmitTab::Upload {
                                                    // ── Upload path ──────────────────────
                                                    let file = match f_file.get() {
                                                        Some(f) => f,
                                                        None => {
                                                            set_submit_error.set(
                                                                "Please select a .mdix file.".to_string()
                                                            );
                                                            return;
                                                        }
                                                    };
                                                    let filename    = f_filename.get();
                                                    let category    = f_category.get();
                                                    let version     = f_version.get();
                                                    let description = f_description.get();
                                                    let tags_raw    = f_tags.get();

                                                    if version.trim().is_empty() || description.trim().is_empty() {
                                                        set_submit_error.set(
                                                            "Version and description are required.".to_string()
                                                        );
                                                        return;
                                                    }
                                                    if !filename.ends_with(".mdix") {
                                                        set_submit_error.set(
                                                            "Only .mdix files are accepted.".to_string()
                                                        );
                                                        return;
                                                    }

                                                    let tags: Vec<String> = tags_raw.split(',')
                                                        .map(|t| t.trim().to_string())
                                                        .filter(|t| !t.is_empty())
                                                        .collect();

                                                    let user_id = gloo_storage::LocalStorage::get::<String>("mms_user_id")
                                                        .unwrap_or_default();
                                                    let mid_id  = profile.get()
                                                        .map(|p| p.mid_id.clone())
                                                        .unwrap_or_default();

                                                    if user_id.is_empty() || mid_id.is_empty() {
                                                        set_submit_error.set(
                                                            "Profile not loaded. Please refresh.".to_string()
                                                        );
                                                        return;
                                                    }

                                                    set_submitting.set(true);
                                                    set_submit_error.set(String::new());

                                                    spawn_local(async move {
                                                        let path_id = match generate_path_id().await {
                                                            Ok(id)  => id,
                                                            Err(e)  => {
                                                                set_submit_error.set(e);
                                                                set_submitting.set(false);
                                                                return;
                                                            }
                                                        };
                                                        let storage_path = format!("{}/{}.mdix", mid_id, path_id);

                                                        let file_bytes = {
                                                            let blob: &web_sys::Blob = file.as_ref();
                                                            let promise = blob.array_buffer();
                                                            match JsFuture::from(promise).await {
                                                                Ok(buf) => Uint8Array::new(&buf).to_vec(),
                                                                Err(_)  => {
                                                                    set_submit_error.set("Failed to read file.".to_string());
                                                                    set_submitting.set(false);
                                                                    return;
                                                                }
                                                            }
                                                        };

                                                        let client = SupabaseClient::new();
                                                        if let Err(e) = client.upload_registry_file(
                                                            "registry-pending", &storage_path, file_bytes,
                                                        ).await {
                                                            set_submit_error.set(e);
                                                            set_submitting.set(false);
                                                            return;
                                                        }

                                                        match client.create_registry_submission(
                                                            &user_id, &mid_id, &filename, &category,
                                                            &description, tags, &version,
                                                            Some(format!("registry-pending/{}", storage_path)),
                                                        ).await {
                                                            Ok(sub) => {
                                                                set_submissions.update(|s| s.insert(0, sub));
                                                                set_show_form.set(false);
                                                                set_f_version.set(String::new());
                                                                set_f_description.set(String::new());
                                                                set_f_tags.set(String::new());
                                                                set_f_file.set(None);
                                                                set_f_filename.set(String::new());
                                                                set_submitting.set(false);
                                                            }
                                                            Err(e) => {
                                                                set_submit_error.set(e);
                                                                set_submitting.set(false);
                                                            }
                                                        }
                                                    });

                                                } else {
                                                    // ── Write path ───────────────────────
                                                    let name        = w_name.get();
                                                    let category    = w_category.get();
                                                    let version     = w_version.get();
                                                    let description = w_description.get();
                                                    let tags_raw    = w_tags.get();
                                                    let content     = w_content.get();

                                                    if name.trim().is_empty() {
                                                        set_submit_error.set(
                                                            "Please enter a package name.".to_string()
                                                        );
                                                        return;
                                                    }
                                                    if version.trim().is_empty() || description.trim().is_empty() {
                                                        set_submit_error.set(
                                                            "Version and description are required.".to_string()
                                                        );
                                                        return;
                                                    }
                                                    if content.trim().is_empty() {
                                                        set_submit_error.set(
                                                            "File content cannot be empty.".to_string()
                                                        );
                                                        return;
                                                    }

                                                    let sections = [
                                                        "@DATA", "@CONFIG", "@QUICKFUNCS",
                                                        "@ENUMS", "@DLM", "@IMPORTS", "@SECURITY",
                                                    ];
                                                    let content_upper = content.to_uppercase();
                                                    if !sections.iter().any(|s| content_upper.contains(s)) {
                                                        set_submit_error.set(
                                                            "Must contain at least one DixScript section \
                                                             (@DATA, @CONFIG, @QUICKFUNCS, etc.).".to_string()
                                                        );
                                                        return;
                                                    }

                                                    let filename = {
                                                        let n = name.trim();
                                                        if n.ends_with(".mdix") {
                                                            n.to_string()
                                                        } else {
                                                            format!("{}.mdix", n)
                                                        }
                                                    };

                                                    let tags: Vec<String> = tags_raw.split(',')
                                                        .map(|t| t.trim().to_string())
                                                        .filter(|t| !t.is_empty())
                                                        .collect();

                                                    let user_id = gloo_storage::LocalStorage::get::<String>("mms_user_id")
                                                        .unwrap_or_default();
                                                    let mid_id  = profile.get()
                                                        .map(|p| p.mid_id.clone())
                                                        .unwrap_or_default();

                                                    if user_id.is_empty() || mid_id.is_empty() {
                                                        set_submit_error.set(
                                                            "Profile not loaded. Please refresh.".to_string()
                                                        );
                                                        return;
                                                    }

                                                    set_submitting.set(true);
                                                    set_submit_error.set(String::new());

                                                    let file_bytes = content.into_bytes();

                                                    spawn_local(async move {
                                                        let path_id = match generate_path_id().await {
                                                            Ok(id)  => id,
                                                            Err(e)  => {
                                                                set_submit_error.set(e);
                                                                set_submitting.set(false);
                                                                return;
                                                            }
                                                        };
                                                        let storage_path = format!("{}/{}.mdix", mid_id, path_id);

                                                        let client = SupabaseClient::new();
                                                        if let Err(e) = client.upload_registry_file(
                                                            "registry-pending", &storage_path, file_bytes,
                                                        ).await {
                                                            set_submit_error.set(e);
                                                            set_submitting.set(false);
                                                            return;
                                                        }

                                                        match client.create_registry_submission(
                                                            &user_id, &mid_id, &filename, &category,
                                                            &description, tags, &version,
                                                            Some(format!("registry-pending/{}", storage_path)),
                                                        ).await {
                                                            Ok(sub) => {
                                                                set_submissions.update(|s| s.insert(0, sub));
                                                                set_show_form.set(false);
                                                                set_w_name.set(String::new());
                                                                set_w_version.set(String::new());
                                                                set_w_description.set(String::new());
                                                                set_w_tags.set(String::new());
                                                                set_w_content.set(String::new());
                                                                set_submitting.set(false);
                                                            }
                                                            Err(e) => {
                                                                set_submit_error.set(e);
                                                                set_submitting.set(false);
                                                            }
                                                        }
                                                    });
                                                }
                                            }
                                        >
                                            // ── Upload section ─────────────────────────
                                            <div class=move || if submit_tab.get() == SubmitTab::Upload {
                                                "tab-section"
                                            } else {
                                                "tab-section tab-section--hidden"
                                            }>
                                                <div class="form-group">
                                                    <label class="form-label">
                                                        "Package File "
                                                        <span class="form-label-required">"*"</span>
                                                    </label>
                                                    <label class="file-drop-zone">
                                                        <input
                                                            type="file"
                                                            accept=".mdix"
                                                            class="file-input-hidden"
                                                            on:change=handle_file_change
                                                        />
                                                        <div class="file-drop-inner">
                                                            <IconUpload class="icon-svg file-drop-icon" />
                                                            {move || if f_filename.get().is_empty() {
                                                                view! {
                                                                    <span class="file-drop-text">
                                                                        "Click to select a .mdix file"
                                                                    </span>
                                                                }.into_any()
                                                            } else {
                                                                view! {
                                                                    <span class="file-drop-selected">
                                                                        {f_filename.get()}
                                                                    </span>
                                                                }.into_any()
                                                            }}
                                                        </div>
                                                    </label>
                                                </div>

                                                <div class="registry-form-row">
                                                    <div class="form-group">
                                                        <label class="form-label">"Category"</label>
                                                        <select
                                                            class="form-input"
                                                            on:change=move |ev| {
                                                                if let Some(sel) = ev.target()
                                                                    .and_then(|t| t.dyn_into::<web_sys::HtmlSelectElement>().ok())
                                                                {
                                                                    set_f_category.set(sel.value());
                                                                }
                                                            }
                                                        >
                                                            {CATEGORIES.iter().map(|(val, label)| {
                                                                view! { <option value=*val>{*label}</option> }
                                                            }).collect_view()}
                                                        </select>
                                                    </div>
                                                    <div class="form-group">
                                                        <label class="form-label">"Version *"</label>
                                                        <input
                                                            class="form-input"
                                                            type="text"
                                                            placeholder="1.0.0"
                                                            prop:value=move || f_version.get()
                                                            on:input=move |ev| set_f_version.set(event_target_value(&ev))
                                                        />
                                                    </div>
                                                </div>

                                                <div class="form-group">
                                                    <label class="form-label">"Description *"</label>
                                                    <textarea
                                                        class="form-textarea"
                                                        placeholder="What does this package do?"
                                                        rows="3"
                                                        prop:value=move || f_description.get()
                                                        on:input=move |ev| set_f_description.set(event_target_value(&ev))
                                                    ></textarea>
                                                </div>

                                                <div class="form-group">
                                                    <label class="form-label">
                                                        "Tags "
                                                        <span class="form-label-optional">"(comma-separated)"</span>
                                                    </label>
                                                    <input
                                                        class="form-input"
                                                        type="text"
                                                        placeholder="ui, animation, physics"
                                                        prop:value=move || f_tags.get()
                                                        on:input=move |ev| set_f_tags.set(event_target_value(&ev))
                                                    />
                                                </div>
                                            </div>

                                            // ── Write section ──────────────────────────
                                            <div class=move || if submit_tab.get() == SubmitTab::Write {
                                                "tab-section"
                                            } else {
                                                "tab-section tab-section--hidden"
                                            }>
                                                <div class="registry-form-row">
                                                    <div class="form-group">
                                                        <label class="form-label">"Package Name *"</label>
                                                        <input
                                                            class="form-input"
                                                            type="text"
                                                            placeholder="my_package"
                                                            prop:value=move || w_name.get()
                                                            on:input=move |ev| set_w_name.set(event_target_value(&ev))
                                                        />
                                                        <span class="form-hint">
                                                            ".mdix extension added automatically"
                                                        </span>
                                                    </div>
                                                    <div class="form-group">
                                                        <label class="form-label">"Category"</label>
                                                        <select
                                                            class="form-input"
                                                            on:change=move |ev| {
                                                                if let Some(sel) = ev.target()
                                                                    .and_then(|t| t.dyn_into::<web_sys::HtmlSelectElement>().ok())
                                                                {
                                                                    set_w_category.set(sel.value());
                                                                }
                                                            }
                                                        >
                                                            {CATEGORIES.iter().map(|(val, label)| {
                                                                view! { <option value=*val>{*label}</option> }
                                                            }).collect_view()}
                                                        </select>
                                                    </div>
                                                </div>

                                                <div class="registry-form-row">
                                                    <div class="form-group">
                                                        <label class="form-label">"Version *"</label>
                                                        <input
                                                            class="form-input"
                                                            type="text"
                                                            placeholder="1.0.0"
                                                            prop:value=move || w_version.get()
                                                            on:input=move |ev| set_w_version.set(event_target_value(&ev))
                                                        />
                                                    </div>
                                                    <div class="form-group">
                                                        <label class="form-label">
                                                            "Tags "
                                                            <span class="form-label-optional">"(comma-separated)"</span>
                                                        </label>
                                                        <input
                                                            class="form-input"
                                                            type="text"
                                                            placeholder="rpg, enums, config"
                                                            prop:value=move || w_tags.get()
                                                            on:input=move |ev| set_w_tags.set(event_target_value(&ev))
                                                        />
                                                    </div>
                                                </div>

                                                <div class="form-group">
                                                    <label class="form-label">"Description *"</label>
                                                    <textarea
                                                        class="form-textarea"
                                                        placeholder="What does this package provide?"
                                                        rows="2"
                                                        prop:value=move || w_description.get()
                                                        on:input=move |ev| set_w_description.set(event_target_value(&ev))
                                                    ></textarea>
                                                </div>

                                                <div class="form-group">
                                                    <label class="form-label">
                                                        "File Content "
                                                        <span class="form-label-required">"*"</span>
                                                    </label>
                                                    <textarea
                                                        class="form-textarea mdix-editor"
                                                        rows="18"
                                                        placeholder="@CONFIG(
  version -> \"1.0.0\"
  features -> \"advanced\"
)

@DATA(
  // your config here
)"
                                                        prop:value=move || w_content.get()
                                                        on:input=move |ev| set_w_content.set(event_target_value(&ev))
                                                    ></textarea>
                                                    <span class="form-hint">
                                                        "Must contain at least one @DATA, @CONFIG, @QUICKFUNCS, @ENUMS, or @DLM section."
                                                    </span>
                                                </div>
                                            </div>

                                            // ── Shared error + actions ─────────────────
                                            {move || if !submit_error.get().is_empty() {
                                                view! {
                                                    <div class="status-msg status-msg--error">
                                                        {submit_error.get()}
                                                    </div>
                                                }.into_any()
                                            } else { view! { <span></span> }.into_any() }}

                                            <div class="registry-form-actions">
                                                <button
                                                    type="button"
                                                    class="btn btn-ghost btn-sm"
                                                    disabled=move || submitting.get()
                                                    on:click=move |_| {
                                                        set_show_form.set(false);
                                                        set_submit_error.set(String::new());
                                                    }
                                                >
                                                    "Cancel"
                                                </button>
                                                <button
                                                    type="submit"
                                                    class="btn btn-primary btn-sm"
                                                    disabled=move || submitting.get()
                                                >
                                                    {move || if submitting.get() {
                                                        view! {
                                                            <IconLoader class="icon-svg spin" />
                                                            <span>"Submitting..."</span>
                                                        }.into_any()
                                                    } else if submit_tab.get() == SubmitTab::Upload {
                                                        view! {
                                                            <IconUpload class="icon-svg icon-xs" />
                                                            <span>"Submit Package"</span>
                                                        }.into_any()
                                                    } else {
                                                        view! {
                                                            <IconEdit class="icon-svg icon-xs" />
                                                            <span>"Submit Package"</span>
                                                        }.into_any()
                                                    }}
                                                </button>
                                            </div>

                                        </form>
                                    </div>
                                }.into_any()
                            } else { view! { <span></span> }.into_any() }}
                        </div>

                        // ── Submissions list card ──────────────────────────────
                        <div class="registry-card">
                            <h2 class="registry-card-title">"My Submissions"</h2>

                            {move || if loading.get() {
                                view! {
                                    <div class="registry-loading"><div class="spinner"></div></div>
                                }.into_any()
                            } else if submissions.get().is_empty() {
                                view! {
                                    <div class="registry-empty">
                                        <IconFileText class="icon-svg registry-empty-icon" />
                                        <p class="registry-empty-title">"No submissions yet"</p>
                                        <p class="registry-empty-sub">
                                            "Submit your first .mdix package to get it listed in the registry."
                                        </p>
                                    </div>
                                }.into_any()
                            } else {
                                view! {
                                    <div class="registry-list">
                                        {move || submissions.get().into_iter().map(|sub| {
                                            view! { <SubmissionRow submission=sub /> }
                                        }).collect_view()}
                                    </div>
                                }.into_any()
                            }}
                        </div>

                    </div>
                }.into_any()
            }}

        </div>
    }
}

// ── Submission row ─────────────────────────────────────────────

#[component]
fn SubmissionRow(submission: RegistrySubmission) -> impl IntoView {
    let status_class = match submission.status.as_str() {
        "approved" => "sub-status sub-status--approved",
        "rejected" => "sub-status sub-status--rejected",
        _          => "sub-status sub-status--pending",
    };

    let status_icon: AnyView = match submission.status.as_str() {
        "approved" => view! { <IconCheck         class="icon-svg icon-xs" /> }.into_any(),
        "rejected" => view! { <IconAlertTriangle class="icon-svg icon-xs" /> }.into_any(),
        _          => view! { <IconClock         class="icon-svg icon-xs" /> }.into_any(),
    };

    let admin_note   = submission.admin_note.clone();
    let status_label = submission.status_label().to_string();
    let tags_display = submission.tags.join(", ");

    view! {
        <div class="sub-row">
            <div class="sub-row-main">
                <div class="sub-row-icon">
                    <IconPackage class="icon-svg icon-xs" />
                </div>
                <div class="sub-row-info">
                    <div class="sub-row-top">
                        <span class="sub-filename">{submission.filename.clone()}</span>
                        <span class=status_class>
                            {status_icon}
                            {status_label}
                        </span>
                    </div>
                    <div class="sub-row-meta">
                        <span class="sub-meta-item">
                            <span class="sub-meta-label">"Category"</span>
                            <span class="sub-meta-value">{submission.category.clone()}</span>
                        </span>
                        <span class="sub-meta-item">
                            <span class="sub-meta-label">"Version"</span>
                            <code class="sub-meta-value">{submission.version.clone()}</code>
                        </span>
                        <span class="sub-meta-item">
                            <span class="sub-meta-label">"Submitted"</span>
                            <span class="sub-meta-value">{submission.formatted_submitted()}</span>
                        </span>
                    </div>
                    {if !tags_display.is_empty() {
                        view! {
                            <div class="sub-tags">
                                {submission.tags.iter().map(|t| view! {
                                    <span class="sub-tag">{t.clone()}</span>
                                }).collect_view()}
                            </div>
                        }.into_any()
                    } else {
                        view! { <span></span> }.into_any()
                    }}
                    {if let Some(note) = admin_note {
                        if !note.is_empty() {
                            view! {
                                <div class="sub-admin-note">
                                    <span class="sub-admin-note-label">"Review note:"</span>
                                    <span class="sub-admin-note-text">{note}</span>
                                </div>
                            }.into_any()
                        } else {
                            view! { <span></span> }.into_any()
                        }
                    } else {
                        view! { <span></span> }.into_any()
                    }}
                </div>
            </div>
        </div>
    }
}
