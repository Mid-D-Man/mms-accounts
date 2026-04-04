use leptos::prelude::*;

#[component]
pub fn IconUser(#[prop(default = "icon-svg")] class: &'static str) -> impl IntoView {
    view! {
        <svg class=class viewBox="0 0 24 24" fill="none" stroke="currentColor"
             stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"
             xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
            <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/>
            <circle cx="12" cy="7" r="4"/>
        </svg>
    }
}

#[component]
pub fn IconMail(#[prop(default = "icon-svg")] class: &'static str) -> impl IntoView {
    view! {
        <svg class=class viewBox="0 0 24 24" fill="none" stroke="currentColor"
             stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"
             xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
            <path d="M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z"/>
            <polyline points="22,6 12,13 2,6"/>
        </svg>
    }
}

#[component]
pub fn IconLock(#[prop(default = "icon-svg")] class: &'static str) -> impl IntoView {
    view! {
        <svg class=class viewBox="0 0 24 24" fill="none" stroke="currentColor"
             stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"
             xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
            <rect x="3" y="11" width="18" height="11" rx="2" ry="2"/>
            <path d="M7 11V7a5 5 0 0 1 10 0v4"/>
        </svg>
    }
}

#[component]
pub fn IconEye(#[prop(default = "icon-svg")] class: &'static str) -> impl IntoView {
    view! {
        <svg class=class viewBox="0 0 24 24" fill="none" stroke="currentColor"
             stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"
             xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
            <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/>
            <circle cx="12" cy="12" r="3"/>
        </svg>
    }
}

#[component]
pub fn IconEyeOff(#[prop(default = "icon-svg")] class: &'static str) -> impl IntoView {
    view! {
        <svg class=class viewBox="0 0 24 24" fill="none" stroke="currentColor"
             stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"
             xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
            <path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24"/>
            <line x1="1" y1="1" x2="23" y2="23"/>
        </svg>
    }
}

#[component]
pub fn IconCheck(#[prop(default = "icon-svg")] class: &'static str) -> impl IntoView {
    view! {
        <svg class=class viewBox="0 0 24 24" fill="none" stroke="currentColor"
             stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"
             xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
            <polyline points="20 6 9 17 4 12"/>
        </svg>
    }
}

#[component]
pub fn IconX(#[prop(default = "icon-svg")] class: &'static str) -> impl IntoView {
    view! {
        <svg class=class viewBox="0 0 24 24" fill="none" stroke="currentColor"
             stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
             xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
            <line x1="18" y1="6" x2="6" y2="18"/>
            <line x1="6" y1="6" x2="18" y2="18"/>
        </svg>
    }
}

#[component]
pub fn IconSettings(#[prop(default = "icon-svg")] class: &'static str) -> impl IntoView {
    view! {
        <svg class=class viewBox="0 0 24 24" fill="none" stroke="currentColor"
             stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"
             xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
            <circle cx="12" cy="12" r="3"/>
            <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/>
        </svg>
    }
}

#[component]
pub fn IconShield(#[prop(default = "icon-svg")] class: &'static str) -> impl IntoView {
    view! {
        <svg class=class viewBox="0 0 24 24" fill="none" stroke="currentColor"
             stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"
             xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
            <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/>
        </svg>
    }
}

#[component]
pub fn IconLogOut(#[prop(default = "icon-svg")] class: &'static str) -> impl IntoView {
    view! {
        <svg class=class viewBox="0 0 24 24" fill="none" stroke="currentColor"
             stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"
             xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
            <path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"/>
            <polyline points="16 17 21 12 16 7"/>
            <line x1="21" y1="12" x2="9" y2="12"/>
        </svg>
    }
}

#[component]
pub fn IconUsers(#[prop(default = "icon-svg")] class: &'static str) -> impl IntoView {
    view! {
        <svg class=class viewBox="0 0 24 24" fill="none" stroke="currentColor"
             stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"
             xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
            <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/>
            <circle cx="9" cy="7" r="4"/>
            <path d="M23 21v-2a4 4 0 0 0-3-3.87"/>
            <path d="M16 3.13a4 4 0 0 1 0 7.75"/>
        </svg>
    }
}

