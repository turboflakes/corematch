use serde::{Deserialize, Serialize};
use yew::AttrValue;

pub type ChainPrefix = u16;

pub const POLKADOT_SPEC: &str = include_str!("../../artifacts/chain_specs/polkadot.json");
pub const POLKADOT_PEOPLE_SPEC: &str =
    include_str!("../../artifacts/chain_specs/polkadot_people.json");
pub const KUSAMA_SPEC: &str = include_str!("../../artifacts/chain_specs/kusama.json");
pub const KUSAMA_PEOPLE_SPEC: &str = include_str!("../../artifacts/chain_specs/kusama_people.json");

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

    pub fn default_rpc_url(&self) -> &'static str {
        match &self {
            Self::Polkadot => "wss://rpc.ibp.network:443/polkadot",
            Self::Kusama => "wss://rpc.ibp.network:443/kusama",
        }
    }

    pub fn default_people_rpc_url(&self) -> &'static str {
        match &self {
            Self::Polkadot => "wss://sys.ibp.network:443/people-polkadot",
            Self::Kusama => "wss://sys.ibp.network:443/people-kusama",
        }
    }

    pub fn chain_specs(&self) -> &str {
        match &self {
            Self::Polkadot => POLKADOT_SPEC,
            Self::Kusama => KUSAMA_SPEC,
        }
    }

    pub fn chain_specs_people(&self) -> &str {
        match &self {
            Self::Polkadot => POLKADOT_PEOPLE_SPEC,
            Self::Kusama => KUSAMA_PEOPLE_SPEC,
        }
    }

    pub fn unit(&self) -> &'static str {
        match &self {
            Self::Polkadot => "DOT",
            Self::Kusama => "KSM",
        }
    }

    pub fn decimals(&self) -> u16 {
        match &self {
            Self::Polkadot => 10,
            Self::Kusama => 12,
        }
    }

    pub fn class(&self) -> String {
        self.to_string().to_lowercase()
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
