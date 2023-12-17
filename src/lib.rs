use app::App;
use block::Block;
use runtimes::support::SupportedRuntime;
use subscription_provider::SubscriptionId;
use wasm_bindgen::prelude::*;
use yew::Callback;

mod app;
mod block;
mod buttons;
mod core;
mod runtimes;
mod subscription_provider;

#[derive(Clone, PartialEq)]
pub enum NetworkStatus {
    Initializing,
    Switching,
    Active,
    Inactive,
}

/// NetworkState is a shared state between all components.
#[derive(Clone, PartialEq)]
pub struct NetworkState {
    /// The status of the network.
    status: NetworkStatus,
    /// Counter to keep track of subscriptions.
    subscription_id: Option<SubscriptionId>,
    // A subscription callback to handle subscription changes.
    subscription_callback: Callback<SubscriptionId>,
    /// A runtime supported by the App.
    runtime: SupportedRuntime,
    // A runtime callback to handle data subscribed by the runtime.
    runtime_callback: Callback<(SubscriptionId, Block)>,
}

impl NetworkState {
    pub fn is_initializing(&self) -> bool {
        self.status == NetworkStatus::Initializing
    }

    pub fn is_active(&self) -> bool {
        self.status == NetworkStatus::Active
    }

    pub fn is_valid(&self, id: SubscriptionId) -> bool {
        if let Some(subscription_id) = self.subscription_id {
            self.status == NetworkStatus::Active && subscription_id == id
        } else {
            false
        }
    }
}

#[wasm_bindgen]
/// init and start component on given root html element
pub fn init_app(root: web_sys::Element) {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::with_root(root).render();
}