#[component]
pub fn IconHome(#[prop(default = "icon-svg")] class: &'static str) -> impl IntoView {
    view! {
        <svg class=class viewBox="0 0 24 24" fill="none" stroke="currentColor"
             stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"
             xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
            <path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/>
            <polyline points="9 22 9 12 15 12 15 22"/>
        </svg>
    }
}

#[component]
pub fn IconEdit(#[prop(default = "icon-svg")] class: &'static str) -> impl IntoView {
    view! {
        <svg class=class viewBox="0 0 24 24" fill="none" stroke="currentColor"
             stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"
             xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
            <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/>
            <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/>
        </svg>
    }
}

#[component]
pub fn IconLoader(#[prop(default = "icon-svg spin")] class: &'static str) -> impl IntoView {
    view! {
        <svg class=class viewBox="0 0 24 24" fill="none" stroke="currentColor"
             stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
             xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
            <line x1="12" y1="2"  x2="12" y2="6"/>
            <line x1="12" y1="18" x2="12" y2="22"/>
            <line x1="4.93"  y1="4.93"  x2="7.76"  y2="7.76"/>
            <line x1="16.24" y1="16.24" x2="19.07" y2="19.07"/>
            <line x1="2"  y1="12" x2="6"  y2="12"/>
            <line x1="18" y1="12" x2="22" y2="12"/>
            <line x1="4.93"  y1="19.07" x2="7.76"  y2="16.24"/>
            <line x1="16.24" y1="7.76"  x2="19.07" y2="4.93"/>
        </svg>
    }
}

#[component]
pub fn IconGlobe(#[prop(default = "icon-svg")] class: &'static str) -> impl IntoView {
    view! {
        <svg class=class viewBox="0 0 24 24" fill="none" stroke="currentColor"
             stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"
             xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
            <circle cx="12" cy="12" r="10"/>
            <line x1="2" y1="12" x2="22" y2="12"/>
            <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/>
        </svg>
    }
}

#[component]
pub fn IconGitHub(#[prop(default = "icon-svg")] class: &'static str) -> impl IntoView {
    view! {
        <svg class=class viewBox="0 0 24 24" fill="none" stroke="currentColor"
             stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"
             xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
            <path d="M9 19c-5 1.5-5-2.5-7-3m14 6v-3.87a3.37 3.37 0 0 0-.94-2.61c3.14-.35 6.44-1.54 6.44-7A5.44 5.44 0 0 0 20 4.77 5.07 5.07 0 0 0 19.91 1S18.73.65 16 2.48a13.38 13.38 0 0 0-7 0C6.27.65 5.09 1 5.09 1A5.07 5.07 0 0 0 5 4.77a5.44 5.44 0 0 0-1.5 3.78c0 5.42 3.3 6.61 6.44 7A3.37 3.37 0 0 0 9 18.13V22"/>
        </svg>
    }
}

#[component]
pub fn IconZap(#[prop(default = "icon-svg")] class: &'static str) -> impl IntoView {
    view! {
        <svg class=class viewBox="0 0 24 24" fill="none" stroke="currentColor"
             stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"
             xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
            <polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"/>
        </svg>
    }
}

#[component]
pub fn IconArrowRight(#[prop(default = "icon-svg")] class: &'static str) -> impl IntoView {
    view! {
        <svg class=class viewBox="0 0 24 24" fill="none" stroke="currentColor"
             stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
             xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
            <line x1="5" y1="12" x2="19" y2="12"/>
            <polyline points="12 5 19 12 12 19"/>
        </svg>
    }
}

#[component]
pub fn IconKey(#[prop(default = "icon-svg")] class: &'static str) -> impl IntoView {
    view! {
        <svg class=class viewBox="0 0 24 24" fill="none" stroke="currentColor"
             stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"
             xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
            <circle cx="7.5" cy="15.5" r="5.5"/>
            <path d="M21 2l-9.6 9.6"/>
            <path d="M15.5 7.5l3 3L22 7l-3-3"/>
        </svg>
    }
}

