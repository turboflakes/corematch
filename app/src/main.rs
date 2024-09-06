mod app;
mod components;
mod pages;
mod router;
use crate::router::Router;

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<Router>::new().render();
}
