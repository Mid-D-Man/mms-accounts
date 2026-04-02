use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;
use gloo_storage::Storage;
use js_sys::Uint8Array;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use crate::supabase::{SupabaseClient, Profile, RegistrySubmission, generate_path_id};
use crate::components::icons::{
    IconPackage, IconUpload, IconFileText, IconClock, IconCheck, IconAlertTriangle, IconLoader,
};

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

    // Form state
    let (f_category,    set_f_category)    = signal("script".to_string());
    let (f_version,     set_f_version)     = signal(String::new());
    let (f_description, set_f_description) = signal(String::new());
    let (f_tags,        set_f_tags)        = signal(String::new());
    let (f_file,        set_f_file)        = signal(Option::<web_sys::File>::None);
    let (f_filename,    set_f_filename)    = signal(String::new());

    // Check if user is subscribed to the registry service
    let (is_subscribed, set_is_subscribed) = signal(false);
    let (checking_sub,  set_checking_sub)  = signal(true);

    // Load data
    Effect::new(move |_| {
        let user_id = gloo_storage::LocalStorage::get::<String>("mms_user_id")
            .unwrap_or_default();
        let client = SupabaseClient::new();

        spawn_local(async move {
            // Check subscription
            if let Ok(subs) = client.get_user_subscriptions(&user_id).await {
                let subscribed = subs.iter().any(|s| {
                    s.status == "active"
                    // We check by looking at service slugs from a parallel fetch,
                    // but for now we'll check if they have ANY active service.
                    // A more complete impl would cross-reference the service slug.
                    // For now, we optimistically allow — RLS will gate the actual upload.
                });
                // Ideally you'd cross-reference service_id → slug.
                // As a practical shortcut: fetch services and match.
                let _ = subscribed; // suppress unused

                if let Ok(services) = client.list_services().await {
                    let registry_service = services.iter()
                        .find(|s| s.slug == "dixscript-registry");
                    if let Some(svc) = registry_service {
                        let active = subs.iter().any(|s| {
                            s.service_id == svc.id && s.is_active()
                        });
                        set_is_subscribed.set(active);
                    }
                }
            }
            set_checking_sub.set(false);

            // Load submissions
            match client.list_user_submissions(&user_id).await {
                Ok(s) => {
                    set_submissions.set(s);
                    set_loading.set(false);
                }
                Err(e) => {
                    set_error.set(e);
                    set_loading.set(false);
                }
            }
        });
    });

    // File input change handler
    let handle_file_change = move |ev: web_sys::Event| {
        let input = ev.target()
            .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok());
        if let Some(input) = input {
            if let Some(files) = input.files() {
                if let Some(file) = files.get(0) {
                    let name = file.name();
                    set_f_filename.set(name);
                    set_f_file.set(Some(file));
                }
            }
        }
    };

    // Submit handler
    let handle_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();

        let file = match f_file.get() {
            Some(f) => f,
            None => {
                set_submit_error.set("Please select a .mdix file.".to_string());
                return;
            }
        };

        let filename    = f_filename.get();
        let category    = f_category.get();
        let version     = f_version.get();
        let description = f_description.get();
        let tags_raw    = f_tags.get();

        if version.trim().is_empty() || description.trim().is_empty() {
            set_submit_error.set("Version and description are required.".to_string());
            return;
        }

        if !filename.ends_with(".mdix") {
            set_submit_error.set("Only .mdix files are accepted.".to_string());
            return;
        }

        let tags: Vec<String> = tags_raw
            .split(',')
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty())
            .collect();

        let user_id = gloo_storage::LocalStorage::get::<String>("mms_user_id")
            .unwrap_or_default();
        let mid_id  = profile.get().map(|p| p.mid_id.clone()).unwrap_or_default();

        if user_id.is_empty() || mid_id.is_empty() {
            set_submit_error.set("Profile not loaded. Please refresh.".to_string());
            return;
        }

        set_submitting.set(true);
        set_submit_error.set(String::new());

        let client = SupabaseClient::new();

        spawn_local(async move {
            // 1. Generate path ID
            let path_id = match generate_path_id().await {
                Ok(id)  => id,
                Err(e)  => { set_submit_error.set(e); set_submitting.set(false); return; }
            };

            let storage_path = format!("{}/{}.mdix", mid_id, path_id);

            // 2. Read file bytes via arrayBuffer()
            let file_bytes = {
                use wasm_bindgen::JsCast;
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

            // 3. Upload to Supabase Storage
            if let Err(e) = client.upload_registry_file(
                "registry-pending", &storage_path, file_bytes,
            ).await {
                set_submit_error.set(e);
                set_submitting.set(false);
                return;
            }

            // 4. Create database record
            match client.create_registry_submission(
                &user_id, &mid_id, &filename, &category,
                &description, tags, &version,
                Some(format!("registry-pending/{}", storage_path)),
            ).await {
                Ok(submission) => {
                    set_submissions.update(|s| s.insert(0, submission));
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
    };

    view! {
        <div class="registry-view">

            // ── Header ────────────────────────────────────────
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
                    <div class="registry-loading">
                        <div class="spinner"></div>
                    </div>
                }.into_any()
            } else if !is_subscribed.get() {
                // Not subscribed — prompt to enable service
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
                // Subscribed — show submit button + submissions
                view! {
                    <div class="registry-content">

                        // ── Submit section ─────────────────────
                        <div class="registry-card">
                            <div class="registry-card-head">
                                <div>
                                    <h2 class="registry-card-title">"Submit a Package"</h2>
                                    <p class="registry-card-desc">
                                        "Packages are reviewed before becoming publicly available."
                                    </p>
                                </div>
                                {move || if !show_form.get() {
                                    view! {
                                        <button
                                            class="btn btn-primary btn-sm"
                                            on:click=move |_| {
                                                set_show_form.set(true);
                                                set_submit_error.set(String::new());
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

                            {move || if show_form.get() {
                                view! {
                                    <div class="registry-form-wrap">
                                        <form class="registry-form" on:submit=handle_submit>

                                            // File picker
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

                                            // Category + Version row
                                            <div class="registry-form-row">
                                                <div class="form-group">
                                                    <label class="form-label">"Category"</label>
                                                    <select
                                                        class="form-input"
                                                        on:change=move |ev| {
                                                            use wasm_bindgen::JsCast;
                                                            if let Some(sel) = ev.target()
                                                                .and_then(|t| t.dyn_into::<web_sys::HtmlSelectElement>().ok())
                                                            {
                                                                set_f_category.set(sel.value());
                                                            }
                                                        }
                                                    >
                                                        {CATEGORIES.iter().map(|(val, label)| {
                                                            view! {
                                                                <option value=*val>{*label}</option>
                                                            }
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
                                                        required
                                                    />
                                                </div>
                                            </div>

                                            // Description
                                            <div class="form-group">
                                                <label class="form-label">"Description *"</label>
                                                <textarea
                                                    class="form-textarea"
                                                    placeholder="What does this package do?"
                                                    rows="3"
                                                    prop:value=move || f_description.get()
                                                    on:input=move |ev| set_f_description.set(event_target_value(&ev))
                                                    required
                                                ></textarea>
                                            </div>

                                            // Tags
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
                                                    } else {
                                                        view! {
                                                            <IconUpload class="icon-svg icon-xs" />
                                                            <span>"Submit Package"</span>
                                                        }.into_any()
                                                    }}
                                                </button>
                                            </div>
                                        </form>
                                    </div>
                                }.into_any()
                            } else {
                                view! { <span></span> }.into_any()
                            }}
                        </div>

                        // ── Submissions list ───────────────────
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

    let tags_display = submission.tags.join(", ");
    let admin_note   = submission.admin_note.clone();
    let status_label = submission.status_label().to_string();

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
