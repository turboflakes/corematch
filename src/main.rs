use crate::app::App;

mod app;
mod block;
mod buttons;
mod core;
mod network;
mod runtimes;
mod subscription_provider;

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::new().render();
}