use leptos::prelude::*;
use crate::components::icons::{IconShield, IconUser, IconGlobe, IconZap};

struct Feature {
    title:       &'static str,
    description: &'static str,
    icon_type:   &'static str,
}

#[component]
pub fn FeaturesSection() -> impl IntoView {
    let features = vec![
        Feature {
            title:       "Single Sign-On",
            description: "One account for every MidManStudio product. Sign in once and stay authenticated across games, tools, and services.",
            icon_type:   "user",
        },
        Feature {
            title:       "Secure by Default",
            description: "Powered by Supabase Auth. Your credentials are hashed, sessions are JWT-based, and nothing sensitive is stored client-side.",
            icon_type:   "shield",
        },
        Feature {
            title:       "Studio Identity",
            description: "Build your MmS profile — display name, bio, and links. Your identity travels with you across every MidManStudio experience.",
            icon_type:   "globe",
        },
        Feature {
            title:       "Always Free",
            description: "MmS Accounts is free for all players and studio members. No subscriptions, no paywalls on your own identity.",
            icon_type:   "zap",
        },
    ];

    view! {
        <section class="features-section">
            <div class="features-inner">
                <div class="features-header">
                    <h2 class="features-title">
                        "Built for the"
                        <em>" MmS Ecosystem"</em>
                    </h2>
                    <p class="features-sub">
                        "Everything you need to manage your studio identity in one place."
                    </p>
                </div>

                <div class="features-grid">
                    {features.into_iter().map(|f| view! {
                        <div class="feature-card">
                            <div class="feature-icon-wrap">
                                {match f.icon_type {
                                    "shield" => view! { <IconShield class="icon-svg" /> }.into_any(),
                                    "globe"  => view! { <IconGlobe  class="icon-svg" /> }.into_any(),
                                    "zap"    => view! { <IconZap    class="icon-svg" /> }.into_any(),
                                    _        => view! { <IconUser   class="icon-svg" /> }.into_any(),
                                }}
                            </div>
                            <h3 class="feature-title">{f.title}</h3>
                            <p class="feature-desc">{f.description}</p>
                        </div>
                    }).collect_view()}
                </div>
            </div>
        </section>
    }
      }
