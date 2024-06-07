use serde::{Deserialize, Serialize};
use yew::AttrValue;

pub type ChainPrefix = u16;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Deserialize, Serialize)]
pub enum SupportedRelayRuntime {
    Polkadot,
    Kusama,
}

impl SupportedRelayRuntime {
    pub fn _chain_prefix(&self) -> ChainPrefix {
        match &self {
            Self::Polkadot => 0,
            Self::Kusama => 2,
        }
    }

    pub fn default_rpc_url(&self) -> String {
        match &self {
            Self::Polkadot => "wss://rpc.turboflakes.io:443/polkadot".to_string(),
            Self::Kusama => "rpc.turboflakes.io:443/kusama".to_string(),
        }
    }

    pub fn columns_size(&self) -> u32 {
        match &self {
            Self::Polkadot => 8,
            Self::Kusama => 8,
        }
    }

    pub fn hashtag(&self) -> String {
        match &self {
            Self::Polkadot => "@Polkadot #BuildOnPolkadot".to_string(),
            Self::Kusama => "@kusamanetwork #BuildOnKusama".to_string(),
        }
    }

    pub fn asset_hub_runtime(&self) -> SupportedParachainRuntime {
        match &self {
            Self::Polkadot => SupportedParachainRuntime::AssetHubPolkadot,
            Self::Kusama => SupportedParachainRuntime::AssetHubKusama,
        }
    }
}

impl From<AttrValue> for SupportedRelayRuntime {
    fn from(v: AttrValue) -> Self {
        match v.as_str() {
            "Polkadot" => Self::Polkadot,
            "polkadot" => Self::Polkadot,
            "DOT" => Self::Polkadot,
            "Kusama" => Self::Kusama,
            "kusama" => Self::Kusama,
            "KSM" => Self::Kusama,
            _ => unimplemented!("Chain prefix not supported"),
        }
    }
}

impl From<String> for SupportedRelayRuntime {
    fn from(v: String) -> Self {
        match v.as_str() {
            "Polkadot" => Self::Polkadot,
            "polkadot" => Self::Polkadot,
            "DOT" => Self::Polkadot,
            "Kusama" => Self::Kusama,
            "kusama" => Self::Kusama,
            "KSM" => Self::Kusama,
            _ => unimplemented!("Chain prefix not supported"),
        }
    }
}

impl From<ChainPrefix> for SupportedRelayRuntime {
    fn from(v: ChainPrefix) -> Self {
        match v {
            0 => Self::Polkadot,
            2 => Self::Kusama,
            _ => unimplemented!("Chain prefix not supported"),
        }
    }
}

impl std::fmt::Display for SupportedRelayRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Polkadot => write!(f, "Polkadot"),
            Self::Kusama => write!(f, "Kusama"),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SupportedParachainRuntime {
    AssetHubPolkadot,
    AssetHubKusama,
}

impl SupportedParachainRuntime {
    pub fn default_rpc_url(&self) -> String {
        match &self {
            Self::AssetHubPolkadot => "wss://sys.ibp.network/westmint".to_string(),
            Self::AssetHubKusama => "wss://sys.ibp.network/westmint".to_string(),
        }
    }
}
impl std::fmt::Display for SupportedParachainRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AssetHubPolkadot => write!(f, "AssetHub Polkadot"),
            Self::AssetHubKusama => write!(f, "AssetHub Polkadot"),
        }
    }
}
