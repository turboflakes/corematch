use corematch_common::components::block::{Block, Corespace};
use corematch_common::components::core::Core;
use corematch_common::errors::CorematchError;
use corematch_common::runtimes::{
    support::SupportedRelayRuntime, utils::get_para_id_from_storage_key,
};

use futures::StreamExt;
use log::error;
use node_runtime::runtime_types::{
    polkadot_parachain_primitives::primitives::Id,
    polkadot_runtime_parachains::scheduler::common::Assignment,
    polkadot_runtime_parachains::scheduler::pallet::CoreOccupied,
};
use rand::Rng;
use std::time::Duration;
use subxt::{utils::H256, OnlineClient, PolkadotConfig};
use yew::{
    platform::{pinned::mpsc::UnboundedSender, spawn_local, time::sleep},
    AttrValue, Callback,
};

use corematch_common::types::network::{SubscriptionId, STOP_SIGNAL};

#[subxt::subxt(
    runtime_metadata_path = "artifacts/metadata/polkadot_metadata.scale",
    derive_for_all_types = "PartialEq, Clone"
)]
pub mod node_runtime {}

const SIX_SECS: Duration = Duration::from_secs(6);
const DEFAULT_TOTAL_CORES: u32 = 64;
const DEFAULT_TOTAL_BLOCKS: u32 = 9;

/// subscribes to finalized blocks, when a block is received, fetch storage for the block hash and send it via the callback.
pub async fn subscribe_to_finalized_blocks(
    api: OnlineClient<PolkadotConfig>,
    cb: Callback<(SubscriptionId, Block)>,
) -> Result<(SubscriptionId, UnboundedSender<AttrValue>), CorematchError> {
    // Create channel so that an unsubscribe signal could be received.
    let (tx, mut rx) = yew::platform::pinned::mpsc::unbounded::<AttrValue>();
    // Generate a unique subscription_id
    let mut rng = rand::thread_rng();
    let subscription_id = rng.gen::<u32>();

    spawn_local(async move {
        // set number of previous blocks to be fetched
        let mut previous_blocks_processed = Some(DEFAULT_TOTAL_BLOCKS);

        match api.blocks().subscribe_finalized().await {
            Ok(mut blocks_sub) => {
                while let Some(result) = blocks_sub.next().await {
                    // 1. verify if there is an unsubscribe signal pending to be processed
                    if let Some(signal) = rx.next().await {
                        if signal == AttrValue::from(STOP_SIGNAL) {
                            break;
                        }
                    }

                    // 2. initialize and process results
                    match result {
                        Ok(block) => {
                            // 2.1 fetch previous 16 blocks
                            // process older blocks that have not been processed first
                            while let Some(counter) = previous_blocks_processed {
                                if counter == 0 {
                                    previous_blocks_processed = None;
                                } else {
                                    let block_number = block.number() - counter;
                                    let bloc_hash_addr =
                                        node_runtime::storage().system().block_hash(block_number);

                                    match api.storage().at_latest().await {
                                        Ok(storage) => match storage.fetch(&bloc_hash_addr).await {
                                            Ok(Some(block_hash)) => {
                                                match fetch_corespace(&api, block_number, block_hash).await {
                                                    Ok(block) => {
                                                        cb.emit((
                                                            subscription_id,
                                                            block,
                                                        ));
                                                    }
                                                    Err(e) => error!("{}", e),
                                                }
                                            }
                                            Ok(None) => error!("Failed to fetch block_hash for block_number: {block_number}"),
                                            Err(e) => error!("{}", e),
                                        },
                                        Err(e) => error!("{}", e),
                                    }

                                    previous_blocks_processed = Some(counter - 1);
                                }
                            }

                            // 2.2 process latest block
                            match fetch_corespace(
                                &api,
                                block.number().clone(),
                                block.hash().clone(),
                            )
                            .await
                            {
                                Ok(block) => {
                                    cb.emit((subscription_id, block));
                                }
                                Err(e) => error!("{}", e),
                            }

                            // NOTE: pause task for six seconds to ensure that data is processed always at the same pace
                            sleep(SIX_SECS).await;
                        }
                        Err(e) => error!("{}", e),
                    }
                }
            }
            Err(e) => error!("{}", e),
        }
    });
    Ok((subscription_id, tx))
}

pub async fn fetch_corespace(
    api: &OnlineClient<PolkadotConfig>,
    block_number: u32,
    block_hash: H256,
) -> Result<Block, CorematchError> {
    // Fetch availability_cores
    let availability_cores_addr = node_runtime::storage()
        .para_scheduler()
        .availability_cores();

    let availability_cores_option = api
        .storage()
        .at(block_hash)
        .fetch(&availability_cores_addr)
        .await?;

    if let Some(availability_cores) = availability_cores_option {
        let mut corespace = availability_cores
            .iter()
            .enumerate()
            .map(|(i, core_occupied)| match core_occupied {
                CoreOccupied::Free => Core::new(i, None),
                CoreOccupied::Paras(paras_entry) => match &paras_entry.assignment {
                    Assignment::Pool {
                        para_id: Id(para_id),
                        core_index: _,
                    } => Core::new(i, Some(*para_id)),
                    Assignment::Bulk(Id(para_id)) => Core::new(i, Some(*para_id)),
                },
            })
            .collect::<Corespace>();

        // Note: keep only the predefined number of cores
        corespace.truncate(DEFAULT_TOTAL_CORES as usize);

        return Ok(Block::new(
            block_number.clone(),
            corespace.clone(),
            SupportedRelayRuntime::Polkadot,
        ));
    }
    Err(CorematchError::Other(
        format!("Failed to fetch availability_cores for block_hash: {block_hash}").into(),
    ))
}

pub async fn fetch_para_ids(api: OnlineClient<PolkadotConfig>) -> Result<Vec<u32>, CorematchError> {
    let mut para_ids: Vec<u32> = Vec::new();
    let address = node_runtime::storage().paras().para_lifecycles_iter();
    let mut iter = api.storage().at_latest().await?.iter(address).await?;

    while let Some(Ok(storage)) = iter.next().await {
        para_ids.push(get_para_id_from_storage_key(storage.key_bytes));
    }
    para_ids.sort();
    Ok(para_ids)
}
