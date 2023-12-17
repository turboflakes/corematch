use log::{debug, info};
use std::collections::BTreeMap;
use std::rc::Rc;
use std::time::Duration;
use subxt::utils::H256;
use yew::{
    classes, function_component, html, html::Scope, platform::time::sleep, AttrValue, Callback,
    Component, Context, ContextProvider, Html, Properties,
};

use crate::block::{Block, BlockNumber, BlockView, Corespace};
use crate::buttons::{BlockViewButton, HelpButton, NetworkButton, StartButton};
use crate::core::{Core, CoreView};
use crate::runtimes::support::SupportedRuntime;
use crate::subscription_provider::{SubscriptionId, SubscriptionProvider};
use crate::{NetworkState, NetworkStatus};

const SIX_SECS: Duration = Duration::from_secs(6);

#[derive(Clone, PartialEq)]
pub enum GameStatus {
    Standby,
    Resetting,
    On,
    Over,
}

pub enum Msg {
    NetworkSubscriptionCreated(SubscriptionId),
    NetworkDataReceived((SubscriptionId, Block)),
    NetworkButtonClicked(AttrValue),
    BlockClicked(usize),
    BlockMatched,
    BlockAnimationEnded(BlockNumber),
    StartButtonClicked,
    HelpButtonClicked,
    GameFinished,
    BlockViewClicked(BlockView),
}

type Index = usize;

pub struct App {
    network_state: Rc<NetworkState>,
    blocks: Vec<Option<Block>>,
    block_view: BlockView,
    core_view: CoreView,
    match_block: Option<Block>,
    matches: BTreeMap<H256, u32>,
    game_status: GameStatus,
    duration: u32,
    points: u32,
    tries: u32,
    is_help_on: bool,
    is_help_disabled: bool,
    help_duration: u32,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        // Set default runtime as Polkadot
        let runtime = SupportedRuntime::Polkadot;
        let runtime_callback = ctx.link().callback(Msg::NetworkDataReceived);
        let subscription_callback = ctx.link().callback(Msg::NetworkSubscriptionCreated);
        // Initialized shared state
        let network_state = Rc::new(NetworkState {
            status: NetworkStatus::Initializing,
            subscription_id: None,
            subscription_callback: subscription_callback.clone(),
            runtime: runtime.clone(),
            runtime_callback,
        });