#[component]
pub fn IconCopy(#[prop(default = "icon-svg")] class: &'static str) -> impl IntoView {
    view! {
        <svg class=class viewBox="0 0 24 24" fill="none" stroke="currentColor"
             stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"
             xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
            <rect x="9" y="9" width="13" height="13" rx="2" ry="2"/>
            <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/>
        </svg>
    }
}

#[component]
pub fn IconTrash(#[prop(default = "icon-svg")] class: &'static str) -> impl IntoView {
    view! {
        <svg class=class viewBox="0 0 24 24" fill="none" stroke="currentColor"
             stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"
             xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
            <polyline points="3 6 5 6 21 6"/>
            <path d="M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6"/>
            <path d="M10 11v6"/>
            <path d="M14 11v6"/>
            <path d="M9 6V4a1 1 0 0 1 1-1h4a1 1 0 0 1 1 1v2"/>
        </svg>
    }
}

#[component]
pub fn IconPlus(#[prop(default = "icon-svg")] class: &'static str) -> impl IntoView {
    view! {
        <svg class=class viewBox="0 0 24 24" fill="none" stroke="currentColor"
             stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
             xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
            <line x1="12" y1="5" x2="12" y2="19"/>
            <line x1="5" y1="12" x2="19" y2="12"/>
        </svg>
    }
}

#[component]
pub fn IconAlertTriangle(#[prop(default = "icon-svg")] class: &'static str) -> impl IntoView {
    view! {
        <svg class=class viewBox="0 0 24 24" fill="none" stroke="currentColor"
             stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"
             xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
            <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/>
            <line x1="12" y1="9" x2="12" y2="13"/>
            <line x1="12" y1="17" x2="12.01" y2="17"/>
        </svg>
    }
}

#[component]
pub fn IconMenu(#[prop(default = "icon-svg")] class: &'static str) -> impl IntoView {
    view! {
        <svg class=class viewBox="0 0 24 24" fill="none" stroke="currentColor"
             stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
             xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
            <line x1="3" y1="6"  x2="21" y2="6"/>
            <line x1="3" y1="12" x2="21" y2="12"/>
            <line x1="3" y1="18" x2="21" y2="18"/>
        </svg>
    }
}

#[component]
pub fn IconLayers(#[prop(default = "icon-svg")] class: &'static str) -> impl IntoView {
    view! {
        <svg class=class viewBox="0 0 24 24" fill="none" stroke="currentColor"
             stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"
             xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
            <polygon points="12 2 2 7 12 12 22 7 12 2"/>
            <polyline points="2 17 12 22 22 17"/>
            <polyline points="2 12 12 17 22 12"/>
        </svg>
    }
}

#[component]
pub fn IconPackage(#[prop(default = "icon-svg")] class: &'static str) -> impl IntoView {
    view! {
        <svg class=class viewBox="0 0 24 24" fill="none" stroke="currentColor"
             stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"
             xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
            <path d="M16.5 9.4l-9-5.19"/>
            <path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/>
            <polyline points="3.27 6.96 12 12.01 20.73 6.96"/>
            <line x1="12" y1="22.08" x2="12" y2="12"/>
        </svg>
    }
}

#[component]
pub fn IconUpload(#[prop(default = "icon-svg")] class: &'static str) -> impl IntoView {
    view! {
        <svg class=class viewBox="0 0 24 24" fill="none" stroke="currentColor"
             stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"
             xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
            <polyline points="16 16 12 12 8 16"/>
            <line x1="12" y1="12" x2="12" y2="21"/>
            <path d="M20.39 18.39A5 5 0 0 0 18 9h-1.26A8 8 0 1 0 3 16.3"/>
        </svg>
    }
}

#[component]
pub fn IconChevronsRight(#[prop(default = "icon-svg")] class: &'static str) -> impl IntoView {
    view! {
        <svg class=class viewBox="0 0 24 24" fill="none" stroke="currentColor"
             stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
             xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
            <polyline points="13 17 18 12 13 7"/>
            <polyline points="6 17 11 12 6 7"/>
        </svg>
    }
}

