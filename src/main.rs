mod account;
// mod account_provider;
mod app;
mod block;
mod block_timer;
mod buttons;
mod core;
mod keyboard;
mod network;
mod pages;
mod router;
mod runtimes;
mod subscription_provider;
mod views;
use crate::router::Router;

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<Router>::new().render();
}
