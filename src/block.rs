use crate::core::{Core, CoreView};
use futures::io::Empty;
use log::{error, info};
use std::str::FromStr;
use subxt::utils::H256;
use yew::{
    classes, function_component, html, use_state, AttrValue, Callback, Component, Context,
    ContextProvider, Html, MouseEvent, Properties,
};

pub type Corespace = Vec<Core>;
pub type Index = usize;
pub type BlockNumber = u32;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Status {
    Matched,
    Selected,
    Revealed,
}

#[derive(Clone, PartialEq)]
pub enum BlockView {
    Cores,
    Palette,
}

impl std::fmt::Display for BlockView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cores => write!(f, "core usage"),
            Self::Palette => write!(f, "block palette"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Block {
    pub block_number: BlockNumber,
    pub corespace: Corespace,
    pub status: Status,
    pub missed_class: Option<String>,
    pub matched_class: Option<String>,
    pub help_class: Option<String>,
}

impl Block {
    pub fn new(block_number: BlockNumber, corespace: Corespace) -> Self {
        Self {
            block_number,
            corespace,
            status: Status::Revealed,
            missed_class: None,
            matched_class: None,
            help_class: None,
        }
    }

    pub fn clicked(&mut self) {
        self.status = match self.status {
            Status::Revealed => Status::Selected,
            Status::Selected => Status::Revealed,
            _ => return,
        };
    }

    pub fn matched(&mut self) {
        self.status = Status::Matched;
        self.matched_class = Some("matched".to_string());
    }

    pub fn missed(&mut self) {
        self.missed_class = Some("missed".to_string());
    }

    pub fn help(&mut self) {
        self.help_class = Some("help".to_string());
    }

    pub fn reset_class(&mut self) {
        self.missed_class = None;
        self.matched_class = None;
        self.help_class = None;
    }

    pub fn classes(&self) -> String {
        match self.status {
            Status::Revealed => "".to_string(),
            Status::Selected => "highlight".to_string(),
            Status::Matched => "disable".to_string(),
        }
    }

    pub fn is_help_available(&self) -> bool {
        self.status == Status::Revealed
    }

    pub fn is_matched(&self) -> bool {
        self.status == Status::Matched
    }

    pub fn corespace_hash(&self, view: CoreView) -> H256 {
        let data = self
            .corespace
            .iter()
            .map(|core| match view {
                CoreView::Binary => {
                    if core.para_id.is_some() {
                        "1".to_string()
                    } else {
                        "0".to_string()
                    }
                }
                CoreView::Multi => {
                    if let Some(para_id) = core.para_id {
                        para_id.to_string()
                    } else {
                        "0".to_string()
                    }
                }
            })
            .collect::<String>();
        let hash = sp_core_hashing::blake2_256(data.as_bytes());
        H256::from(&hash)
    }

    pub fn corespace_usage(&self) -> usize {
        let filled = self
            .corespace
            .iter()
            .filter(|&core| core.para_id.is_some())
            .count();

        filled * 100 / self.corespace.len()
    }

    pub fn render(
        &self,
        block_view: BlockView,
        core_view: CoreView,
        onclick: Callback<()>,
        onanimationend: Callback<BlockNumber>,
    ) -> Html {
        html! { <BlockComponent block={self.clone()} {block_view} {core_view} {onclick} {onanimationend} /> }
    }
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub block: Block,
    pub block_view: BlockView,
    pub core_view: CoreView,
    pub onclick: Callback<()>,
    pub onanimationend: Callback<BlockNumber>,
}

#[function_component(BlockComponent)]
pub fn block(props: &Props) -> Html {
    let onclick = props.onclick.reform(move |_| ());
    let block_number = props.block.block_number.clone();
    let onanimationend = props.onanimationend.reform(move |_| block_number.clone());

    html! {
        <div class={classes!("corespace", props.block.classes(),
            props.block.missed_class.clone(), props.block.matched_class.clone(),
            props.block.help_class.clone())}
            {onclick} {onanimationend}>
            {
                match props.block_view {
                    BlockView::Cores => {
                        html! { for props.block.corespace.iter().map(|c| c.render(props.core_view.clone())) }
                    }
                    BlockView::Palette => {
                        let label = format!(
                            "#{} / {}%",
                            props.block.block_number.clone(),
                            props.block.corespace_usage()
                        );
                        let inline_style = format!("background-color: rgba(230, 0, 122, {});", props.block.corespace_usage() as f32 / 100.0);
                        html! {
                            <div class="corespace-palette" style={inline_style}>
                                <span class="label">{ label }</span>
                            </div>
                        }
                    }
                }
            }
        </div>
    }
}
