// src/components/dashboard/admin_r2.rs
// R2 file browser for admins.
// Lists files in the mdix-registry R2 bucket via dixscript-docs API,
// supports prefix-based navigation, delete with confirmation, move/rename,
// and direct upload to R2.

use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use std::sync::Arc;
use gloo_storage::Storage;
use crate::supabase::{SupabaseClient, Profile, R2FileInfo};
use crate::components::icons::{
    IconFolder, IconFileText, IconTrash, IconRefresh, IconLoader,
    IconUpload, IconAlertTriangle, IconCheck, IconEdit, IconX,
};

// ── Constants ──────────────────────────────────────────────────
const DIXSCRIPT_DOCS: &str = "https://dixscript-docs.pages.dev";

// ── Helper: bearer token from local storage ────────────────────
fn get_token() -> Option<String> {
    gloo_storage::LocalStorage::get::<String>("mms_access_token").ok()
}

// ── R2 API calls ───────────────────────────────────────────────

async fn api_list(prefix: &str, token: &str) -> Result<Vec<R2FileInfo>, String> {
    let prefix_enc = js_sys::encode_uri_component(prefix)
        .as_string()
        .unwrap_or_else(|| prefix.to_string());
    let url = format!("{}/api/admin/r2/list?prefix={}", DIXSCRIPT_DOCS, prefix_enc);

    let res = gloo_net::http::Request::get(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .send().await
        .map_err(|e| format!("Network error: {}", e))?;

    if !res.ok() {
        let text = res.text().await.unwrap_or_default();
        return Err(format!("List failed: {}", text));
    }

    #[derive(serde::Deserialize)]
    struct Resp { files: Vec<R2FileInfo> }
    let resp: Resp = res.json().await
        .map_err(|e| format!("Parse error: {}", e))?;
    Ok(resp.files)
}

async fn api_delete(key: &str, token: &str) -> Result<(), String> {
    let url  = format!("{}/api/admin/r2/delete", DIXSCRIPT_DOCS);
    let body = serde_json::json!({ "key": key });

    let res = gloo_net::http::Request::post(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .body(body.to_string())
        .map_err(|e| format!("Request error: {}", e))?
        .send().await
        .map_err(|e| format!("Network error: {}", e))?;

    if !res.ok() {
        let text = res.text().await.unwrap_or_default();
        return Err(format!("Delete failed: {}", text));
    }
    Ok(())
}

async fn api_move(from_key: &str, to_key: &str, token: &str) -> Result<(), String> {
    let url  = format!("{}/api/admin/r2/move", DIXSCRIPT_DOCS);
    let body = serde_json::json!({ "from_key": from_key, "to_key": to_key });

    let res = gloo_net::http::Request::post(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .body(body.to_string())
        .map_err(|e| format!("Request error: {}", e))?
        .send().await
        .map_err(|e| format!("Network error: {}", e))?;

    if !res.ok() {
        let text = res.text().await.unwrap_or_default();
        return Err(format!("Move failed: {}", text));
    }
    Ok(())
}

// ── Component ──────────────────────────────────────────────────

#[component]
pub fn AdminR2View(profile: ReadSignal<Option<Profile>>) -> impl IntoView {
    let is_admin = move || profile.get().as_ref().map(|p| p.is_admin()).unwrap_or(false);

    // ── State ──────────────────────────────────────────────────
    let (files,          set_files)          = signal(Vec::<R2FileInfo>::new());
    let (loading,        set_loading)        = signal(true);
    let (error,          set_error)          = signal(String::new());
    let (action_error,   set_action_error)   = signal(String::new());
    let (action_success, set_action_success) = signal(String::new());
    let (prefix,         set_prefix)         = signal(String::new());

    // per-row state
    let (confirm_delete, set_confirm_delete) = signal(None::<String>);
    let (deleting_key,   set_deleting_key)   = signal(None::<String>);
    let (rename_key,     set_rename_key)     = signal(None::<String>);
    let (rename_value,   set_rename_value)   = signal(String::new());
    let (renaming_key,   set_renaming_key)   = signal(None::<String>);

    // upload form
    let (show_upload,   set_show_upload)   = signal(false);
    let (up_category,   set_up_category)   = signal("game".to_string());
    let (up_desc,       set_up_desc)       = signal(String::new());
    let (up_tags,       set_up_tags)       = signal(String::new());
    let (up_version,    set_up_version)    = signal("1.0.0".to_string());
    let (up_filename,   set_up_filename)   = signal(String::new());
    let (up_file,       set_up_file)       = signal(None::<web_sys::File>);
    let (uploading,     set_uploading)     = signal(false);
    let (upload_error,  set_upload_error)  = signal(String::new());

    // ── Load files ─────────────────────────────────────────────
    let load_files = move || {
        let prefix_val = prefix.get();
        set_loading.set(true);
        set_error.set(String::new());
        spawn_local(async move {
            match get_token() {
                None => {
                    set_error.set("Not authenticated.".to_string());
                    set_loading.set(false);
                }
                Some(token) => {
                    match api_list(&prefix_val, &token).await {
                        Ok(f)  => { set_files.set(f); set_loading.set(false); }
                        Err(e) => { set_error.set(e);  set_loading.set(false); }
                    }
                }
            }
        });
    };

    // Initial load
    Effect::new(move |_| {
        if !is_admin() { set_loading.set(false); return; }
        load_files();
    });

    let load_files_arc = Arc::new(load_files);

    // ── Delete ─────────────────────────────────────────────────
    let handle_delete = {
        let lf = load_files_arc.clone();
        move |key: String| {
            set_deleting_key.set(Some(key.clone()));
            set_action_error.set(String::new());
            set_action_success.set(String::new());
            let lf = lf.clone();
            spawn_local(async move {
                match get_token() {
                    None => {
                        set_action_error.set("Not authenticated.".to_string());
                        set_deleting_key.set(None);
                    }
                    Some(token) => {
                        match api_delete(&key, &token).await {
                            Ok(()) => {
                                set_confirm_delete.set(None);
                                set_deleting_key.set(None);
                                set_action_success.set("File deleted.".to_string());
                                lf();
                            }
                            Err(e) => {
                                set_action_error.set(e);
                                set_deleting_key.set(None);
                            }
                        }
                    }
                }
            });
        }
    };

    // ── Rename / Move ──────────────────────────────────────────
    let handle_rename = {
        let lf = load_files_arc.clone();
        move |from_key: String, to_key: String| {
            if to_key.trim().is_empty() {
                set_action_error.set("New key cannot be empty.".to_string());
                return;
            }
            set_renaming_key.set(Some(from_key.clone()));
            set_action_error.set(String::new());
            set_action_success.set(String::new());
            let lf = lf.clone();
            spawn_local(async move {
                match get_token() {
                    None => {
                        set_action_error.set("Not authenticated.".to_string());
                        set_renaming_key.set(None);
                    }
                    Some(token) => {
                        match api_move(&from_key, &to_key, &token).await {
                            Ok(()) => {
                                set_rename_key.set(None);
                                set_rename_value.set(String::new());
                                set_renaming_key.set(None);
                                set_action_success.set("File moved.".to_string());
                                lf();
                            }
                            Err(e) => {
                                set_action_error.set(e);
                                set_renaming_key.set(None);
                            }
                        }
                    }
                }
            });
        }
    };

    // ── Upload ─────────────────────────────────────────────────
    let handle_upload = {
        let lf = load_files_arc.clone();
        move |ev: web_sys::SubmitEvent| {
            ev.prevent_default();
            let file = match up_file.get() {
                Some(f) => f,
                None => {
                    set_upload_error.set("Select a .mdix file.".to_string());
                    return;
                }
            };
            if !file.name().ends_with(".mdix") {
                set_upload_error.set("Only .mdix files are accepted.".to_string());
                return;
            }
            if up_desc.get().trim().is_empty() {
                set_upload_error.set("Description is required.".to_string());
                return;
            }

            set_uploading.set(true);
            set_upload_error.set(String::new());

            let category = up_category.get();
            let desc     = up_desc.get();
            let tags     = up_tags.get();
            let version  = up_version.get();
            let lf       = lf.clone();

            spawn_local(async move {
                let token = match get_token() {
                    Some(t) => t,
                    None => {
                        set_upload_error.set("Not authenticated.".to_string());
                        set_uploading.set(false);
                        return;
                    }
                };

                let form_data = match web_sys::FormData::new() {
                    Ok(f)  => f,
                    Err(_) => {
                        set_upload_error.set("Failed to create form data.".to_string());
                        set_uploading.set(false);
                        return;
                    }
                };

                let blob: &web_sys::Blob = file.as_ref();
                let _ = form_data.append_with_blob_and_filename("file", blob, &file.name());
                let _ = form_data.append_with_str("category", &category);
                let _ = form_data.append_with_str("desc",     &desc);
                let _ = form_data.append_with_str("tags",     &tags);
                let _ = form_data.append_with_str("version",  &version);
                let _ = form_data.append_with_str("addedBy",  "MidManStudio");

                let url = format!("{}/api/admin/r2/upload", DIXSCRIPT_DOCS);

                let res = gloo_net::http::Request::post(&url)
                    .header("Authorization", &format!("Bearer {}", token))
                    .body(form_data)
                    .map_err(|e| format!("Request error: {}", e));

                match res {
                    Err(e) => {
                        set_upload_error.set(e);
                        set_uploading.set(false);
                    }
                    Ok(req) => {
                        match req.send().await {
                            Err(e) => {
                                set_upload_error.set(format!("Network error: {}", e));
                                set_uploading.set(false);
                            }
                            Ok(r) => {
                                if r.ok() {
                                    set_show_upload.set(false);
                                    set_up_desc.set(String::new());
                                    set_up_tags.set(String::new());
                                    set_up_version.set("1.0.0".to_string());
                                    set_up_file.set(None);
                                    set_up_filename.set(String::new());
                                    set_action_success.set("File uploaded.".to_string());
                                    lf();
                                } else {
                                    let text = r.text().await.unwrap_or_default();
                                    set_upload_error.set(format!("Upload failed: {}", text));
                                }
                                set_uploading.set(false);
                            }
                        }
                    }
                }
            });
        }
    };

    let handle_file_change = move |ev: web_sys::Event| {
        let input = ev.target()
            .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok());
        if let Some(input) = input {
            if let Some(files) = input.files() {
                if let Some(file) = files.get(0) {
                    set_up_filename.set(file.name());
                    set_up_file.set(Some(file));
                }
            }
        }
    };

    let lf_refresh = load_files_arc.clone();

    view! {
        <div class="admin-r2-view">
            {move || if !is_admin() {
                view! {
                    <div class="admin-gate">
                        <IconFolder class="icon-svg admin-gate-icon" />
                        <p class="admin-gate-text">"Admin access required."</p>
                    </div>
                }.into_any()
            } else {
                view! {
                    <div class="admin-r2-content">

                        // ── Header ─────────────────────────────
                        <div class="admin-section-header">
                            <div class="admin-section-header-icon">
                                <IconFolder class="icon-svg" />
                            </div>
                            <div>
                                <h1 class="admin-section-title">"R2 File Browser"</h1>
                                <p class="admin-section-subtitle">
                                    "Browse, upload, rename, and delete files in the "
                                    "mdix-registry Cloudflare R2 bucket."
                                </p>
                            </div>
                        </div>

                        // ── Toolbar ─────────────────────────────
                        <div class="r2-toolbar">
                            <div class="r2-breadcrumb">
                                <button
                                    class="r2-breadcrumb-btn"
                                    on:click=move |_| {
                                        set_prefix.set(String::new());
                                        let lf = lf_refresh.clone();
                                        lf();
                                    }
                                >
                                    "mdix-registry/"
                                </button>
                                {move || {
                                    let p = prefix.get();
                                    if p.is_empty() {
                                        view! { <span></span> }.into_any()
                                    } else {
                                        view! {
                                            <span class="r2-breadcrumb-sep">"/"</span>
                                            <span class="r2-breadcrumb-current">{p}</span>
                                        }.into_any()
                                    }
                                }}
                            </div>
                            <div class="r2-toolbar-actions">
                                <button
                                    class="btn btn-ghost btn-sm"
                                    on:click={
                                        let lf = load_files_arc.clone();
                                        move |_| lf()
                                    }
                                    disabled=move || loading.get()
                                >
                                    {move || if loading.get() {
                                        view! { <IconLoader class="icon-svg spin" /> }.into_any()
                                    } else {
                                        view! { <IconRefresh class="icon-svg icon-sm" /> }.into_any()
                                    }}
                                    "Refresh"
                                </button>
                                <button
                                    class="btn btn-primary btn-sm"
                                    on:click=move |_| {
                                        set_show_upload.set(true);
                                        set_upload_error.set(String::new());
                                    }
                                >
                                    <IconUpload class="icon-svg icon-xs" />
                                    "Upload File"
                                </button>
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

                        // ── Upload form ─────────────────────────
                        {move || if show_upload.get() {
                            view! {
                                <div class="r2-upload-wrap">
                                    <h3 class="r2-upload-title">"Upload Package to R2"</h3>
                                    <form class="r2-upload-form" on:submit=handle_upload>

                                        <div class="form-group">
                                            <label class="form-label">"Package File (.mdix)"</label>
                                            <label class="file-drop-zone">
                                                <input
                                                    type="file"
                                                    accept=".mdix"
                                                    class="file-input-hidden"
                                                    on:change=handle_file_change
                                                />
                                                <div class="file-drop-inner">
                                                    <IconUpload class="icon-svg file-drop-icon" />
                                                    {move || if up_filename.get().is_empty() {
                                                        view! {
                                                            <span class="file-drop-text">
                                                                "Click to select a .mdix file"
                                                            </span>
                                                        }.into_any()
                                                    } else {
                                                        view! {
                                                            <span class="file-drop-selected">
                                                                {up_filename.get()}
                                                            </span>
                                                        }.into_any()
                                                    }}
                                                </div>
                                            </label>
                                        </div>

                                        <div class="r2-upload-row">
                                            <div class="form-group">
                                                <label class="form-label">"Category"</label>
                                                <select
                                                    class="form-input"
                                                    on:change=move |ev| {
                                                        if let Some(sel) = ev.target()
                                                            .and_then(|t| t.dyn_into::<web_sys::HtmlSelectElement>().ok())
                                                        {
                                                            set_up_category.set(sel.value());
                                                        }
                                                    }
                                                >
                                                    <option value="game">"game"</option>
                                                    <option value="ml">"ml"</option>
                                                    <option value="api">"api"</option>
                                                    <option value="crypto">"crypto"</option>
                                                    <option value="ecommerce">"ecommerce"</option>
                                                    <option value="utils">"utils"</option>
                                                </select>
                                            </div>
                                            <div class="form-group">
                                                <label class="form-label">"Version"</label>
                                                <input
                                                    class="form-input"
                                                    type="text"
                                                    placeholder="1.0.0"
                                                    prop:value=move || up_version.get()
                                                    on:input=move |ev| set_up_version.set(event_target_value(&ev))
                                                />
                                            </div>
                                        </div>

                                        <div class="form-group">
                                            <label class="form-label">"Description *"</label>
                                            <textarea
                                                class="form-textarea"
                                                rows="2"
                                                placeholder="What does this package provide?"
                                                prop:value=move || up_desc.get()
                                                on:input=move |ev| set_up_desc.set(event_target_value(&ev))
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
                                                placeholder="rpg, enums, config"
                                                prop:value=move || up_tags.get()
                                                on:input=move |ev| set_up_tags.set(event_target_value(&ev))
                                            />
                                        </div>

                                        {move || if !upload_error.get().is_empty() {
                                            view! {
                                                <div class="status-msg status-msg--error">
                                                    {upload_error.get()}
                                                </div>
                                            }.into_any()
                                        } else { view! { <span></span> }.into_any() }}

                                        <div class="r2-upload-actions">
                                            <button
                                                type="button"
                                                class="btn btn-ghost btn-sm"
                                                disabled=move || uploading.get()
                                                on:click=move |_| {
                                                    set_show_upload.set(false);
                                                    set_upload_error.set(String::new());
                                                }
                                            >
                                                "Cancel"
                                            </button>
                                            <button
                                                type="submit"
                                                class="btn btn-primary btn-sm"
                                                disabled=move || uploading.get()
                                            >
                                                {move || if uploading.get() {
                                                    view! {
                                                        <IconLoader class="icon-svg spin" />
                                                        <span>"Uploading..."</span>
                                                    }.into_any()
                                                } else {
                                                    view! {
                                                        <IconUpload class="icon-svg icon-xs" />
                                                        <span>"Upload to R2"</span>
                                                    }.into_any()
                                                }}
                                            </button>
                                        </div>
                                    </form>
                                </div>
                            }.into_any()
                        } else { view! { <span></span> }.into_any() }}

                        // ── File list ───────────────────────────
                        {move || if loading.get() {
                            view! {
                                <div class="r2-loading"><div class="spinner"></div></div>
                            }.into_any()
                        } else if files.get().is_empty() {
                            view! {
                                <div class="r2-empty">
                                    <IconFolder class="icon-svg r2-empty-icon" />
                                    <p class="r2-empty-title">"No files found"</p>
                                    <p class="r2-empty-sub">
                                        "Upload a .mdix file or check the current prefix."
                                    </p>
                                </div>
                            }.into_any()
                        } else {
                            let total = files.get().len();
                            let mdix_count = files.get().iter().filter(|f| !f.is_meta).count();
                            view! {
                                <div class="r2-list-wrap">
                                    <div class="r2-stats-bar">
                                        <span class="r2-stat">
                                            <strong>{total.to_string()}</strong>
                                            " total objects"
                                        </span>
                                        <span class="r2-stat-sep">"·"</span>
                                        <span class="r2-stat">
                                            <strong>{mdix_count.to_string()}</strong>
                                            " .mdix packages"
                                        </span>
                                    </div>
                                    <div class="r2-list">
                                        {move || {
                                            let hd = Arc::new(handle_delete.clone());
                                            let hr = Arc::new(handle_rename.clone());

                                            files.get().into_iter().map(|file| {
                                                let hd = hd.clone();
                                                let hr = hr.clone();

                                                let key        = Arc::new(file.key.clone());
                                                let key_del    = key.clone();
                                                let key_cfm    = key.clone();
                                                let key_cancel = key.clone();
                                                let key_ren    = key.clone();
                                                let key_ren_go = key.clone();

                                                let is_confirming = {
                                                    let k = key.clone();
                                                    move || confirm_delete.get().as_deref() == Some(k.as_str())
                                                };
                                                let is_deleting = {
                                                    let k = key.clone();
                                                    move || deleting_key.get().as_deref() == Some(k.as_str())
                                                };
                                                let is_renaming_row = {
                                                    let k = key.clone();
                                                    move || rename_key.get().as_deref() == Some(k.as_str())
                                                };
                                                let is_renaming_in_progress = {
                                                    let k = key.clone();
                                                    move || renaming_key.get().as_deref() == Some(k.as_str())
                                                };

                                                let name     = file.name.clone();
                                                let size_str = file.display_size();
                                                let date_str = file.formatted_uploaded();
                                                let category = file.category();
                                                let is_meta  = file.is_meta;

                                                view! {
                                                    <div class="r2-row">
                                                        <div class="r2-row-main">
                                                            <div class="r2-row-icon">
                                                                {if is_meta {
                                                                    view! { <IconFileText class="icon-svg icon-xs" /> }.into_any()
                                                                } else {
                                                                    view! { <IconFolder class="icon-svg icon-xs" /> }.into_any()
                                                                }}
                                                            </div>
                                                            <div class="r2-row-info">
                                                                <div class="r2-row-top">
                                                                    <span class="r2-filename">{name}</span>
                                                                    {if !is_meta {
                                                                        view! {
                                                                            <span class="r2-badge">{category}</span>
                                                                        }.into_any()
                                                                    } else {
                                                                        view! {
                                                                            <span class="r2-badge r2-badge--meta">"meta"</span>
                                                                        }.into_any()
                                                                    }}
                                                                </div>
                                                                <div class="r2-row-meta">
                                                                    <span class="r2-meta-item">{size_str}</span>
                                                                    <span class="r2-meta-sep">"·"</span>
                                                                    <span class="r2-meta-item">{date_str}</span>
                                                                </div>
                                                            </div>
                                                            <div class="r2-row-actions">
                                                                // Rename toggle
                                                                <button
                                                                    class=move || if is_renaming_row() {
                                                                        "r2-btn r2-btn--active"
                                                                    } else { "r2-btn" }
                                                                    title="Rename / Move"
                                                                    on:click=move |_| {
                                                                        if is_renaming_row() {
                                                                            set_rename_key.set(None);
                                                                            set_rename_value.set(String::new());
                                                                        } else {
                                                                            set_rename_key.set(Some((*key_ren).clone()));
                                                                            set_rename_value.set((*key_ren).clone());
                                                                        }
                                                                    }
                                                                >
                                                                    <IconEdit class="icon-svg icon-xs" />
                                                                </button>
                                                                // Delete / Confirm
                                                                {move || {
                                                                    let hd = hd.clone();
                                                                    let key_cfm_inner    = key_cfm.clone();
                                                                    let key_cancel_inner = key_cancel.clone();
                                                                    let key_del_inner    = key_del.clone();

                                                                    if is_confirming() {
                                                                        view! {
                                                                            <button
                                                                                class="r2-btn r2-btn--danger"
                                                                                disabled=move || is_deleting()
                                                                                on:click=move |_| {
                                                                                    hd((*key_cfm_inner).clone());
                                                                                }
                                                                            >
                                                                                {move || if is_deleting() {
                                                                                    view! { <IconLoader class="icon-svg spin" /> }.into_any()
                                                                                } else {
                                                                                    view! { <IconCheck class="icon-svg icon-xs" /> }.into_any()
                                                                                }}
                                                                            </button>
                                                                            <button
                                                                                class="r2-btn"
                                                                                on:click=move |_| {
                                                                                    if confirm_delete.get().as_deref() == Some(key_cancel_inner.as_str()) {
                                                                                        set_confirm_delete.set(None);
                                                                                    }
                                                                                }
                                                                            >
                                                                                <IconX class="icon-svg icon-xs" />
                                                                            </button>
                                                                        }.into_any()
                                                                    } else {
                                                                        view! {
                                                                            <button
                                                                                class="r2-btn r2-btn--delete"
                                                                                title="Delete"
                                                                                on:click=move |_| {
                                                                                    set_confirm_delete.set(Some((*key_del_inner).clone()));
                                                                                }
                                                                            >
                                                                                <IconTrash class="icon-svg icon-xs" />
                                                                            </button>
                                                                        }.into_any()
                                                                    }
                                                                }}
                                                            </div>
                                                        </div>

                                                        // Rename form
                                                        {move || if is_renaming_row() {
                                                            let hr_inner = hr.clone();
                                                            let key_go   = key_ren_go.clone();
                                                            view! {
                                                                <div class="r2-rename-form">
                                                                    <input
                                                                        class="form-input r2-rename-input"
                                                                        type="text"
                                                                        placeholder="New R2 key path"
                                                                        prop:value=move || rename_value.get()
                                                                        on:input=move |ev| set_rename_value.set(event_target_value(&ev))
                                                                    />
                                                                    <button
                                                                        class="btn btn-primary btn-sm"
                                                                        disabled=move || is_renaming_in_progress()
                                                                        on:click=move |_| {
                                                                            hr_inner((*key_go).clone(), rename_value.get());
                                                                        }
                                                                    >
                                                                        {move || if is_renaming_in_progress() {
                                                                            view! {
                                                                                <IconLoader class="icon-svg spin" />
                                                                                <span>"Moving..."</span>
                                                                            }.into_any()
                                                                        } else {
                                                                            view! { <span>"Move"</span> }.into_any()
                                                                        }}
                                                                    </button>
                                                                    <button
                                                                        class="btn btn-ghost btn-sm"
                                                                        on:click=move |_| {
                                                                            set_rename_key.set(None);
                                                                            set_rename_value.set(String::new());
                                                                        }
                                                                    >
                                                                        "Cancel"
                                                                    </button>
                                                                </div>
                                                            }.into_any()
                                                        } else { view! { <span></span> }.into_any() }}

                                                        // Delete confirm label
                                                        {move || if is_confirming() && !is_deleting() {
                                                            view! {
                                                                <div class="r2-delete-confirm">
                                                                    "Permanently delete this file and its .meta.json sidecar?"
                                                                </div>
                                                            }.into_any()
                                                        } else { view! { <span></span> }.into_any() }}
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
