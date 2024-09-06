use crate::router::{Query, Routes};
use corematch_common::runtimes::support::SupportedRelayRuntime;
use gloo::timers::callback::Timeout;
use log::info;
use yew::{
    classes, function_component, html, use_state, AttrValue, Callback, Children, FocusEvent, Html,
    Properties,
};
use yew_hooks::use_clipboard;
use yew_router::prelude::use_navigator;

#[derive(Properties, PartialEq)]
pub struct NetworkButtonProps {
    pub switch_to_chain: SupportedRelayRuntime,
    pub class: Option<AttrValue>,
    pub children: Children,
}

#[function_component(NetworkButton)]
pub fn button(props: &NetworkButtonProps) -> Html {
    let optional_class = props.class.clone();
    let chain = props.switch_to_chain.clone();
    let navigator = use_navigator().unwrap();

    let onclick = Callback::from(move |_| {
        navigator
            .push_with_query(&Routes::Index, &Query { chain })
            .unwrap();
    });

    html! {
        <button class={classes!("btn__icon", optional_class)} {onclick} >
            {props.children.clone()}
        </button>
    }
}
