use yew::AttrValue;

pub type ChainPrefix = u16;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SupportedRuntime {
    Polkadot,
    Kusama,
    // Westend,
}

impl SupportedRuntime {
    pub fn _chain_prefix(&self) -> ChainPrefix {
        match &self {
            Self::Polkadot => 0,
            Self::Kusama => 2,
            // Self::Westend => 42,
        }
    }

    pub fn default_rpc_url(&self) -> String {
        match &self {
            Self::Polkadot => "wss://rpc.ibp.network/polkadot".to_string(),
            Self::Kusama => "wss://rpc.ibp.network/kusama".to_string(),
            // Self::Westend => "wss://rpc.ibp.network/westend".to_string(),
        }
    }
}

impl From<AttrValue> for SupportedRuntime {
    fn from(v: AttrValue) -> Self {
        match v.as_str() {
            "polkadot" => Self::Polkadot,
            "DOT" => Self::Polkadot,
            "kusama" => Self::Kusama,
            "KSM" => Self::Kusama,
            // "westend" => Self::Westend,
            // "WND" => Self::Westend,
            _ => unimplemented!("Chain prefix not supported"),
        }
    }
}

impl From<String> for SupportedRuntime {
    fn from(v: String) -> Self {
        match v.as_str() {
            "polkadot" => Self::Polkadot,
            "DOT" => Self::Polkadot,
            "kusama" => Self::Kusama,
            "KSM" => Self::Kusama,
            // "westend" => Self::Westend,
            // "WND" => Self::Westend,
            _ => unimplemented!("Chain prefix not supported"),
        }
    }
}

impl From<ChainPrefix> for SupportedRuntime {
    fn from(v: ChainPrefix) -> Self {
        match v {
            0 => Self::Polkadot,
            2 => Self::Kusama,
            // 42 => Self::Westend,
            _ => unimplemented!("Chain prefix not supported"),
        }
    }
}

impl std::fmt::Display for SupportedRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Polkadot => write!(f, "Polkadot"),
            Self::Kusama => write!(f, "Kusama"),
            // Self::Westend => write!(f, "Westend"),
        }
    }
}
