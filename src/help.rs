use yew::{classes, function_component, html, AttrValue, Callback, Html, Properties};

#[derive(Properties, PartialEq)]
pub struct Props {
    pub is_game_on: bool,
    pub is_help_on: bool,
    pub onclick: Callback<()>,
}

#[function_component(HelpButton)]
pub fn button(props: &Props) -> Html {
    let onclick = props.onclick.reform(move |_| ());

    html! {
        <div class={classes!("control")}>
            if props.is_game_on {
                <button {onclick} >{"help"}</button>
            }

            if props.is_help_on {
                { "HELP ON"}
            }
        </div>
    }
}
