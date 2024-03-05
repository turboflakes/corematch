use crate::app::App;

mod account;
mod account_provider;
mod app;
mod block;
mod block_timer;
mod buttons;
mod core;
mod keyboard;
mod network;
mod runtimes;
mod subscription_provider;
mod views;

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::new().render();
}
