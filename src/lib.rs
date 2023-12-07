use app::{App, Corespace};
use runtimes::support::SupportedRuntime;
use wasm_bindgen::prelude::*;
use yew::Callback;

mod app;
mod components;
mod runtimes;

/// NetworkState is a shared state between all components.
#[derive(Clone, PartialEq)]
pub struct NetworkState {
    /// A runtime supported by the App.
    runtime: SupportedRuntime,
    // A runtime callback to handle data subscribed by the runtime.
    runtime_callback: Callback<Corespace>,
}

#[wasm_bindgen]
/// init and start component on given root html element
pub fn init_app(root: web_sys::Element) {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::with_root(root).render();
}
