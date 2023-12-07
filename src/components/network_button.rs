use yew::{classes, function_component, html, AttrValue, Callback, Html, Properties};

#[derive(Properties, PartialEq)]
pub struct Props {
    pub network: AttrValue,
    pub onclick: Callback<AttrValue>,
}

#[function_component(NetworkButton)]
pub fn button(props: &Props) -> Html {
    let network = props.network.clone();

    let onclick = props.onclick.reform(move |_| network.clone());

    html! {
        <div class={classes!("network")}>

            <button {onclick} >{ props.network.clone() }</button>

        </div>
    }
}
