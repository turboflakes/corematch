use crate::block::{Block, Corespace};
use crate::core::Core;
use futures::StreamExt;
use log::error;
use node_runtime::runtime_types::{
    polkadot_parachain_primitives::primitives::Id, polkadot_primitives::v5::CoreOccupied,
};
use std::time::Duration;
use subxt::{OnlineClient, PolkadotConfig};
use yew::{
    platform::{pinned::mpsc::UnboundedSender, spawn_local, time::sleep},
    AttrValue, Callback,
};

use crate::subscription_provider::STOP_SIGNAL;

#[subxt::subxt(
    runtime_metadata_path = "metadata/kusama_metadata.scale",
    derive_for_all_types = "PartialEq, Clone"
)]
pub mod node_runtime {}

const SIX_SECS: Duration = Duration::from_secs(6);

/// subscribes to finalized blocks, when a block is received, fetch storage for the block hash and send it via the callback.
pub async fn subscribe_to_finalized_blocks(
    api: OnlineClient<PolkadotConfig>,
    cb: Callback<Block>,
) -> Result<UnboundedSender<AttrValue>, subxt::Error> {
    // Create channel so that an unsubscribe signal could be received.
    let (tx, mut rx) = yew::platform::pinned::mpsc::unbounded::<AttrValue>();

    spawn_local(async move {
        match api.blocks().subscribe_finalized().await {
            Ok(mut blocks_sub) => {
                while let Some(result) = blocks_sub.next().await {
                    // 1st verify if there is an unsubscribe signal pending to be processed
                    if let Some(signal) = rx.next().await {
                        if signal == AttrValue::from(STOP_SIGNAL) {
                            break;
                        }
                    }

                    // 2nd process result
                    match result {
                        Ok(block) => {
                            // Fetch availability_cores
                            let availability_cores_addr = node_runtime::storage()
                                .para_scheduler()
                                .availability_cores();

                            match api
                                .storage()
                                .at(block.hash())
                                .fetch(&availability_cores_addr)
                                .await
                            {
                                Ok(availability_cores_option) => {
                                    if let Some(availability_cores) = availability_cores_option {
                                        let corespace = availability_cores
                                            .iter()
                                            .enumerate()
                                            .map(|(i, core_occupied)| match core_occupied {
                                                CoreOccupied::Free => Core::new(i, None),
                                                CoreOccupied::Paras(paras_entry) => {
                                                    let Id(para_id) =
                                                        paras_entry.assignment.para_id;
                                                    Core::new(i, Some(para_id))
                                                }
                                            })
                                            .collect::<Corespace>();

                                        cb.emit(Block::new(
                                            block.number().clone(),
                                            corespace.clone(),
                                        ));
                                    }
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
    Ok(tx)
}
