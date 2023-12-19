use crate::block::BlockView;
use crate::runtimes::support::SupportedRuntime;
use yew::{classes, function_component, html, use_state, AttrValue, Callback, Html, Properties};
use yew_hooks::use_clipboard;

#[derive(Properties, PartialEq)]
pub struct ActionButtonProps {
    pub disable: bool,
    pub label: AttrValue,
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
            <div class={classes!("btn-link", disabled_class)} {onclick} >{format!("{}", props.label.to_string())}</div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct ShareButtonProps {
    pub label: AttrValue,
    pub data: AttrValue,
}

#[function_component(ShareButton)]
pub fn button(props: &ShareButtonProps) -> Html {
    let visible_class = use_state(|| None);
    let clipboard = use_clipboard();
    let onclick = {
        let clipboard = clipboard.clone();
        let visible_class = visible_class.clone();
        let data = props.data.clone();
        Callback::from(move |_| {
            clipboard.write_text(data.to_string());
            visible_class.set(Some("visible"))
        })
    };

    html! {
        <div class="share">
            <ActionButton label={props.label.clone()} disable={false} {onclick} />
            <span class={classes!("action-msg", *visible_class)}>{"results copied to clipboard"}</span>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct NetworkButtonProps {
    pub switch_to: AttrValue,
    pub visible: bool,
    pub onclick: Callback<AttrValue>,
}

#[function_component(NetworkButton)]
pub fn button(props: &NetworkButtonProps) -> Html {
    if !props.visible {
        return html! {};
    }

    let switch_to = props.switch_to.clone();

    let onclick = props.onclick.reform(move |_| switch_to.clone());

    let label = format!("switch to {}", props.switch_to.clone());

    html! {
        <div class={classes!("btn-network")}>

            <div class={classes!("btn-link")} {onclick} >{ label }</div>

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

#[derive(Properties, PartialEq)]
pub struct InfoProps {
    pub label: AttrValue,
    pub onclick: Callback<()>,
}

#[function_component(InfoButton)]
pub fn button(props: &InfoProps) -> Html {
    let onclick = props.onclick.reform(move |_| ());

    html! {
        <div class={classes!("btn-link")} {onclick}>{format!("{}", props.label.to_string())}</div>
    }
}