        Self {
            network_state,
            blocks: vec![None; 16],
            block_view: BlockView::Cores,
            core_view: CoreView::Binary,
            match_block: None,
            matches: BTreeMap::new(),
            game_status: GameStatus::Standby,
            duration: 0,
            points: 0,
            tries: 6,
            is_help_on: false,
            is_help_disabled: false,
            help_duration: 16,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::NetworkButtonClicked(network) => {
                if self.network_state.is_active() {
                    let network_state = Rc::make_mut(&mut self.network_state);
                    network_state.status = NetworkStatus::Switching;
                    network_state.runtime = SupportedRuntime::from(network);
                }
            }
            Msg::NetworkSubscriptionCreated(subscription_id) => {
                let network_state = Rc::make_mut(&mut self.network_state);
                network_state.subscription_id = Some(subscription_id);
                network_state.status = NetworkStatus::Active;
                // apply a full reset
                self.full_reset();
            }
            Msg::NetworkDataReceived((subscription_id, block)) => {
                if self.network_state.is_valid(subscription_id) {
                    self.blocks.insert(0, Some(block.clone()));
                    let hash = block.corespace_hash(self.core_view.clone());
                    // add counter for key
                    self.matches
                        .entry(hash)
                        .and_modify(|m| *m += 1)
                        .or_insert(1);
                    // remove oldest corespace
                    if self.blocks.len() > 16 {
                        if let Some(opt) = self.blocks.pop() {
                            if let Some(block) = opt {
                                let hash = block.corespace_hash(self.core_view.clone());
                                // subtract counter from hash
                                self.matches.entry(hash.clone()).and_modify(|m| *m -= 1);
                                // remove if counter is zero
                                if let Some(counter) = self.matches.get(&hash) {
                                    if *counter == 0 {
                                        self.matches.remove(&hash);
                                    }
                                }
                                // set match block to node in case is the one being expired
                                if let Some(match_block) = &self.match_block {
                                    if match_block.corespace_hash(self.core_view.clone()) == hash {
                                        self.match_block = None;
                                    }
                                }
                            }
                        }
                    }
                    debug!("_matches {:?}", self.matches);
                    // update game stats if game is on
                    self.incr_duration();
                    // highlight matches if help is on
                    if self.is_help_on {
                        let matches: Vec<_> = Vec::from_iter(self.matches.iter())
                            .iter()
                            .filter(|(_, counter)| **counter > 1)
                            .map(|(hash, _)| **hash)
                            .collect();

                        for hash in matches.iter() {
                            for opt in self.blocks.iter_mut() {
                                if let Some(block) = opt {
                                    if *hash == block.corespace_hash(self.core_view.clone()) {
                                        info!("HELP");
                                        if block.is_help_available() {
                                            block.help();
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Msg::BlockClicked(i) => {
                if self.is_game_on() {
                    if let Some(opt) = self.blocks.get_mut(i) {
                        if let Some(block) = opt {
                            if block.is_matched() {
                                return false;
                            }
                            if let Some(match_block) = &self.match_block {
                                let corespace_hash = block.corespace_hash(self.core_view.clone());
                                if match_block.block_number == block.block_number {
                                    block.clicked();
                                    self.match_block = None;
                                } else if match_block.corespace_hash(self.core_view.clone())
                                    == corespace_hash
                                {
                                    info!("Congrats, you found a match!");
                                    // increase points and update corespace by an empty space
                                    block.matched();
                                    self.match_succeed();
                                    ctx.link().send_message(Msg::BlockMatched);
                                } else {
                                    block.missed();
                                    self.match_failed();
                                }
                            } else {
                                block.clicked();
                                self.match_block = Some(block.clone());
                            }
                        }
                    }
                }
            }
            Msg::BlockMatched => {
                // lookout for current index of the matched block
                if let Some(match_block) = &self.match_block {
                    if let Some(i) = self.blocks.iter().position(|opt| {
                        opt.clone().unwrap().block_number == match_block.block_number
                    }) {
                        if let Some(opt) = self.blocks.get_mut(i) {
                            if let Some(block) = opt {
                                self.match_block = None;
                                block.matched();
                            }
                        }
                    }
                }
            }
            Msg::BlockAnimationEnded(block_number) => {
                if let Some(i) = self
                    .blocks
                    .iter()
                    .position(|opt| opt.clone().unwrap().block_number == block_number)
                {
                    if let Some(block_option) = self.blocks.get_mut(i) {
                        if let Some(block) = block_option {
                            block.reset_class();
                        }
                    }
                }
            }
            Msg::StartButtonClicked => {
                self.start();
            }
            Msg::HelpButtonClicked => {
                self.start_help();
            }
            Msg::GameFinished => {
                info!("GameFinished");
                // reset the current selected match block
                if let Some(match_block) = &self.match_block {
                    if let Some(i) = self.blocks.iter().position(|opt| {
                        opt.clone().unwrap().block_number == match_block.block_number
                    }) {
                        if let Some(opt) = self.blocks.get_mut(i) {
                            if let Some(block) = opt {
                                self.match_block = None;
                                block.clicked();
                            }
                        }
                    }
                }
                // reset game counters
                self.reset();
                // show view to share results and restart the game
                // TODO
            }
            Msg::BlockViewClicked(view) => {
                self.block_view = view;
            }
        }
        true
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        let network_state = self.network_state.clone();
        html! {
            <ContextProvider<Rc<NetworkState>> context={ network_state.clone() }>

                if self.blocks.is_empty() {
                    <p>{"Loading..."}</p>
                }
                else { { self.game_view(ctx.link()) } }

            </ContextProvider<Rc<NetworkState>>>
        }
    }
}

impl App {
    fn game_view(&self, link: &Scope<Self>) -> Html {
        html! {
            <>
                <div class="content">
                    <div class="content-header">
                        <a class="logo-img" href="https://polkadot.network" target="_blank">
                            <img class="icon-img" src="/assets/Polkadot_Logo_Horizontal_Pink-Black.svg" alt="polkadot logo" />
                        </a>
                        { self.game_stats_view() }
                    </div>
                    <div class="content-menu">
                        { self.head_menu_left_view(link) }
                        { self.head_menu_right_view(link) }
                    </div>
                    <div class="content-body">
                        { self.game_controls_right_view(link) }
                        <div class="board">
                            { for self.blocks.iter().enumerate().map(|(i, block_option)| {
                                    if let Some(block) = block_option {
                                        let block_clicked = link.callback(move |_| Msg::BlockClicked(i.clone()));
                                        let block_animation_ended = link.callback(move |bn| Msg::BlockAnimationEnded(bn));
                                        block.render(self.block_view.clone(), self.core_view.clone(), block_clicked.clone(), block_animation_ended.clone())
                                    } else {
                                        html! { <div class="corespace empty"></div> }
                                    }
                                })
                            }
                        </div>
                    </div>
                </div>
                { self.footer_view() }
            </>
        }
    }

    fn head_menu_left_view(&self, link: &Scope<Self>) -> Html {
        let option_click = link.callback(move |e| Msg::BlockViewClicked(e));
        html! {
            <div class="block-view-options">
                <BlockViewButton view={BlockView::Cores}
                    selected={self.block_view == BlockView::Cores}
                    onclick={option_click.clone()} />
                <BlockViewButton view={BlockView::Palette}
                    selected={self.block_view == BlockView::Palette}
                    onclick={option_click.clone()} />
            </div>
        }
    }

    fn head_menu_right_view(&self, link: &Scope<Self>) -> Html {
        let network_state = self.network_state.clone();
        let network_onclick = link.callback(move |e| Msg::NetworkButtonClicked(e));
        let visible = self.network_state.is_active();

        html! {
            <SubscriptionProvider>
                { match network_state.runtime {
                    SupportedRuntime::Polkadot => html! {
                        <NetworkButton switch_to="kusama" {visible} onclick={network_onclick.clone()} />
                    },
                    SupportedRuntime::Kusama => html! {
                        <NetworkButton switch_to="polkadot" {visible} onclick={network_onclick.clone()} />
                    }
                }}
            </SubscriptionProvider>
        }
    }

    fn game_controls_right_view(&self, link: &Scope<Self>) -> Html {
        let start_onclick = link.callback(move |_| Msg::StartButtonClicked);
        let help_onclick = link.callback(move |_| Msg::HelpButtonClicked);
        html! {
            <div class="game-controls-right">
                <StartButton is_game_on={self.is_game_on()} onclick={start_onclick} />
                <HelpButton is_game_on={self.is_game_on()} is_help_on={self.is_help_on}
                    duration={self.help_duration} onclick={help_onclick} />
            </div>
        }
    }

    fn is_game_on_view(&self) -> Html {
        html! {
            if self.is_game_on() {
                <div class="game-on">{"GAME ON!"}</div>
            } else { }
        }
    }

    fn game_stats_view(&self) -> Html {
        let help_class = if self.is_help_on {
            Some("help-on")
        } else {
            None
        };

        html! {
            <table class="game-stats">
                <tr>
                    { if self.is_game_on() { html! { <th class="game-on" rowspan="2">{"It's ON!"}</th> } } else { html! {} } }

                    <th>{"Duration"}</th>
                    <th>{"Tries"}</th>
                    <th>{"Helps"}</th>
                    <th>{"Points"}</th>
                </tr>
                <tr>
                    // { if self.is_game_on() { html! { <td></td> } } else { html! {} } }
                    <td>{self.duration}</td>
                    <td>{self.tries}</td>
                    <td class={classes!(help_class)}>{self.help_duration}</td>
                    <td class="points">{self.points}</td>
                </tr>
            </table>
        }
    }

    fn footer_view(&self) -> Html {
        html! {
            <footer class="footer">
                <div class="footer-content">
                    <a class="logo" href="https://turboflakes.io" target="_blank">
                        <img class="icon-img" src="/assets/logo_mark_black_subtract_turboflakes_.svg" alt="turboflakes logo" />
                    </a>
                    <span>{"Corematch built by TurboFlakes © 2023 // Unstoppable by Polkadot"}</span>
                </div>

                <div class="footer-content">
                    <a class="logo" href="https://github.com/turboflakes/corematch" target="_blank">
                        <img class="icon-img" src="/assets/github.svg" alt="github logo" />
                    </a>
                </div>
            </footer>
        }
    }

    fn full_reset(&mut self) {
        self.reset();
        self.blocks = vec![None; 16];
        self.match_block = None;
        self.matches = BTreeMap::new();
    }

    fn reset(&mut self) {
        self.game_status = GameStatus::Standby;
        self.duration = 0;
        self.points = 0;
        self.tries = 6;
        self.is_help_on = false;
        self.is_help_disabled = false;
        self.help_duration = 16;
    }

    fn is_game_on(&self) -> bool {
        self.game_status == GameStatus::On
    }

    fn start(&mut self) {
        self.reset();
        self.game_status = GameStatus::On;
    }

    fn match_succeed(&mut self) {
        if self.is_game_on() {
            self.incr_points();
        }
    }

    fn match_failed(&mut self) {
        if self.is_game_on() {
            self.decr_tries();
        }
    }

    fn incr_points(&mut self) {
        if self.is_game_on() {
            self.points += 20;
        }
    }

    fn incr_duration(&mut self) {
        if self.is_game_on() {
            self.duration += 1;
            if self.is_help_on {
                self.decr_help_duration();
            }
        }
    }

    fn decr_tries(&mut self) {
        if self.is_game_on() && self.tries > 0 {
            self.tries -= 1;
            // terminate game when no tries left to be played
            if self.tries == 0 {
                self.game_status = GameStatus::Over;
            }
        }
    }

    fn start_help(&mut self) {
        if self.is_game_on() && !self.is_help_disabled {
            self.is_help_on = true;
        }
    }

    fn decr_help_duration(&mut self) {
        if self.is_help_on && self.help_duration > 0 {
            self.help_duration -= 1;
            // terminate game when no tries left to be played
            if self.help_duration == 0 {
                self.is_help_on = false;
                self.is_help_disabled = true;
            }
        }
    }
}
