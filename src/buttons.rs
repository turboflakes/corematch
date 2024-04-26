use crate::app::GameLevel;
use crate::block::BlockView;
use crate::router::{Query, Routes};
use crate::runtimes::support::SupportedRelayRuntime;
use gloo::timers::callback::Timeout;
use log::info;
use yew::{
    classes, function_component, html, use_state, AttrValue, Callback, Children, FocusEvent, Html,
    Properties,
};
use yew_hooks::use_clipboard;
use yew_router::prelude::use_navigator;

#[derive(Properties, PartialEq)]
pub struct ActionButtonProps {
    pub disable: bool,
    pub label: AttrValue,
    pub children: Children,
    pub onclick: Callback<()>,
}

#[function_component(ActionButton)]
pub fn button(props: &ActionButtonProps) -> Html {
    let onclick = props.onclick.reform(move |_| ());
    let disabled_class = if props.disable {
        Some("disabled")
    } else {
        None
    };

    html! {
        <div class={classes!("control")}>
            <button class={classes!("btn__link", disabled_class)} {onclick}>
                {props.children.clone()}<span class="label">{format!("{}", props.label.to_string())}</span>
            </button>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct IconButtonProps {
    pub disable: bool,
    pub children: Children,
    pub onclick: Callback<()>,
}

#[function_component(IconButton)]
pub fn button(props: &IconButtonProps) -> Html {
    let onclick = props.onclick.reform(move |_| ());
    let disabled_class = if props.disable {
        Some("disabled")
    } else {
        None
    };

    html! {
        <button disabled={disabled_class.is_some()} class={classes!("btn__icon", disabled_class)} {onclick} >
            {props.children.clone()}
        </button>
    }
}

#[derive(Properties, PartialEq)]
pub struct ShareButtonProps {
    pub label: AttrValue,
    pub data: AttrValue,
    pub children: Children,
}

#[function_component(ShareButton)]
pub fn button(props: &ShareButtonProps) -> Html {
    let optional_class = use_state(|| Some("hidden"));
    let timeout = use_state(|| None);
    let clipboard = use_clipboard();

    let onclick = {
        let clipboard = clipboard.clone();
        let optional_class = optional_class.clone();
        let data = props.data.clone();
        let timeout = timeout.clone();

        Callback::from(move |_| {
            let hidden_class = optional_class.clone();
            let handle = Timeout::new(3_000, move || hidden_class.set(Some("hidden")));
            timeout.set(Some(handle));
            clipboard.write_text(data.to_string());
            optional_class.set(Some("visible"))
        })
    };

    html! {
        <div class="share">
            <ActionButton label={props.label.clone()} disable={false} {onclick} >{props.children.clone()}</ActionButton>
            <span class={classes!("clipboard__message", *optional_class)}>{"score copied to clipboard"}</span>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct MintButtonProps {
    pub label: AttrValue,
    pub children: Children,
    pub onclick: Callback<()>,
}

#[function_component(MintButton)]
pub fn button(props: &MintButtonProps) -> Html {
    let onclick = props.onclick.reform(move |_| ());

    // TODO: MintButton is disabled for the moment
    html! {
        <ActionButton label={props.label.clone()} disable={true} {onclick} >{props.children.clone()}</ActionButton>
    }
}

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

#[derive(Properties, PartialEq)]
pub struct BlockViewProps {
    pub view: BlockView,
    pub disable: bool,
    pub children: Children,
    pub onclick: Callback<BlockView>,
}

#[function_component(BlockViewButton)]
pub fn button(props: &BlockViewProps) -> Html {
    let view = props.view.clone();
    let onclick = props.onclick.reform(move |_| view.clone());

    html! {
        <IconButton disable={props.disable.clone()} {onclick}>
            {props.children.clone()}
        </IconButton>
    }
}

#[derive(Properties, PartialEq)]
pub struct LevelProps {
    pub level: GameLevel,
    pub disable: bool,
    pub children: Children,
    pub onclick: Callback<GameLevel>,
}

#[function_component(LevelButton)]
pub fn button(props: &LevelProps) -> Html {
    let level = props.level.clone();
    let onclick = props.onclick.reform(move |_| level.clone());

    html! {
        <IconButton disable={props.disable.clone()} {onclick}>
            {props.children.clone()}
        </IconButton>
    }
}

#[derive(Properties, PartialEq)]
pub struct TextProps {
    pub label: AttrValue,
    pub onclick: Callback<()>,
}

#[function_component(TextButton)]
pub fn button(props: &TextProps) -> Html {
    let onclick = props.onclick.reform(move |_| ());

    html! {
        <div class={classes!("btn-link")} {onclick}>{format!("{}", props.label.to_string())}</div>
    }
}
