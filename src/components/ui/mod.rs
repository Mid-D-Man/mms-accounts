use leptos::prelude::*;
use crate::components::icons::IconLoader;

// ── Button ─────────────────────────────────────────────────────

#[derive(Clone, PartialEq, Default)]
pub enum ButtonVariant {
    #[default]
    Primary,
    Ghost,
    Danger,
}

#[component]
pub fn Button(
    #[prop(default = ButtonVariant::Primary)] variant: ButtonVariant,
    #[prop(default = false)]                  loading:  bool,
    #[prop(default = false)]                  disabled: bool,
    #[prop(default = "button")]               btn_type: &'static str,
    #[prop(default = String::new())]          class:    String,
    children: Children,
) -> impl IntoView {
    let base = "btn";
    let var_class = match variant {
        ButtonVariant::Primary => "btn-primary",
        ButtonVariant::Ghost   => "btn-ghost",
        ButtonVariant::Danger  => "btn-danger",
    };
    let full_class = format!("{} {} {}", base, var_class, class);
    let is_disabled = loading || disabled;

    view! {
        <button
            type=btn_type
            class=full_class
            disabled=is_disabled
        >
            {move || if loading {
                view! {
                    <IconLoader class="icon-svg spin" />
                }.into_any()
            } else {
                view! { <span></span> }.into_any()
            }}
            {children()}
        </button>
    }
}

// ── Input ──────────────────────────────────────────────────────

#[component]
pub fn Input(
    #[prop(default = "text")]        input_type: &'static str,
    #[prop(default = String::new())] placeholder: String,
    #[prop(default = String::new())] value:       String,
    #[prop(default = false)]         required:    bool,
    #[prop(default = String::new())] label:       String,
    #[prop(default = String::new())] error:       String,
    on_input: impl Fn(String) + 'static,
) -> impl IntoView {
    let has_error  = !error.is_empty();
    let has_label  = !label.is_empty();
    let input_class = if has_error {
        "form-input form-input--error"
    } else {
        "form-input"
    };

    view! {
        <div class="form-group">
            {if has_label {
                view! {
                    <label class="form-label">{label.clone()}</label>
                }.into_any()
            } else {
                view! { <span></span> }.into_any()
            }}
            <input
                type=input_type
                class=input_class
                placeholder=placeholder
                prop:value=value
                required=required
                on:input=move |ev| on_input(event_target_value(&ev))
            />
            {if has_error {
                view! {
                    <span class="form-error">{error.clone()}</span>
                }.into_any()
            } else {
                view! { <span></span> }.into_any()
            }}
        </div>
    }
}

// ── Card ───────────────────────────────────────────────────────

#[component]
pub fn Card(
    #[prop(default = String::new())] class: String,
    children: Children,
) -> impl IntoView {
    let full_class = format!("card {}", class);
    view! {
        <div class=full_class>
            {children()}
        </div>
    }
}

// ── Badge ──────────────────────────────────────────────────────

#[derive(Clone, PartialEq)]
pub enum BadgeVariant {
    Default,
    Admin,
    Success,
    Warning,
    Error,
}

#[component]
pub fn Badge(
    #[prop(default = BadgeVariant::Default)] variant: BadgeVariant,
    children: Children,
) -> impl IntoView {
    let class = match variant {
        BadgeVariant::Default => "badge",
        BadgeVariant::Admin   => "badge badge--admin",
        BadgeVariant::Success => "badge badge--success",
        BadgeVariant::Warning => "badge badge--warning",
        BadgeVariant::Error   => "badge badge--error",
    };
    view! {
        <span class=class>{children()}</span>
    }
}

// ── Spinner ────────────────────────────────────────────────────

#[component]
pub fn Spinner(
    #[prop(default = "spinner")] class: &'static str,
) -> impl IntoView {
    view! {
        <div class=class>
            <IconLoader class="icon-svg spin" />
        </div>
    }
}

// ── Status message ─────────────────────────────────────────────

#[derive(Clone, PartialEq)]
pub enum StatusKind {
    Success,
    Error,
    Info,
}

#[component]
pub fn StatusMsg(
    kind:    StatusKind,
    message: String,
) -> impl IntoView {
    if message.is_empty() {
        return view! { <div></div> }.into_any();
    }

    let class = match kind {
        StatusKind::Success => "status-msg status-msg--success",
        StatusKind::Error   => "status-msg status-msg--error",
        StatusKind::Info    => "status-msg status-msg--info",
    };

    view! {
        <div class=class>
            {message}
        </div>
    }.into_any()
      }