#[component]
pub fn IconChevronsLeft(#[prop(default = "icon-svg")] class: &'static str) -> impl IntoView {
    view! {
        <svg class=class viewBox="0 0 24 24" fill="none" stroke="currentColor"
             stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
             xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
            <polyline points="11 17 6 12 11 7"/>
            <polyline points="18 17 13 12 18 7"/>
        </svg>
    }
}

#[component]
pub fn IconCloud(#[prop(default = "icon-svg")] class: &'static str) -> impl IntoView {
    view! {
        <svg class=class viewBox="0 0 24 24" fill="none" stroke="currentColor"
             stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"
             xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
            <path d="M18 10h-1.26A8 8 0 1 0 9 20h9a5 5 0 0 0 0-10z"/>
        </svg>
    }
}

#[component]
pub fn IconTrophy(#[prop(default = "icon-svg")] class: &'static str) -> impl IntoView {
    view! {
        <svg class=class viewBox="0 0 24 24" fill="none" stroke="currentColor"
             stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"
             xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
            <path d="M6 9H4.5a2.5 2.5 0 0 1 0-5H6"/>
            <path d="M18 9h1.5a2.5 2.5 0 0 0 0-5H18"/>
            <path d="M4 22h16"/>
            <path d="M10 14.66V17c0 .55-.47.98-.97 1.21C7.85 18.75 7 20.24 7 22"/>
            <path d="M14 14.66V17c0 .55.47.98.97 1.21C16.15 18.75 17 20.24 17 22"/>
            <path d="M18 2H6v7a6 6 0 0 0 12 0V2z"/>
        </svg>
    }
}

#[component]
pub fn IconActivity(#[prop(default = "icon-svg")] class: &'static str) -> impl IntoView {
    view! {
        <svg class=class viewBox="0 0 24 24" fill="none" stroke="currentColor"
             stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"
             xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
            <polyline points="22 12 18 12 15 21 9 3 6 12 2 12"/>
        </svg>
    }
}

#[component]
pub fn IconFileText(#[prop(default = "icon-svg")] class: &'static str) -> impl IntoView {
    view! {
        <svg class=class viewBox="0 0 24 24" fill="none" stroke="currentColor"
             stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"
             xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
            <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
            <polyline points="14 2 14 8 20 8"/>
            <line x1="16" y1="13" x2="8" y2="13"/>
            <line x1="16" y1="17" x2="8" y2="17"/>
            <polyline points="10 9 9 9 8 9"/>
        </svg>
    }
}

#[component]
pub fn IconClock(#[prop(default = "icon-svg")] class: &'static str) -> impl IntoView {
    view! {
        <svg class=class viewBox="0 0 24 24" fill="none" stroke="currentColor"
             stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"
             xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
            <circle cx="12" cy="12" r="10"/>
            <polyline points="12 6 12 12 16 14"/>
        </svg>
    }
}

/// Folder icon — R2 directory navigation
#[component]
pub fn IconFolder(#[prop(default = "icon-svg")] class: &'static str) -> impl IntoView {
    view! {
        <svg class=class viewBox="0 0 24 24" fill="none" stroke="currentColor"
             stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"
             xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
            <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
        </svg>
    }
}

/// Refresh / reload icon
#[component]
pub fn IconRefresh(#[prop(default = "icon-svg")] class: &'static str) -> impl IntoView {
    view! {
        <svg class=class viewBox="0 0 24 24" fill="none" stroke="currentColor"
             stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"
             xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
            <polyline points="23 4 23 10 17 10"/>
            <polyline points="1 20 1 14 7 14"/>
            <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"/>
        </svg>
    }
}

/// Database / permissions icon
#[component]
pub fn IconDatabase(#[prop(default = "icon-svg")] class: &'static str) -> impl IntoView {
    view! {
        <svg class=class viewBox="0 0 24 24" fill="none" stroke="currentColor"
             stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"
             xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
            <ellipse cx="12" cy="5" rx="9" ry="3"/>
            <path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3"/>
            <path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5"/>
        </svg>
    }
    }
