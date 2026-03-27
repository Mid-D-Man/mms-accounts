use leptos::prelude::*;
use crate::components::icons::{IconArrowRight, IconZap};

#[component]
pub fn HeroSection() -> impl IntoView {
    view! {
        <section class="hero-section">
            <div class="hero-bg">
                <div class="hero-grid"></div>
                <div class="hero-orb orb-cobalt"></div>
                <div class="hero-orb orb-crimson"></div>
            </div>

            <div class="hero-content">
                <div class="hero-eyebrow">
                    <span class="eyebrow-dot"></span>
                    <span>"Your MidManStudio Identity"</span>
                </div>

                <h1 class="hero-headline">
                    <span class="headline-primary">"One Account."</span>
                    <br />
                    <span class="headline-accent">"Every MmS Product."</span>
                </h1>

                <p class="hero-description">
                    "MmS Accounts is your single identity across all MidManStudio "
                    "products — games, tools, and services. Sign in once, access everything."
                </p>

                <div class="hero-cta-row">
                    <a href="#auth" class="btn btn-primary">
                        <span>"Get Started"</span>
                        <IconArrowRight class="icon-svg icon-sm" />
                    </a>
                    <a href="https://github.com/mid-d-man" target="_blank"
                       rel="noopener noreferrer" class="btn btn-ghost">
                        "Learn More"
                    </a>
                </div>

                <div class="hero-badges">
                    <span class="hero-badge">
                        <IconZap class="icon-svg icon-xs" />
                        "Free forever"
                    </span>
                    <span class="hero-badge">"Secure"</span>
                    <span class="hero-badge">"Open source"</span>
                </div>
            </div>
        </section>
    }
  }
