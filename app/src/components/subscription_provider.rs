use anyhow::anyhow;
use corematch_common::components::block::Block;
use corematch_common::runtimes::support::SupportedRelayRuntime;
use corematch_common::types::network::{
    NetworkState, ParachainIds, SubscriptionId, CONTINUE_SIGNAL, STOP_SIGNAL,
};
use futures::FutureExt;
use log::{error, info};
use std::rc::Rc;
use subxt::{OnlineClient, PolkadotConfig};
use yew::{
    html, platform::pinned::mpsc::UnboundedSender, AttrValue, Callback, Children, Component,
    Context, ContextHandle, Html, Properties,
};

use corematch_kusama::kusama;
use corematch_polkadot::polkadot;

pub enum Msg {
    Error(anyhow::Error),
    OnlineClientCreated(OnlineClient<PolkadotConfig>),
    OnlineClientDataReceived((SubscriptionId, Block)),
    SubscriptionCreated((SubscriptionId, UnboundedSender<AttrValue>)),
    ParachainsCollected(ParachainIds),
    ContextChanged(Rc<NetworkState>),
}

#[derive(Properties, PartialEq)]
pub struct Props {
    #[prop_or_default]
    pub children: Children,
}

pub struct SubscriptionProvider {
    state: Rc<NetworkState>,
    _listener: ContextHandle<Rc<NetworkState>>,
    online_client: Option<OnlineClient<PolkadotConfig>>,
    error: Option<AttrValue>,
    subscription_channel: Option<UnboundedSender<AttrValue>>,
}

impl Component for SubscriptionProvider {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let (state, _listener) = ctx
            .link()
            .context::<Rc<NetworkState>>(ctx.link().callback(Msg::ContextChanged))
            .expect("context to be set");

        // TODO: test light client support
        // let (lightclient, polkadot_rpc) = LightClient::relay_chain(POLKADOT_SPEC).unwrap();
        // ctx.link().send_future(OnlineClient::<PolkadotConfig>::from_rpc_client(polkadot_rpc).map(|result| {
        //     match result {
        //         Ok(online_client) => Msg::OnlineClientCreated(online_client),
        //         Err(err) => Msg::Error(anyhow!("RPC connection could not be established, make sure RPC endpoint is valid:\n{err}")),
        //     }
        // }));

        let runtime = SupportedRelayRuntime::from(state.runtime.clone());
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
            subscription_channel: None,
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
                self.online_client = Some(online_client);

                // Fetch parachains
                let api = self.online_client.as_ref().unwrap().clone();

                match self.state.runtime {
                    SupportedRelayRuntime::Polkadot => {
                        ctx.link()
                            .send_future(polkadot::fetch_para_ids(api).map(|result| match result {
                                Ok(para_ids) => Msg::ParachainsCollected(para_ids),
                                Err(err) => Msg::Error(err.into()),
                            }))
                    }
                    SupportedRelayRuntime::Kusama => ctx.link().send_future(
                        kusama::fetch_para_ids(api).map(|result| match result {
                            Ok(para_ids) => Msg::ParachainsCollected(para_ids),
                            Err(err) => Msg::Error(err.into()),
                        }),
                    ),
                    // _ => unimplemented!(),
                }

                // Subscribe blocks
                let cb: Callback<(SubscriptionId, Block)> =
                    ctx.link().callback(Msg::OnlineClientDataReceived);
                let api = self.online_client.as_ref().unwrap().clone();

                match self.state.runtime {
                    SupportedRelayRuntime::Polkadot => ctx.link().send_future(
                        polkadot::subscribe_to_finalized_blocks(api, cb).map(
                            |result| match result {
                                Ok((subscription_id, subscription_channel)) => {
                                    Msg::SubscriptionCreated((
                                        subscription_id,
                                        subscription_channel,
                                    ))
                                }
                                Err(err) => Msg::Error(err.into()),
                            },
                        ),
                    ),
                    SupportedRelayRuntime::Kusama => {
                        ctx.link()
                            .send_future(kusama::subscribe_to_finalized_blocks(api, cb).map(
                                |result| match result {
                                    Ok((subscription_id, subscription_channel)) => {
                                        Msg::SubscriptionCreated((
                                            subscription_id,
                                            subscription_channel,
                                        ))
                                    }
                                    Err(err) => Msg::Error(err.into()),
                                },
                            ))
                    } // _ => unimplemented!(),
                };
                true
            }
            Msg::ParachainsCollected(para_ids) => {
                // send parachains to be processed by the app
                self.state.parachains_callback.emit(para_ids);
                true
            }
            Msg::SubscriptionCreated((subscription_id, subscription_channel)) => {
                self.subscription_channel = Some(subscription_channel);

                // send subscription_id to be updated by the app
                self.state.subscription_callback.emit(subscription_id);

                if let Some(subscription_channel) = &self.subscription_channel {
                    subscription_channel
                        .send_now(CONTINUE_SIGNAL.into())
                        .expect("failed to send signal");
                }

                true
            }
            Msg::OnlineClientDataReceived((subscription_id, block)) => {
                if let Some(subscription_channel) = &self.subscription_channel {
                    subscription_channel
                        .send_now(CONTINUE_SIGNAL.into())
                        .expect("failed to send signal");
                }

                // send block to be processed by the app
                self.state.runtime_callback.emit((subscription_id, block));

                true
            }
            Msg::ContextChanged(state) => {
                info!("ContextChanged");
                if state.runtime != self.state.runtime {
                    // Send a signal to the subscription task to drop subscription.
                    if let Some(subscription_channel) = &self.subscription_channel {
                        subscription_channel
                            .send_now(STOP_SIGNAL.into())
                            .expect("failed to send signal");
                    }
                    // Create a new online client
                    ctx.link().send_future(OnlineClient::<PolkadotConfig>::from_url(state.runtime.default_rpc_url()).map(|result| {
                        match result {
                            Ok(online_client) => Msg::OnlineClientCreated(online_client),
                            Err(err) => Msg::Error(anyhow!("RPC connection could not be established, make sure RPC endpoint is valid:\n{err}")),
                        }
                    }));
                }
                self.state = state;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {{ ctx.props().children.clone() }}
    }
}
