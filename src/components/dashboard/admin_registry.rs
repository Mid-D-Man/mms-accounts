// src/components/dashboard/admin_registry.rs
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;
use std::rc::Rc;
use std::sync::Arc;
use crate::supabase::{SupabaseClient, Profile, RegistrySubmission};
use crate::components::icons::{
    IconPackage, IconCheck, IconX, IconLoader,
    IconAlertTriangle, IconFileText, IconEye,
};

#[component]
pub fn AdminRegistryView(profile: ReadSignal<Option<Profile>>) -> impl IntoView {
    let is_admin = move || profile.get().as_ref().map(|p| p.is_admin()).unwrap_or(false);

    let (submissions,     set_submissions)     = signal(Vec::<RegistrySubmission>::new());
    let (loading,         set_loading)         = signal(true);
    let (error,           set_error)           = signal(String::new());
    let (expanded_id,     set_expanded_id)     = signal(None::<String>);
    let (preview_id,      set_preview_id)      = signal(None::<String>);
    let (preview_content, set_preview_content) = signal(String::new());
    let (preview_loading, set_preview_loading) = signal(false);
    let (reject_id,       set_reject_id)       = signal(None::<String>);
    let (reject_note,     set_reject_note)     = signal(String::new());
    let (processing_id,   set_processing_id)   = signal(None::<String>);
    let (action_error,    set_action_error)    = signal(String::new());
    let (action_success,  set_action_success)  = signal(String::new());

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
                Err(e) => { set_action_error.set(e); set_processing_id.set(None); }
            }
        });
    };

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
                Err(e) => { set_action_error.set(e); set_processing_id.set(None); }
            }
        });
    };

    let handle_preview = move |sub_id: String, storage_path: Option<String>| {
        if preview_id.get().as_deref() == Some(sub_id.as_str()) {
            set_preview_id.set(None);
            set_preview_content.set(String::new());
            return;
        }
        let path = match storage_path.filter(|p| !p.is_empty()) {
            Some(p) => p,
            None => {
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
                Ok(text) => { set_preview_content.set(text);                              set_preview_loading.set(false); }
                Err(e)   => { set_preview_content.set(format!("// Error:\n// {}", e));    set_preview_loading.set(false); }
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
                                            let ha = Arc::new(handle_approve);
                                            let hr = Arc::new(handle_reject);
                                            let hp = Arc::new(handle_preview);

                                            submissions.get().into_iter().map(|sub| {
                                                let ha = ha.clone();
                                                let hr = hr.clone();
                                                let hp = hp.clone();

                                                let sid = Arc::new(sub.id.clone());
                                                let sp  = Arc::new(sub.supabase_storage_path.clone());

                                                // ── Per-row reactive checks wrapped in Rc so each
                                                //    closure site can independently clone without
                                                //    requiring Send+Sync (WASM is single-threaded).
                                                //    Calling `.clone()` on an Rc inside a move ||
                                                //    closure body keeps the closure Fn, not FnOnce.
                                                let is_exp: Rc<dyn Fn() -> bool> = {
                                                    let s = sid.clone();
                                                    Rc::new(move || expanded_id.get().as_deref() == Some(s.as_str()))
                                                };
                                                let is_prev: Rc<dyn Fn() -> bool> = {
                                                    let s = sid.clone();
                                                    Rc::new(move || preview_id.get().as_deref() == Some(s.as_str()))
                                                };
                                                let is_rej: Rc<dyn Fn() -> bool> = {
                                                    let s = sid.clone();
                                                    Rc::new(move || reject_id.get().as_deref() == Some(s.as_str()))
                                                };
                                                let is_proc: Rc<dyn Fn() -> bool> = {
                                                    let s = sid.clone();
                                                    Rc::new(move || processing_id.get().as_deref() == Some(s.as_str()))
                                                };

                                                // Arc<String> for click-handler keys
                                                let sid_approve    = sid.clone();
                                                let sid_expand     = sid.clone();
                                                let sid_reject_tog = sid.clone();
                                                let sid_reject_cfm = sid.clone();
                                                let sid_preview    = sid.clone();

                                                // Static display data
                                                let filename    = sub.filename.clone();
                                                let category    = sub.category.clone();
                                                let version     = sub.version.clone();
                                                let mid_id      = sub.mid_id.clone();
                                                let description = sub.description.clone();
                                                let tags_str    = sub.tags.join(", ");
                                                let submitted   = sub.formatted_submitted();

                                                // ── One Rc clone per closure site that needs is_exp ──
                                                let ie_div  = is_exp.clone(); // outer div class
                                                let ie_bcls = is_exp.clone(); // expand btn class
                                                let ie_bttl = is_exp.clone(); // expand btn title
                                                let ie_bclk = is_exp.clone(); // expand btn on:click
                                                let ie_bcon = is_exp.clone(); // expand btn content
                                                let ie_det  = is_exp.clone(); // details div class

                                                // ── is_prev ──────────────────────────────────────────
                                                let ip_bcls  = is_prev.clone(); // preview btn class
                                                // ip_con: captured by the btn content closure;
                                                // inside that closure body we call ip_con.clone()
                                                // to get ip_span for the nested span closure.
                                                let ip_con   = is_prev.clone();
                                                let ip_panel = is_prev.clone(); // preview panel block

                                                // ── is_rej ───────────────────────────────────────────
                                                let ir_bcls = is_rej.clone(); // reject btn class
                                                let ir_bclk = is_rej.clone(); // reject btn on:click
                                                // ir_form: captured by the form reactive block;
                                                // iproc_form is also captured and cloned inside.
                                                let ir_form = is_rej.clone();

                                                // ── is_proc ──────────────────────────────────────────
                                                let iproc_adis = is_proc.clone(); // approve disabled
                                                let iproc_acon = is_proc.clone(); // approve content
                                                let iproc_rdis = is_proc.clone(); // reject btn disabled
                                                // iproc_form: captured by the reject form closure;
                                                // cloned inside body for disabled + content closures.
                                                let iproc_form = is_proc.clone();

                                                view! {
                                                    <div class=move || {
                                                        if ie_div() { "areg-row areg-row--expanded" }
                                                        else        { "areg-row" }
                                                    }>

                                                        // ── Row header ──────────────────────────────
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

                                                            <div class="areg-row-actions">

                                                                // Expand / collapse
                                                                <button
                                                                    class=move || if ie_bcls() {
                                                                        "areg-btn areg-btn--expand areg-btn--active"
                                                                    } else {
                                                                        "areg-btn areg-btn--expand"
                                                                    }
                                                                    title=move || if ie_bttl() { "Collapse" } else { "Expand" }
                                                                    on:click=move |_| {
                                                                        if ie_bclk() {
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
                                                                    {move || if ie_bcon() {
                                                                        view! { <IconX   class="icon-svg icon-xs" /> }.into_any()
                                                                    } else {
                                                                        view! { <IconEye class="icon-svg icon-xs" /> }.into_any()
                                                                    }}
                                                                </button>

                                                                // Approve
                                                                <button
                                                                    class="areg-btn areg-btn--approve"
                                                                    disabled=move || iproc_adis() || processing_id.get().is_some()
                                                                    on:click=move |_| { (ha)((*sid_approve).clone()) }
                                                                >
                                                                    {move || if iproc_acon() {
                                                                        view! { <IconLoader class="icon-svg spin" /> }.into_any()
                                                                    } else {
                                                                        view! { <IconCheck  class="icon-svg icon-xs" /> }.into_any()
                                                                    }}
                                                                    " Approve"
                                                                </button>

                                                                // Reject toggle
                                                                <button
                                                                    class=move || if ir_bcls() {
                                                                        "areg-btn areg-btn--reject areg-btn--active"
                                                                    } else {
                                                                        "areg-btn areg-btn--reject"
                                                                    }
                                                                    disabled=move || iproc_rdis() || processing_id.get().is_some()
                                                                    on:click=move |_| {
                                                                        if ir_bclk() {
                                                                            set_reject_id.set(None);
                                                                            set_reject_note.set(String::new());
                                                                        } else {
                                                                            set_reject_id.set(Some((*sid_reject_tog).clone()));
                                                                            set_expanded_id.set(Some((*sid_reject_tog).clone()));
                                                                        }
                                                                    }
                                                                >
                                                                    <IconX class="icon-svg icon-xs" />
                                                                    " Reject"
                                                                </button>
                                                            </div>
                                                        </div>

                                                        // ── Expanded details ─────────────────────────
                                                        <div class=move || {
                                                            if ie_det() { "areg-details" }
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
                                                                    class=move || if ip_bcls() {
                                                                        "btn btn-ghost btn-sm areg-preview-btn areg-preview-btn--active"
                                                                    } else {
                                                                        "btn btn-ghost btn-sm areg-preview-btn"
                                                                    }
                                                                    on:click=move |_| {
                                                                        (hp)((*sid_preview).clone(), (*sp).clone())
                                                                    }
                                                                >
                                                                    // ip_con owns its Rc. Inside the body we
                                                                    // call ip_con.clone() for the nested span
                                                                    // closure — this keeps the outer Fn.
                                                                    {move || if preview_loading.get() && ip_con() {
                                                                        view! {
                                                                            <IconLoader class="icon-svg spin" />
                                                                            <span>"Loading..."</span>
                                                                        }.into_any()
                                                                    } else {
                                                                        let ip_span = ip_con.clone();
                                                                        view! {
                                                                            <IconEye class="icon-svg icon-xs" />
                                                                            <span>
                                                                                {move || if ip_span() { "Hide File" } else { "Preview File" }}
                                                                            </span>
                                                                        }.into_any()
                                                                    }}
                                                                </button>
                                                            </div>

                                                            // File preview panel
                                                            {move || if ip_panel() {
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
                                                            // ir_form owns its Rc; iproc_form also owned.
                                                            // Inside the body we clone iproc_form twice
                                                            // (for disabled attr and for content block).
                                                            {move || if ir_form() {
                                                                let iproc_fdis = iproc_form.clone();
                                                                let iproc_fcon = iproc_form.clone();
                                                                let hr_c  = hr.clone();
                                                                let sid_c = sid_reject_cfm.clone();
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
                                                                                disabled=move || iproc_fdis()
                                                                                on:click=move |_| {
                                                                                    let note = reject_note.get();
                                                                                    (hr_c)((*sid_c).clone(), note);
                                                                                }
                                                                            >
                                                                                {move || if iproc_fcon() {
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
