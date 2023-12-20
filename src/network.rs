use crate::block::Block;
use crate::runtimes::support::SupportedRuntime;
use crate::subscription_provider::SubscriptionId;
use yew::Callback;

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
    pub status: NetworkStatus,
    /// Counter to keep track of subscriptions.
    pub subscription_id: Option<SubscriptionId>,
    // A subscription callback to handle subscription changes.
    pub subscription_callback: Callback<SubscriptionId>,
    /// A runtime supported by the App.
    pub runtime: SupportedRuntime,
    // A runtime callback to handle data subscribed by the runtime.
    pub runtime_callback: Callback<(SubscriptionId, Block)>,
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
