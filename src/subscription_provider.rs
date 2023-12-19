use crate::block::{Block, Corespace};
use crate::NetworkState;
use crate::SupportedRuntime;
use anyhow::anyhow;
use futures::FutureExt;
use log::{error, info};
use std::rc::Rc;
use subxt::{OnlineClient, PolkadotConfig};
use yew::{
    html, platform::pinned::mpsc::UnboundedSender, AttrValue, Callback, Children, Component,
    Context, ContextHandle, Html, Properties,
};

use crate::runtimes::{ polkadot};

pub const STOP_SIGNAL: &str = "stop";
pub const CONTINUE_SIGNAL: &str = "continue";

pub type SubscriptionId = u32;

pub enum Msg {
    Error(anyhow::Error),
    OnlineClientCreated(OnlineClient<PolkadotConfig>),
    OnlineClientDataReceived((SubscriptionId, Block)),
    SubscriptionCreated((SubscriptionId, UnboundedSender<AttrValue>)),
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

        let runtime = SupportedRuntime::from(state.runtime.clone());
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
                let cb: Callback<(SubscriptionId, Block)> =
                    ctx.link().callback(Msg::OnlineClientDataReceived);
                let api = self.online_client.as_ref().unwrap().clone();

                match self.state.runtime {
                    SupportedRuntime::Polkadot => ctx.link().send_future(
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
                    // SupportedRuntime::Kusama => {
                    //     ctx.link()
                    //         .send_future(kusama::subscribe_to_finalized_blocks(api, cb).map(
                    //             |result| match result {
                    //                 Ok((subscription_id, subscription_channel)) => {
                    //                     Msg::SubscriptionCreated((
                    //                         subscription_id,
                    //                         subscription_channel,
                    //                     ))
                    //                 }
                    //                 Err(err) => Msg::Error(err.into()),
                    //             },
                    //         ))
                    // } 
                    _ => unimplemented!(),
                };
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
                    let runtime = SupportedRuntime::from(state.runtime.clone());
                    ctx.link().send_future(OnlineClient::<PolkadotConfig>::from_url(runtime.default_rpc_url()).map(|result| {
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
        html! {
            <div>
                {ctx.props().children.clone()}
            </div>
        }
    }
}
