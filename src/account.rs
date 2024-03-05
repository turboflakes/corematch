use crate::block::Block;
use crate::runtimes::support::SupportedParachainRuntime;
use crate::subscription_provider::SubscriptionId;
use js_sys::Promise;
use log::{error, info};
use serde::{Deserialize, Serialize};
use subxt::utils::AccountId32;
use subxt::{
    ext::codec::{Compact, Encode},
    utils::H256,
    OnlineClient, PolkadotConfig,
};
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen_futures::JsFuture;
use yew::{
    classes, function_component, html, use_state, AttrValue, Callback, Component, Context,
    ContextProvider, Html, MouseEvent, Properties, ToHtml,
};

use anyhow::anyhow;
use serde_json::json;
use subxt::utils::Era;

#[derive(Clone, PartialEq)]
pub enum AccountStatus {
    None,
    // Requesting access to load accounts from browser extension
    Requesting,
    // Waiting user to select which account to use
    Selection(Vec<Account>),
    // Account has been selected
    Selected,
    // Account available and ready to sign results
    Signing(AttrValue),
}

#[derive(Clone, PartialEq, Debug)]
pub enum SigningStatus {
    Succeeded,
    Failed,
}

/// AccountState is a shared state between all components.
#[derive(Clone, PartialEq)]
pub struct AccountState {
    /// The status of the account.
    pub status: AccountStatus,
    /// The account selected.
    pub account: Option<Account>,
    // A callback to handle accounts loaded from browser extension.
    pub accounts_callback: Callback<Vec<Account>>,
    /// A parachain runtime supported by the App.
    pub runtime: SupportedParachainRuntime,
    // // A runtime callback to handle data subscribed by the runtime.
    // pub runtime_callback: Callback<(SubscriptionId, Block)>,
    // A callback to handle stored results.
    pub signing_callback: Callback<SigningStatus>,
}

impl AccountState {
    pub fn new(
        runtime: SupportedParachainRuntime,
        accounts_callback: Callback<Vec<Account>>,
        signing_callback: Callback<SigningStatus>,
    ) -> Self {
        Self {
            status: AccountStatus::None,
            account: None,
            runtime,
            accounts_callback,
            signing_callback,
        }
    }

    pub fn is_none(&self) -> bool {
        self.status == AccountStatus::None
    }

    pub fn is_available(&self) -> bool {
        self.status == AccountStatus::Selected
    }
}

/// Account holds info needed to communicate with browser extension
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Account {
    /// account name
    pub name: String,
    /// name of the browser extension
    pub source: String,
    /// the signature type, e.g. "sr25519" or "ed25519"
    pub ty: String,
    /// ss58 formatted address as string. Can be converted into AccountId32 via it's FromStr implementation.
    pub address: String,
}

impl Account {
    pub fn render(&self, onclick: Callback<()>) -> Html {
        html! { <AccountComponent account={self.clone()} {onclick} /> }
    }
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub account: Account,
    pub onclick: Callback<()>,
}

#[function_component(AccountComponent)]
pub fn account(props: &Props) -> Html {
    let onclick = props.onclick.reform(move |_| ());

    html! {
        <div class={classes!("control")}>
            <div class={classes!("btn-link")} {onclick} >{ props.account.name.clone() }</div>
        </div>
    }
}

// Import functionality from JS
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = getAccounts)]
    pub fn js_get_accounts() -> Promise;
    #[wasm_bindgen(js_name = signPayload)]
    pub fn js_sign_payload(payload: String, source: String, address: String) -> Promise;
}

// fetch PJS accounts
pub async fn get_accounts() -> Result<Vec<Account>, anyhow::Error> {
    let result = JsFuture::from(js_get_accounts())
        .await
        .map_err(|js_err| anyhow!("{js_err:?}"))?;
    let accounts_str = result
        .as_string()
        .ok_or(anyhow!("Error converting JsValue into String"))?;
    let accounts: Vec<Account> = serde_json::from_str(&accounts_str)?;
    Ok(accounts)
}

fn to_hex(bytes: impl AsRef<[u8]>) -> String {
    format!("0x{}", hex::encode(bytes.as_ref()))
}

fn encode_then_hex<E: Encode>(input: &E) -> String {
    format!("0x{}", hex::encode(input.encode()))
}

/// The extension signature and flow taken from taken from,
/// https://github.com/paritytech/subxt/blob/master/examples/wasm-example/src/services.rs#L121
/// communicates with JavaScript to obtain a signature for the `partial_extrinsic` via a browser extension (e.g. polkadot-js or Talisman)
///
/// Some parameters are hard-coded here and not taken from the partial_extrinsic itself (mortality_checkpoint, era, tip).
pub async fn extension_signature_for_extrinsic(
    call_data: &[u8],
    api: &OnlineClient<PolkadotConfig>,
    account_nonce: u64,
    account_source: String,
    account_address: String,
) -> Result<Vec<u8>, anyhow::Error> {
    let genesis_hash = encode_then_hex(&api.genesis_hash());
    // These numbers aren't SCALE encoded; their bytes are just converted to hex:
    let spec_version = to_hex(&api.runtime_version().spec_version.to_be_bytes());
    let transaction_version = to_hex(&api.runtime_version().transaction_version.to_be_bytes());
    let nonce = to_hex(&account_nonce.to_be_bytes());
    // If you construct a mortal transaction, then this block hash needs to correspond
    // to the block number passed to `Era::mortal()`.
    let mortality_checkpoint = encode_then_hex(&api.genesis_hash());
    let era = encode_then_hex(&Era::Immortal);
    let method = to_hex(call_data);
    let signed_extensions: Vec<String> = api
        .metadata()
        .extrinsic()
        .signed_extensions()
        .iter()
        .map(|e| e.identifier().to_string())
        .collect();
    let tip = encode_then_hex(&Compact(0u128));

    let payload = json!({
        "specVersion": spec_version,
        "transactionVersion": transaction_version,
        "address": account_address,
        "blockHash": mortality_checkpoint,
        "blockNumber": "0x00000000",
        "era": era,
        "genesisHash": genesis_hash,
        "method": method,
        "nonce": nonce,
        "signedExtensions": signed_extensions,
        "tip": tip,
        "version": 4,
    });

    let payload = payload.to_string();
    let result = JsFuture::from(js_sign_payload(payload, account_source, account_address))
        .await
        .map_err(|js_err| anyhow!("{js_err:?}"))?;
    let signature = result
        .as_string()
        .ok_or(anyhow!("Error converting JsValue into String"))?;
    let signature = hex::decode(&signature[2..])?;
    Ok(signature)
}
