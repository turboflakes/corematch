use crate::types::network::{ParaId, ParachainColors};
use yew::{classes, function_component, html, Html, Properties};
pub type Index = usize;

#[derive(Clone, PartialEq)]
pub enum CoreView {
    NotApplicable,
    Binary,
    Multi(ParachainColors),
}

impl CoreView {
    fn class(&self, para_id: Option<ParaId>) -> Option<String> {
        match self {
            Self::Binary => {
                if para_id.is_some() {
                    Some("core__1".to_string())
                } else {
                    Some("core__0".to_string())
                }
            }
            Self::Multi(_) => {
                if let Some(para_id) = para_id {
                    Some(format!("para__{0}", para_id))
                } else {
                    Some("core__0".to_string())
                }
            }
            _ => unimplemented!(),
        }
    }

    fn style(&self, para_id: Option<ParaId>) -> Option<String> {
        match self {
            Self::Binary => None,
            Self::Multi(parachain_colors) => {
                if let Some(para_id) = para_id {
                    if let Some(color) = parachain_colors.get(&para_id) {
                        return Some(format!(
                            "background-color: hsl({} {}% {}%);",
                            color.0, color.1, color.2
                        ));
                    }
                }
                None
            }
            _ => unimplemented!(),
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
        html! { <CoreComponent class={view.class(self.para_id.clone())} style={view.style(self.para_id.clone())} /> }
    }
}

#[derive(Properties, PartialEq)]
pub struct CoreComponentProps {
    pub class: Option<String>,
    pub style: Option<String>,
}

#[function_component(CoreComponent)]
pub fn core(props: &CoreComponentProps) -> Html {
    html! {
        <div class={classes!("core", props.class.clone())} style={classes!(props.style.clone())} />
    }
}

#[function_component(NaCoreComponent)]
pub fn core() -> Html {
    html! {
        <div class="core" />
    }
}
