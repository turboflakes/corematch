use yew::{classes, function_component, html, AttrValue, Callback, Html, Properties};

pub type Index = usize;
pub type ParaId = u32;

#[derive(Clone, PartialEq)]
pub enum CoreView {
    Binary,
    Multi,
}

impl CoreView {
    fn class(&self, para_id: Option<ParaId>) -> Option<String> {
        match self {
            Self::Binary => {
                if para_id.is_some() {
                    Some("core--1".to_string())
                } else {
                    Some("core--0".to_string())
                }
            }
            Self::Multi => {
                if let Some(para_id) = para_id {
                    Some(format!("para--{0}", para_id))
                } else {
                    None
                }
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Core {
    pub index: Index,
    pub para_id: Option<ParaId>,
}

impl Core {
    pub fn new(index: usize, para_id: Option<ParaId>) -> Self {
        Self { index, para_id }
    }

    pub fn render(&self, view: CoreView) -> Html {
        html! { <CoreComponent class={view.class(self.para_id.clone())} /> }
    }
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub class: Option<String>,
}

#[function_component(CoreComponent)]
pub fn core(props: &Props) -> Html {
    html! {
        <div class={classes!("core", props.class.clone())} />
    }
}
