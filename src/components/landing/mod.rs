pub mod hero;
pub mod features;

pub use hero::*;
pub use features::*;

use leptos::prelude::*;

#[component]
pub fn LandingPage() -> impl IntoView {
    view! {
        <div class="landing-page">
            <LandingNav />
            <main>
                <HeroSection />
                <FeaturesSection />
            </main>
            <LandingFooter />
        </div>
    }
}

#[component]
fn LandingNav() -> impl IntoView {
    view! {
        <nav class="landing-nav">
            <div class="landing-nav-inner">
                <a href="#" class="landing-nav-brand">
                    <span class="brand-mms">"MmS"</span>
                    <span class="brand-accounts">"Accounts"</span>
                </a>
                <div class="landing-nav-actions">
                    <a href="#auth" class="btn btn-ghost btn-sm">"Sign In"</a>
                </div>
            </div>
        </nav>
    }
}

#[component]
fn LandingFooter() -> impl IntoView {
    view! {
        <footer class="landing-footer">
            <div class="landing-footer-inner">
                <div class="footer-brand">
                    <span class="brand-mms">"MmS"</span>
                    <span class="footer-sep">"/"</span>
                    <span class="footer-studio">"MidManStudio"</span>
                </div>
                <p class="footer-copy">
                    "© 2025 MidManStudio. Built with Rust + Leptos."
                </p>
            </div>
        </footer>
    }
      }
