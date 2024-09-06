use yew::{classes, function_component, html, AttrValue, Html, Properties};

#[derive(Properties, PartialEq)]
pub struct ColumnInfoProps {
    pub max: u32,
    pub value: u32,
    pub title: AttrValue,
    pub class: Option<AttrValue>,
    pub position_class: Option<AttrValue>,
    pub box_class: Option<AttrValue>,
}

#[function_component(ColumnInfoView)]
pub fn view(props: &ColumnInfoProps) -> Html {
    let optional_class = props.class.clone();
    let position_class = props.position_class.clone();
    let title = format!("{} {}", props.value.clone(), props.title.clone());
    let mut attempts = Vec::new();
    let available = props.value.clone();
    let gone = props.max.clone() - props.value.clone();
    for _i in 0..available {
        attempts.push(true);
    }
    for _i in 0..gone {
        attempts.push(false);
    }

    html! {
        <div class={classes!("column__info", position_class, optional_class)} {title}>
            { for attempts.into_iter().map(|v| {
                let box_class = props.box_class.clone();
                if v { html! { <span class={classes!("box", box_class)} /> } }
                else { html! { <span class={classes!("box", "disabled")} /> } }
                })
            }
        </div>
    }
}
