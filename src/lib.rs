use wasm_bindgen::prelude::*;

pub mod app;
pub mod components;
pub mod supabase;

use app::App;

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    web_sys::console::log_1(&"MmS Accounts: Starting...".into());
    leptos::mount::mount_to_body(App);
}
