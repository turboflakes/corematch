use crate::block::BlockView;
use yew::{classes, function_component, html, AttrValue, Callback, Html, Properties};

#[derive(Properties, PartialEq)]
pub struct StartButtonProps {
    pub is_game_on: bool,
    pub onclick: Callback<()>,
}

#[function_component(StartButton)]
pub fn button(props: &StartButtonProps) -> Html {
    let onclick = props.onclick.reform(move |_| ());

    html! {
        <div class={classes!("control")}>
            if !props.is_game_on {
                <div class={classes!("btn-link")} {onclick} >{"start | ►"}</div>
            }
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct HelpButtonProps {
    pub is_game_on: bool,
    pub is_help_on: bool,
    pub duration: u32,
    pub onclick: Callback<()>,
}

#[function_component(HelpButton)]
pub fn button(props: &HelpButtonProps) -> Html {
    let onclick = props.onclick.reform(move |_| ());

    html! {
        <div class={classes!("control")}>
            if props.is_game_on && !props.is_help_on && props.duration > 0{
                <div class={classes!("btn-link")} {onclick} >{"help | ?"}</div>
            }
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct NetworkButtonProps {
    pub network: AttrValue,
    pub onclick: Callback<AttrValue>,
}

#[function_component(NetworkButton)]
pub fn button(props: &NetworkButtonProps) -> Html {
    let network = props.network.clone();

    let onclick = props.onclick.reform(move |_| network.clone());

    html! {
        <div class={classes!("network")}>

            <button {onclick} >{ props.network.clone() }</button>

        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct BlockViewProps {
    pub view: BlockView,
    pub selected: bool,
    pub onclick: Callback<BlockView>,
}

#[function_component(BlockViewButton)]
pub fn button(props: &BlockViewProps) -> Html {
    let view = props.view.clone();
    let selected_class = if props.selected {
        Some("selected")
    } else {
        None
    };
    let onclick = props.onclick.reform(move |_| view.clone());

    html! {
        <div class={classes!("btn-link", selected_class)} {onclick}>{ props.view.clone() }</div>
    }
}
