use crate::app::GameLevel;
use crate::{
    core::{Core, CoreView, NaCoreComponent},
    runtimes::support::SupportedRelayRuntime,
};
use log::{error, info};
use std::str::FromStr;
use subxt::utils::H256;
use yew::{
    classes, function_component, html, use_state, AttrValue, Callback, Component, Context,
    ContextProvider, Html, MouseEvent, Properties, ToHtml,
};

pub type Corespace = Vec<Core>;
pub type Index = usize;
pub type BlockNumber = u32;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Status {
    Matched,
    Selected,
    None,
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

impl ToHtml for BlockView {
    fn to_html(&self) -> Html {
        html! { self.to_owned() }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Block {
    pub block_number: BlockNumber,
    pub corespace: Corespace,
    pub runtime: SupportedRelayRuntime,
    pub selected_class: Option<String>,
    pub disable_class: Option<String>,
    pub missed_class: Option<String>,
    pub matched_class: Option<String>,
    pub help_class: Option<String>,
    pub anim_class: Option<String>,
    pub is_selected: bool,
    pub is_flipped: bool,
}

impl Block {
    pub fn new(
        block_number: BlockNumber,
        corespace: Corespace,
        runtime: SupportedRelayRuntime,
    ) -> Self {
        Self {
            block_number,
            corespace,
            runtime,
            selected_class: None,
            disable_class: None,
            missed_class: None,
            matched_class: None,
            help_class: None,
            anim_class: None,
            is_selected: false,
            is_flipped: false,
        }
    }

    pub fn selected(&mut self) {
        self.selected_class = Some("highlight".to_string());
    }

    pub fn unselected(&mut self) {
        self.selected_class = None;
    }

    pub fn is_selected(&self) -> bool {
        self.selected_class.is_some()
    }

    pub fn matched(&mut self) {
        self.matched_class = Some("matched".to_string());
        self.help_class = None;
    }

    pub fn is_matched(&self) -> bool {
        self.matched_class.is_some()
    }

    pub fn disabled(&mut self) {
        self.matched_class = None;
        self.disable_class = Some("disabled".to_string());
    }

    pub fn is_disabled(&self) -> bool {
        self.disable_class.is_some()
    }

    pub fn missed(&mut self) {
        self.missed_class = Some("missed".to_string());
    }

    pub fn flipped(&mut self) {
        self.anim_class = Some("anim".to_string());
        self.is_flipped = !self.is_flipped;
    }

    pub fn is_anim_live(&self) -> bool {
        self.anim_class.is_some()
    }

    pub fn cleared(&mut self) {
        self.missed_class = None;
        self.help_class = None;
        self.anim_class = None;
    }

    pub fn is_help_available(&self) -> bool {
        self.matched_class.is_none()
    }

    pub fn help(&mut self) {
        self.help_class = Some("help".to_string());
    }

    pub fn network_class(&self) -> String {
        self.runtime.to_string().to_lowercase()
    }

    pub fn inline_style(&self) -> String {
        match self.runtime {
            SupportedRelayRuntime::Polkadot => format!(
                "background-color: rgba(230, 0, 122, {});",
                self.corespace_usage() as f32 / 100.0
            ),
            SupportedRelayRuntime::Kusama => format!(
                "background-color: rgba(0, 0, 0, {});",
                self.corespace_usage() as f32 / 100.0
            ),
        }
    }

    pub fn reset(&mut self) {
        self.reset_class();
    }

    pub fn reset_class(&mut self) {
        self.selected_class = None;
        self.disable_class = None;
        self.missed_class = None;
        self.matched_class = None;
        self.help_class = None;
    }

    pub fn classes(&self) -> String {
        classes!(
            self.network_class(),
            self.selected_class.clone(),
            self.disable_class.clone(),
            self.missed_class.clone(),
            self.matched_class.clone(),
            self.help_class.clone(),
            self.anim_class.clone()
        )
        .to_string()
    }

    pub fn corespace_hash(&self, game_level: GameLevel) -> H256 {
        let data: Vec<u8> = match game_level {
            // GameLevel::Level0 => (self.corespace_usage() as u32).to_le_bytes().to_vec(),
            GameLevel::Level1 => self
                .corespace
                .iter()
                .map(|core| {
                    if core.para_id.is_some() {
                        0x01u8
                    } else {
                        0x00u8
                    }
                })
                .collect::<Vec<u8>>(),
            GameLevel::Level2 => self
                .corespace
                .iter()
                .map(|core| {
                    if let Some(para_id) = core.para_id {
                        para_id.to_le_bytes()
                    } else {
                        0x00u32.to_le_bytes()
                    }
                })
                .flatten()
                .collect::<Vec<u8>>(),
        };
        let hash = sp_core_hashing::blake2_256(&data[..]);
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

    pub fn corespace_ascii(&self) -> String {
        self.corespace
            .iter()
            .enumerate()
            .map(|(i, core)| {
                let mut char = if core.para_id.is_some() {
                    "■".to_string()
                } else {
                    "□".to_string()
                };
                if (i as u32 + 1) % self.runtime.columns_size() == 0 {
                    char.push_str("\n");
                }
                char
            })
            .collect::<String>()
    }

    pub fn render(
        &self,
        core_view: CoreView,
        onclick: Callback<()>,
        ondblclick: Callback<()>,
        onanimationend: Callback<BlockNumber>,
    ) -> Html {
        html! { <BlockComponent block={self.clone()} {core_view} {onclick} {ondblclick} {onanimationend} /> }
    }
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub block: Block,
    pub core_view: CoreView,
    pub onclick: Callback<()>,
    pub ondblclick: Callback<()>,
    pub onanimationend: Callback<BlockNumber>,
}

#[function_component(BlockComponent)]
pub fn block(props: &Props) -> Html {
    let onclick = props.onclick.reform(move |_| ());
    let ondblclick = props.ondblclick.reform(move |_| ());
    let block_number = props.block.block_number.clone();
    let onanimationend = props.onanimationend.reform(move |_| block_number.clone());

    let not_available_cores_counter = (props.block.runtime.columns_size()
        * props.block.runtime.columns_size())
        - props.block.corespace.len() as u32;
    let not_available_vec = vec![0; not_available_cores_counter.try_into().unwrap()];

    html! {
        <div class={classes!("corespace", props.block.classes())}
            {onclick} {ondblclick} {onanimationend}>
            {
                if !props.block.is_flipped {
                    html! {
                        <div class={classes!("cores")}>
                            { for props.block.corespace.iter().map(|c| c.render(props.core_view.clone())) }
                            { for not_available_vec.iter().map(|_| html! { <NaCoreComponent /> } ) }
                        </div>
                    }
                } else {
                    let core_usage = format!(
                        "{}%",
                        props.block.corespace_usage()
                    );
                    let block_number = format!(
                        "#{}",
                        props.block.block_number.clone(),
                    );
                    html! {
                        <div class={classes!("palette")}>
                            <span class="label">{ "usage" }</span>
                            <span class="details">{ core_usage }</span>
                            <span class="label">{ "finalized block" }</span>
                            <span class="details">{ block_number }</span>
                        </div>
                    }
                }
            }
        </div>
    }
}
