use crate::account::{
    extension_signature_for_extrinsic, get_accounts, Account, AccountState, AccountStatus,
    SigningStatus,
};
use crate::block::{Block, Corespace};
use crate::buttons::TextButton;
use crate::network::NetworkState;
use crate::runtimes::asset_hub_polkadot::node_runtime::runtime_types::sp_core::ecdsa::Signature;
use crate::runtimes::support::{SupportedParachainRuntime, SupportedRelayRuntime};
use anyhow::anyhow;
use futures::FutureExt;
use log::{error, info};
use std::{rc::Rc, str::FromStr};
use subxt::{
    ext::codec::Decode,
    tx::{SubmittableExtrinsic, TxPayload},
    utils::{AccountId32, MultiSignature},
    OnlineClient, PolkadotConfig,
};
use yew::{
    html, platform::pinned::mpsc::UnboundedSender, AttrValue, Callback, Children, Component,
    Context, ContextHandle, Html, Properties,
};

use crate::runtimes::asset_hub_polkadot;

pub type CallData = Vec<u8>;

pub enum Msg {
    Error(anyhow::Error),
    OnlineClientCreated(OnlineClient<PolkadotConfig>),
    SelectButtonClicked,
    AccountsLoaded(Vec<Account>),
    SignatureReceived(
        MultiSignature,
        SubmittableExtrinsic<PolkadotConfig, OnlineClient<PolkadotConfig>>,
    ),
    SignatureCancelled(anyhow::Error),
    ContextChanged(Rc<AccountState>),
}

pub struct AccountProvider {
    state: Rc<AccountState>,
    _listener: ContextHandle<Rc<AccountState>>,
    online_client: Option<OnlineClient<PolkadotConfig>>,
    error: Option<AttrValue>,
}

impl Component for AccountProvider {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (state, _listener) = ctx
            .link()
            .context::<Rc<AccountState>>(ctx.link().callback(Msg::ContextChanged))
            .expect("context to be set");

        let runtime = SupportedParachainRuntime::from(state.runtime.clone());

        ctx.link().send_future(OnlineClient::<PolkadotConfig>::from_url(runtime.default_rpc_url()).map(|result| {
            match result {
                Ok(online_client) => Msg::OnlineClientCreated(online_client),
                Err(err) => Msg::Error(anyhow!("RPC connection could not be established, make sure RPC endpoint is valid:\n{err}")),
            }
        }));

        Self {
            state,
            _listener,
            online_client: None,
            error: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Error(err) => {
                self.error = Some(err.to_string().into());
                error!("{}", err);
                true
            }
            Msg::OnlineClientCreated(online_client) => {
                info!("OnlineClientCreated");
                self.online_client = Some(online_client);

                // verify if account has been already selected from PJS
                // TODO: verify from localstorage
                // info!("__{:?}", self.state.account);
                // if self.state.account.is_none() {
                //     info!("is_none");
                //     ctx.link()
                //         .send_future(get_accounts().map(|accounts_or_err| match accounts_or_err {
                //             Ok(accounts) => Msg::AccountsLoaded(accounts),
                //             Err(err) => Msg::Error(err),
                //         }));
                // }
                true
            }
            Msg::SelectButtonClicked => {
                ctx.link().send_future(get_accounts().map(
                    |accounts_or_err| match accounts_or_err {
                        Ok(accounts) => Msg::AccountsLoaded(accounts),
                        Err(err) => Msg::Error(err),
                    },
                ));

                true
            }
            Msg::ContextChanged(state) => {
                info!("ContextChanged");
                if state.runtime != self.state.runtime {
                    // Create a new online client
                    ctx.link().send_future(OnlineClient::<PolkadotConfig>::from_url(state.runtime.default_rpc_url()).map(|result| {
                        match result {
                            Ok(online_client) => Msg::OnlineClientCreated(online_client),
                            Err(err) => Msg::Error(anyhow!("RPC connection could not be established, make sure RPC endpoint is valid:\n{err}")),
                        }
                    }));
                }

                let api = self.online_client.as_ref().unwrap().clone();

                match &state.status {
                    AccountStatus::Signing(results) => {
                        let results = results.clone();
                        if let Some(account) = &state.account {
                            let account_address = account.address.clone();
                            let account_source = account.source.clone();
                            if let Ok(account_id) = account.address.parse() {
                                let _state = state.clone();
                                ctx.link().send_future(async move {
                                    let Ok(account_nonce) =
                                        api.tx().account_nonce(&account_id).await
                                    else {
                                        return Msg::Error(anyhow!(
                                            "Fetching account nonce failed"
                                        ));
                                    };

                                    let Ok(payload) = (match &_state.runtime {
                                        SupportedParachainRuntime::AssetHubPolkadot => {
                                            asset_hub_polkadot::prepare_payload(
                                                api.clone(),
                                                account_id.clone(),
                                                results.into(),
                                            )
                                            .await
                                        }
                                        _ => todo!(),
                                    }) else {
                                        return Msg::Error(anyhow!("Fetching payload failed"));
                                    };

                                    let Ok(call_data) = api.tx().call_data(&payload) else {
                                        return Msg::Error(anyhow!("Could not encode call data"));
                                    };

                                    let signature = match extension_signature_for_extrinsic(
                                        &call_data,
                                        &api,
                                        account_nonce,
                                        account_source,
                                        account_address,
                                    )
                                    .await
                                    {
                                        Ok(signature) => signature,
                                        Err(err) => return Msg::SignatureCancelled(err),
                                    };

                                    let Ok(multi_signature) =
                                        MultiSignature::decode(&mut &signature[..])
                                    else {
                                        return Msg::Error(anyhow!("MultiSignature Decoding"));
                                    };

                                    let Ok(partial_signed) =
                                        api.tx().create_partial_signed_with_nonce(
                                            &payload,
                                            account_nonce,
                                            Default::default(),
                                        )
                                    else {
                                        return Msg::Error(anyhow!(
                                            "PartialExtrinsic creation failed"
                                        ));
                                    };

                                    // Apply the signature
                                    let signed_extrinsic = partial_signed
                                        .sign_with_address_and_signature(
                                            &account_id.into(),
                                            &multi_signature,
                                        );

                                    return Msg::SignatureReceived(
                                        multi_signature,
                                        signed_extrinsic,
                                    );
                                })
                            }
                        }
                    }
                    _ => (),
                }
                self.state = state;
                true
            }
            Msg::AccountsLoaded(accounts) => {
                // send accounts back to be handle by another app component
                self.state.accounts_callback.emit(accounts);

                true
            }
            Msg::SignatureReceived(signature, signed_extrinsic) => {
                info!("__ReceivedSignature");
                // if let SigningStage::Signing(account) = &self.stage {
                //     let signed_extrinsic_hex =
                //         format!("0x{}", hex::encode(signed_extrinsic.encoded()));
                //     self.stage = SigningStage::SigningSuccess {
                //         signer_account: account.clone(),
                //         signature,
                //         signed_extrinsic_hex,
                //         submitting_stage: SubmittingStage::Initial { signed_extrinsic },
                //     }
                // }
                true
            }
            Msg::SignatureCancelled(err) => {
                error!("{}", err);
                self.state.signing_callback.emit(SigningStatus::Failed);

                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let select_click = ctx.link().callback(move |_| Msg::SelectButtonClicked);
        html! {
            <div>
                {
                    if self.state.account.is_none() {
                        html! { <TextButton label="select account" onclick={select_click} /> }
                    } else {
                        html! { }
                    }
                }
            </div>
        }
    }
}
