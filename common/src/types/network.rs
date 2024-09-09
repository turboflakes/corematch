use crate::components::block::Block;
use crate::runtimes::support::SupportedRelayRuntime;
use rand::Rng;
use std::collections::BTreeMap;
use yew::Callback;

pub type ParaId = u32;
// Color is define in HSL format
pub type Color = (u32, u32, u32);
pub type ParachainIds = Vec<ParaId>;
pub type ParachainColors = BTreeMap<ParaId, Color>;
pub type SubscriptionId = u32;
pub const STOP_SIGNAL: &str = "stop";
pub const CONTINUE_SIGNAL: &str = "continue";

#[derive(Debug, Clone, PartialEq)]
pub enum NetworkStatus {
    Initializing,
    Switching,
    Active,
    Inactive,
}

/// NetworkState is a shared state between all components.
#[derive(Debug, Clone, PartialEq)]
pub struct NetworkState {
    /// The status of the network.
    pub status: NetworkStatus,
    /// Counter to keep track of subscriptions.
    pub subscription_id: Option<SubscriptionId>,
    // A subscription callback to handle subscription changes.
    pub subscription_callback: Callback<SubscriptionId>,
    /// A runtime supported by the App.
    pub runtime: SupportedRelayRuntime,
    // A runtime callback to handle data subscribed by the runtime.
    pub runtime_callback: Callback<(SubscriptionId, Block)>,
    /// A map between parachain_id and color.
    pub parachain_colors: ParachainColors,
    // A parachains callback to handle data collected.
    pub parachains_callback: Callback<ParachainIds>,
}

impl NetworkState {
    pub fn new(
        runtime: SupportedRelayRuntime,
        runtime_callback: Callback<(SubscriptionId, Block)>,
        subscription_callback: Callback<SubscriptionId>,
        parachains_callback: Callback<ParachainIds>,
    ) -> Self {
        Self {
            status: NetworkStatus::Initializing,
            subscription_id: None,
            subscription_callback,
            runtime,
            runtime_callback,
            parachain_colors: BTreeMap::new(),
            parachains_callback,
        }
    }

    pub fn is_initializing(&self) -> bool {
        self.status == NetworkStatus::Initializing
    }

    pub fn is_active(&self) -> bool {
        self.status == NetworkStatus::Active
    }

    pub fn is_switching(&self) -> bool {
        self.status == NetworkStatus::Switching
    }

    pub fn is_valid(&self, id: SubscriptionId) -> bool {
        if let Some(subscription_id) = self.subscription_id {
            self.status == NetworkStatus::Active && subscription_id == id
        } else {
            false
        }
    }

    pub fn class(&self) -> String {
        self.runtime.to_string().to_lowercase()
    }
}

pub fn generate_parachain_colors(para_ids: ParachainIds) -> ParachainColors {
    let n: u32 = para_ids.len().try_into().unwrap();

    // generate colors
    let mut colors = Vec::<Color>::new();
    for i in 0..n {
        let hue = 360 / n * i as u32;
        colors.push((hue, 96_u32, 68_u32));
    }

    // pick a random color and assign it a para_id
    let mut rng = rand::thread_rng();
    para_ids
        .into_iter()
        .map(|para_id| {
            let i = rng.gen_range(0..colors.len());
            let color = colors.remove(i);
            (para_id, color)
        })
        .collect()

    // para_ids
    //     .into_iter()
    //     .enumerate()
    //     .map(|(i, para_id)| {
    //         let hue = 360 / n * i as u32;
    //         (para_id, (hue, 95_u32, 72_u32))
    //     })
    //     .collect()
}
