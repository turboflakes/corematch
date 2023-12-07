use log::info;
use std::collections::BTreeMap;
use std::rc::Rc;
use yew::{
    classes, function_component, html, AttrValue, Callback, Component, Context, ContextProvider,
    Html, Properties,
};

use crate::components::{
    network_button::NetworkButton, subscription_provider::SubscriptionProvider,
};
use crate::runtimes::support::SupportedRuntime;
use crate::NetworkState;


pub type ParaId = AttrValue;
pub type Corespace = Vec<Option<ParaId>>;

pub enum Msg {
    NetworkDataReceived(Corespace),
    NetworkButtonClicked(AttrValue),
    // CorespaceClicked(usize),
    // StartButtonClicked,
    // HelpButtonClicked,
    // StopButtonClicked,
    // GameFinished,
}

pub struct App {
    network_state: Rc<NetworkState>,
    data: Vec<Option<Corespace>>,
    // match_key: Option<String>,
    // matches: BTreeMap<String, u32>,
    // duration: u32,
    // points: u32,
    // tries: u32,
    // is_game_on: bool,
    // is_help_on: bool,
    // is_help_disabled: bool,
    // help_duration: u32,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        // Set default runtime as Polkadot
        let runtime = SupportedRuntime::Polkadot;
        let runtime_callback = ctx.link().callback(Msg::NetworkDataReceived);
        // Initialized shared state
        let network_state = Rc::new(NetworkState {
            runtime: runtime.clone(),
            runtime_callback,
        });
        
        Self {
            network_state,
            data: vec![None; 16],
        }
    }
    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::NetworkDataReceived(data) => {
                info!("NetworkDataReceived: {:?}", data);
            }
            Msg::NetworkButtonClicked(network) => {
                let network_state = Rc::make_mut(&mut self.network_state);
                network_state.runtime = SupportedRuntime::from(network);
            }
        }
        true
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        let network_state = self.network_state.clone();
        let network_onclick = ctx.link().callback(move |e| Msg::NetworkButtonClicked(e));
        html! {
            <ContextProvider<Rc<NetworkState>> context={ network_state.clone() }>
                <SubscriptionProvider>
                    { match network_state.runtime {
                        SupportedRuntime::Polkadot => html! {
                            <NetworkButton network="kusama" onclick={network_onclick.clone()} />
                        },
                        SupportedRuntime::Kusama => html! {
                            <NetworkButton network="polkadot" onclick={network_onclick.clone()} />
                        }
                    }}
                </SubscriptionProvider>

                <div>{ "network_state" }</div>
            </ContextProvider<Rc<NetworkState>>>
        }
    }
}