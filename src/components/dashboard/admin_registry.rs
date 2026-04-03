// src/components/dashboard/admin_registry.rs
// Registry submission review — admin-only.
//
// Flow:
//   1. Load all pending RegistrySubmissions (admin RLS policy required).
//   2. Each row can be expanded to see description, tags, and file preview.
//   3. Approve → calls dixscript-docs /api/registry/approve (via SupabaseClient).
//      dixscript-docs: verifies JWT + admin role, moves file from Supabase Storage
//      to R2 packages/, creates .meta.json, deletes from Supabase Storage, marks
//      submission approved.
//   4. Reject → inline note form → calls dixscript-docs /api/registry/reject.
//   5. On success, the submission is removed from the local list.
//
// Note: the admin can ALSO submit packages via the normal RegistryView
// (user-facing). This view is exclusively for reviewing OTHER users' submissions.

use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;
use std::sync::Arc;
use crate::supabase::{SupabaseClient, Profile, RegistrySubmission};
use crate::components::icons::{
    IconPackage, IconCheck, IconX, IconClock, IconLoader,
    IconAlertTriangle, IconFileText, IconEye,
};

#[component]
pub fn AdminRegistryView(profile: ReadSignal<Option<Profile>>) -> impl IntoView {
    let is_admin = move || profile.get().as_ref().map(|p| p.is_admin()).unwrap_or(false);

    // ── Data ──────────────────────────────────────────────────
    let (submissions,   set_submissions)   = signal(Vec::<RegistrySubmission>::new());
    let (loading,       set_loading)       = signal(true);
    let (error,         set_error)         = signal(String::new());

    // ── Row UI state (one active at a time) ───────────────────
    let (expanded_id,     set_expanded_id)     = signal(None::<String>);
    let (preview_id,      set_preview_id)      = signal(None::<String>);
    let (preview_content, set_preview_content) = signal(String::new());
    let (preview_loading, set_preview_loading) = signal(false);
    let (reject_id,       set_reject_id)       = signal(None::<String>);
    let (reject_note,     set_reject_note)      = signal(String::new());

    // ── Async operation state ─────────────────────────────────
    let (processing_id,  set_processing_id)  = signal(None::<String>);
    let (action_error,   set_action_error)   = signal(String::new());
    let (action_success, set_action_success) = signal(String::new());

    // ── Load pending submissions ───────────────────────────────
    Effect::new(move |_| {
        if !is_admin() { set_loading.set(false); return; }
        let client = SupabaseClient::new();
        spawn_local(async move {
            match client.get_pending_submissions().await {
                Ok(subs) => { set_submissions.set(subs); set_loading.set(false); }
                Err(e)   => { set_error.set(e);          set_loading.set(false); }
            }
        });
    });

    // ── Approve ────────────────────────────────────────────────
    // Closures capture only Copy WriteSignals, so they are Fn + Copy + Clone.
    let handle_approve = move |sub_id: String| {
        if processing_id.get().is_some() { return; }
        set_processing_id.set(Some(sub_id.clone()));
        set_action_error.set(String::new());
        set_action_success.set(String::new());

        let client = SupabaseClient::new();
        spawn_local(async move {
            match client.approve_submission(&sub_id).await {
                Ok(()) => {
                    set_submissions.update(|s| s.retain(|sub| sub.id != sub_id));
                    set_expanded_id.set(None);
                    set_preview_id.set(None);
                    set_preview_content.set(String::new());
                    set_processing_id.set(None);
                    set_action_success.set(
                        "Approved — package is now live in the registry.".to_string()
                    );
                }
                Err(e) => {
                    set_action_error.set(e);
                    set_processing_id.set(None);
                }
            }
        });
    };

    // ── Reject ─────────────────────────────────────────────────
    let handle_reject = move |sub_id: String, note: String| {
        if processing_id.get().is_some() { return; }
        set_processing_id.set(Some(sub_id.clone()));
        set_action_error.set(String::new());
        set_action_success.set(String::new());

        let client = SupabaseClient::new();
        spawn_local(async move {
            match client.reject_submission(&sub_id, &note).await {
                Ok(()) => {
                    set_submissions.update(|s| s.retain(|sub| sub.id != sub_id));
                    set_reject_id.set(None);
                    set_reject_note.set(String::new());
                    set_expanded_id.set(None);
                    set_processing_id.set(None);
                    set_action_success.set("Submission rejected.".to_string());
                }
                Err(e) => {
                    set_action_error.set(e);
                    set_processing_id.set(None);
                }
            }
        });
    };

    // ── Preview (toggle) ───────────────────────────────────────
    let handle_preview = move |sub_id: String, storage_path: Option<String>| {
        // Toggle off
        if preview_id.get().as_deref() == Some(sub_id.as_str()) {
            set_preview_id.set(None);
            set_preview_content.set(String::new());
            return;
        }

        let path = match storage_path.filter(|p| !p.is_empty()) {
            Some(p) => p,
            None    => {
                set_action_error.set("No file stored for this submission.".to_string());
                return;
            }
        };

        set_preview_id.set(Some(sub_id));
        set_preview_content.set(String::new());
        set_preview_loading.set(true);

        let client = SupabaseClient::new();
        spawn_local(async move {
            match client.download_submission_file(&path).await {
                Ok(text) => {
                    set_preview_content.set(text);
                    set_preview_loading.set(false);
                }
                Err(e) => {
                    set_preview_content.set(format!("// Error loading file:\n// {}", e));
                    set_preview_loading.set(false);
                }
            }
        });
    };

    view! {
        <div class="admin-registry-view">

            {move || if !is_admin() {
                view! {
                    <div class="admin-gate">
                        <IconPackage class="icon-svg admin-gate-icon" />
                        <p class="admin-gate-text">"Admin access required."</p>
                    </div>
                }.into_any()
            } else {
                view! {
                    <div class="admin-registry-content">

                        // ── Header ─────────────────────────────
                        <div class="admin-section-header">
                            <div class="admin-section-header-icon">
                                <IconPackage class="icon-svg" />
                            </div>
                            <div>
                                <h1 class="admin-section-title">"Registry Review"</h1>
                                <p class="admin-section-subtitle">
                                    "Review pending package submissions. Approved packages are "
                                    "published to the public DixScript registry on Cloudflare R2. "
                                    "Submit your own packages via the Registry tab."
                                </p>
                            </div>
                        </div>

                        // ── Feedback banners ────────────────────
                        {move || if !action_error.get().is_empty() {
                            view! {
                                <div class="status-msg status-msg--error">
                                    <IconAlertTriangle class="icon-svg icon-sm" />
                                    {action_error.get()}
                                </div>
                            }.into_any()
                        } else { view! { <span></span> }.into_any() }}

                        {move || if !action_success.get().is_empty() {
                            view! {
                                <div class="status-msg status-msg--success">
                                    <IconCheck class="icon-svg icon-sm" />
                                    {action_success.get()}
                                </div>
                            }.into_any()
                        } else { view! { <span></span> }.into_any() }}

                        {move || if !error.get().is_empty() {
                            view! {
                                <div class="status-msg status-msg--error">{error.get()}</div>
                            }.into_any()
                        } else { view! { <span></span> }.into_any() }}

                        // ── Main content ────────────────────────
                        {move || if loading.get() {
                            view! {
                                <div class="areg-loading"><div class="spinner"></div></div>
                            }.into_any()
                        } else if submissions.get().is_empty() {
                            view! {
                                <div class="areg-empty">
                                    <IconFileText class="icon-svg areg-empty-icon" />
                                    <p class="areg-empty-title">"No pending submissions"</p>
                                    <p class="areg-empty-sub">
                                        "All submissions have been reviewed. Check back later."
                                    </p>
                                </div>
                            }.into_any()
                        } else {
                            let count = submissions.get().len();
                            view! {
                                <div class="areg-list-wrap">
                                    <div class="areg-count-bar">
                                        <span class="areg-count-badge">{count.to_string()}</span>
                                        " pending "
                                        {if count == 1 { "submission" } else { "submissions" }}
                                        " awaiting review"
                                    </div>
                                    <div class="areg-list">
                                        {move || {
                                            // handle_approve / handle_reject / handle_preview
                                            // capture only Copy signals → they ARE Copy + Clone.
                                            // Arc wraps them for cheap sharing across iteration items.
                                            let ha = Arc::new(handle_approve);
                                            let hr = Arc::new(handle_reject);
                                            let hp = Arc::new(handle_preview);

                                            submissions.get().into_iter().map(|sub| {
                                                let ha = ha.clone();
                                                let hr = hr.clone();
                                                let hp = hp.clone();

                                                // Arc<String> allows Fn (not FnOnce) click handlers
                                                let sid            = Arc::new(sub.id.clone());
                                                let sid_approve    = sid.clone();
                                                let sid_expand     = sid.clone();
                                                let sid_reject_tog = sid.clone();
                                                let sid_reject_cfm = sid.clone();
                                                let sid_preview    = sid.clone();
                                                let sp             = Arc::new(sub.supabase_storage_path.clone());

                                                // Reactive per-row state derived from parent signals
                                                let is_exp  = { let s = sid.clone(); move || expanded_id.get().as_deref()    == Some(s.as_str()) };
                                                let is_prev = { let s = sid.clone(); move || preview_id.get().as_deref()     == Some(s.as_str()) };
                                                let is_rej  = { let s = sid.clone(); move || reject_id.get().as_deref()      == Some(s.as_str()) };
                                                let is_proc = { let s = sid.clone(); move || processing_id.get().as_deref()  == Some(s.as_str()) };

                                                // Static display values (owned — no reactive needed)
                                                let filename    = sub.filename.clone();
                                                let category    = sub.category.clone();
                                                let version     = sub.version.clone();
                                                let mid_id      = sub.mid_id.clone();
                                                let description = sub.description.clone();
                                                let tags_str    = sub.tags.join(", ");
                                                let submitted   = sub.formatted_submitted();

                                                view! {
                                                    <div class=move || {
                                                        if is_exp() { "areg-row areg-row--expanded" }
                                                        else        { "areg-row" }
                                                    }>
                                                        // ── Row header ──────────────────
                                                        <div class="areg-row-main">
                                                            <div class="areg-row-icon">
                                                                <IconPackage class="icon-svg icon-xs" />
                                                            </div>

                                                            <div class="areg-row-info">
                                                                <div class="areg-row-top">
                                                                    <span class="areg-filename">{filename}</span>
                                                                    <span class="areg-badge areg-badge--cat">{category}</span>
                                                                    <span class="areg-badge">"v"{version}</span>
                                                                </div>
                                                                <div class="areg-row-meta">
                                                                    <span class="areg-meta-item">
                                                                        <span class="areg-meta-label">"MID ID"</span>
                                                                        <code class="areg-meta-value">{mid_id}</code>
                                                                    </span>
                                                                    <span class="areg-meta-item">
                                                                        <span class="areg-meta-label">"Submitted"</span>
                                                                        <span class="areg-meta-value">{submitted}</span>
                                                                    </span>
                                                                </div>
                                                            </div>

                                                            // ── Action buttons ───────────
                                                            <div class="areg-row-actions">
                                                                // Expand / collapse
                                                                <button
                                                                    class=move || if is_exp() {
                                                                        "areg-btn areg-btn--expand areg-btn--active"
                                                                    } else {
                                                                        "areg-btn areg-btn--expand"
                                                                    }
                                                                    title=move || if is_exp() { "Collapse" } else { "Expand" }
                                                                    on:click=move |_| {
                                                                        if is_exp() {
                                                                            set_expanded_id.set(None);
                                                                            set_preview_id.set(None);
                                                                            set_preview_content.set(String::new());
                                                                            set_reject_id.set(None);
                                                                            set_reject_note.set(String::new());
                                                                        } else {
                                                                            set_expanded_id.set(Some((*sid_expand).clone()));
                                                                        }
                                                                    }
                                                                >
                                                                    {move || if is_exp() {
                                                                        view! { <IconX   class="icon-svg icon-xs" /> }.into_any()
                                                                    } else {
                                                                        view! { <IconEye class="icon-svg icon-xs" /> }.into_any()
                                                                    }}
                                                                </button>

                                                                // Approve
                                                                <button
                                                                    class="areg-btn areg-btn--approve"
                                                                    disabled=move || is_proc() || processing_id.get().is_some()
                                                                    on:click=move |_| {
                                                                        (ha)((*sid_approve).clone())
                                                                    }
                                                                >
                                                                    {move || if is_proc() {
                                                                        view! { <IconLoader class="icon-svg spin" /> }.into_any()
                                                                    } else {
                                                                        view! { <IconCheck  class="icon-svg icon-xs" /> }.into_any()
                                                                    }}
                                                                    " Approve"
                                                                </button>

                                                                // Reject toggle
                                                                <button
                                                                    class=move || if is_rej() {
                                                                        "areg-btn areg-btn--reject areg-btn--active"
                                                                    } else {
                                                                        "areg-btn areg-btn--reject"
                                                                    }
                                                                    disabled=move || is_proc() || processing_id.get().is_some()
                                                                    on:click=move |_| {
                                                                        if is_rej() {
                                                                            set_reject_id.set(None);
                                                                            set_reject_note.set(String::new());
                                                                        } else {
                                                                            set_reject_id.set(Some((*sid_reject_tog).clone()));
                                                                            // Auto-expand so the form is visible
                                                                            set_expanded_id.set(Some((*sid_reject_tog).clone()));
                                                                        }
                                                                    }
                                                                >
                                                                    <IconX class="icon-svg icon-xs" />
                                                                    " Reject"
                                                                </button>
                                                            </div>
                                                        </div>

                                                        // ── Expanded details ─────────────
                                                        <div class=move || {
                                                            if is_exp() { "areg-details" }
                                                            else        { "areg-details areg-details--hidden" }
                                                        }>
                                                            <div class="areg-details-grid">
                                                                <div class="areg-detail-block">
                                                                    <span class="areg-detail-label">"Description"</span>
                                                                    <p class="areg-detail-text">{description}</p>
                                                                </div>

                                                                {if !tags_str.is_empty() {
                                                                    view! {
                                                                        <div class="areg-detail-block">
                                                                            <span class="areg-detail-label">"Tags"</span>
                                                                            <p class="areg-detail-text">{tags_str}</p>
                                                                        </div>
                                                                    }.into_any()
                                                                } else { view! { <span></span> }.into_any() }}
                                                            </div>

                                                            // Preview button
                                                            <div class="areg-preview-bar">
                                                                <button
                                                                    class=move || if is_prev() {
                                                                        "btn btn-ghost btn-sm areg-preview-btn areg-preview-btn--active"
                                                                    } else {
                                                                        "btn btn-ghost btn-sm areg-preview-btn"
                                                                    }
                                                                    on:click=move |_| {
                                                                        (hp)(
                                                                            (*sid_preview).clone(),
                                                                            (*sp).clone(),
                                                                        )
                                                                    }
                                                                >
                                                                    {move || if preview_loading.get() && is_prev() {
                                                                        view! {
                                                                            <IconLoader class="icon-svg spin" />
                                                                            <span>"Loading..."</span>
                                                                        }.into_any()
                                                                    } else {
                                                                        view! {
                                                                            <IconEye class="icon-svg icon-xs" />
                                                                            <span>
                                                                                {move || if is_prev() {
                                                                                    "Hide File"
                                                                                } else {
                                                                                    "Preview File"
                                                                                }}
                                                                            </span>
                                                                        }.into_any()
                                                                    }}
                                                                </button>
                                                            </div>

                                                            // File preview panel
                                                            {move || if is_prev() {
                                                                view! {
                                                                    <div class="areg-preview-wrap">
                                                                        {move || if preview_loading.get() {
                                                                            view! {
                                                                                <div class="areg-preview-loading">
                                                                                    <div class="spinner"></div>
                                                                                </div>
                                                                            }.into_any()
                                                                        } else {
                                                                            view! {
                                                                                <pre class="areg-preview-code">
                                                                                    <code>{preview_content.get()}</code>
                                                                                </pre>
                                                                            }.into_any()
                                                                        }}
                                                                    </div>
                                                                }.into_any()
                                                            } else { view! { <span></span> }.into_any() }}

                                                            // Rejection form
                                                            {move || if is_rej() {
                                                                let hr_clone = hr.clone();
                                                                let sid_cfm  = sid_reject_cfm.clone();
                                                                view! {
                                                                    <div class="areg-reject-form">
                                                                        <div class="form-group">
                                                                            <label class="form-label">
                                                                                "Rejection Note"
                                                                                <span class="form-label-optional">
                                                                                    " — shown to the submitter"
                                                                                </span>
                                                                            </label>
                                                                            <textarea
                                                                                class="form-textarea"
                                                                                rows="3"
                                                                                placeholder="Explain why this submission is being rejected..."
                                                                                prop:value=move || reject_note.get()
                                                                                on:input=move |ev| set_reject_note.set(event_target_value(&ev))
                                                                            ></textarea>
                                                                        </div>
                                                                        <div class="areg-reject-actions">
                                                                            <button
                                                                                class="btn btn-ghost btn-sm"
                                                                                on:click=move |_| {
                                                                                    set_reject_id.set(None);
                                                                                    set_reject_note.set(String::new());
                                                                                }
                                                                            >
                                                                                "Cancel"
                                                                            </button>
                                                                            <button
                                                                                class="btn btn-danger btn-sm"
                                                                                disabled=move || is_proc()
                                                                                on:click=move |_| {
                                                                                    let note = reject_note.get();
                                                                                    (hr_clone)((*sid_cfm).clone(), note);
                                                                                }
                                                                            >
                                                                                {move || if is_proc() {
                                                                                    view! {
                                                                                        <IconLoader class="icon-svg spin" />
                                                                                        <span>"Rejecting..."</span>
                                                                                    }.into_any()
                                                                                } else {
                                                                                    view! {
                                                                                        <span>"Confirm Reject"</span>
                                                                                    }.into_any()
                                                                                }}
                                                                            </button>
                                                                        </div>
                                                                    </div>
                                                                }.into_any()
                                                            } else { view! { <span></span> }.into_any() }}

                                                        </div>
                                                    </div>
                                                }
                                            }).collect_view()
                                        }}
                                    </div>
                                </div>
                            }.into_any()
                        }}
                    </div>
                }.into_any()
            }}
        </div>
    }
  }
