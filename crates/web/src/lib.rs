#![forbid(unsafe_code)]

pub mod app;

pub mod auth;

pub mod plan_form;

pub mod plans;

pub mod plan_ui;

#[cfg(feature = "ssr")]
pub mod mail;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(app::App);
}
