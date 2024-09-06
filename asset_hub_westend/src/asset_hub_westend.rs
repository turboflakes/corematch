use crate::block::{Block, Corespace};
use crate::core::Core;
use crate::runtimes::{support::SupportedRelayRuntime, utils::get_nft_id_from_storage_key};
use anyhow::anyhow;
use futures::StreamExt;
use log::{error, info};
use node_runtime::{
    runtime_types::{
        bounded_collections::bounded_vec::BoundedVec,
        pallet_nfts::types::AttributeNamespace::Account,
    },
    utility::calls::types::Batch,
};
use rand::Rng;
use std::str::FromStr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use subxt::{
    tx::Payload,
    utils::{AccountId32, MultiAddress, H256},
    OnlineClient, PolkadotConfig,
};
use yew::{
    platform::{pinned::mpsc::UnboundedSender, spawn_local, time::sleep},
    AttrValue, Callback,
};

type Call = node_runtime::runtime_types::asset_hub_westend_runtime::RuntimeCall;
type NftsCall = node_runtime::runtime_types::pallet_nfts::pallet::Call;

type NftId = u32;

#[subxt::subxt(
    runtime_metadata_path = "artifacts/metadata/asset_hub_westend_metadata.scale",
    derive_for_all_types = "PartialEq, Clone"
)]
pub mod node_runtime {}

const COLLECTION_ID: u32 = 38;

pub async fn prepare_payload(
    api: OnlineClient<PolkadotConfig>,
    account: AccountId32,
    results: AttrValue,
) -> Result<Payload<Batch>, subxt::Error> {
    //
    let (nft_id, mint) = fetch_or_generate_nft_id(&api, account.clone()).await?;

    // create a batch call to mint and store results as an item attribute
    let mut calls: Vec<Call> = vec![];

    if mint {
        let call = Call::Nfts(NftsCall::mint {
            collection: COLLECTION_ID,
            item: nft_id,
            mint_to: account.clone().into(),
            witness_data: None,
        });
        calls.push(call);
    }

    if let Some((value, key)) = results.rsplit_once('/') {
        let call = Call::Nfts(NftsCall::set_attribute {
            collection: COLLECTION_ID,
            maybe_item: Some(nft_id),
            namespace: Account(account.clone()),
            key: BoundedVec(key.as_bytes().to_vec()),
            value: BoundedVec(value.as_bytes().to_vec()),
        });
        calls.push(call);
    }

    if calls.len() > 0 {
        Ok(node_runtime::tx().utility().batch(calls))
    } else {
        Err(subxt::Error::Other("No calls to encode".to_string()))
    }
}

pub async fn prepare_call(
    api: OnlineClient<PolkadotConfig>,
    account: AccountId32,
    results: AttrValue,
) -> Result<Vec<u8>, subxt::Error> {
    //
    let (nft_id, mint) = fetch_or_generate_nft_id(&api, account.clone()).await?;

    // create a batch call to mint and store results as an item attribute
    let mut calls: Vec<Call> = vec![];

    if mint {
        let call = Call::Nfts(NftsCall::mint {
            collection: COLLECTION_ID,
            item: nft_id,
            mint_to: account.clone().into(),
            witness_data: None,
        });
        calls.push(call);
    }

    if let Some((value, key)) = results.rsplit_once('/') {
        let call = Call::Nfts(NftsCall::set_attribute {
            collection: COLLECTION_ID,
            maybe_item: Some(nft_id),
            namespace: Account(account.clone()),
            key: BoundedVec(key.as_bytes().to_vec()),
            value: BoundedVec(value.as_bytes().to_vec()),
        });
        calls.push(call);
    }

    if calls.len() > 0 {
        let call_payload = node_runtime::tx().utility().batch(calls);
        api.tx().call_data(&call_payload)
    } else {
        Err(subxt::Error::Other("No calls to encode".to_string()))
    }
}

pub async fn fetch_or_generate_nft_id(
    api: &OnlineClient<PolkadotConfig>,
    account: AccountId32,
) -> Result<(NftId, bool), subxt::Error> {
    // check in storage, if the account has already an NFT in the collection:
    let nfts_account_addr = node_runtime::storage()
        .nfts()
        .account_iter2(&account, COLLECTION_ID);
    let mut iter = api
        .storage()
        .at_latest()
        .await?
        .iter(nfts_account_addr)
        .await?;

    // return nft id or generate a new one
    if let Some(Ok((key, _))) = iter.next().await {
        Ok((get_nft_id_from_storage_key(key), false))
    } else {
        let mut rng = rand::thread_rng();
        Ok((rng.gen_range(100000..200000), true))
    }
}
