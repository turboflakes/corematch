use crate::block::{Block, BlockNumber, BlockView, Corespace};
use crate::buttons::{ActionButton, BlockViewButton, InfoButton, NetworkButton, ShareButton};
use crate::core::{Core, CoreView};
use crate::network::{NetworkState, NetworkStatus};
use crate::runtimes::support::SupportedRuntime;
use crate::subscription_provider::{SubscriptionId, SubscriptionProvider};
use log::{debug, info};
use std::collections::BTreeMap;
use std::rc::Rc;
use std::time::Duration;
use subxt::utils::H256;
use yew::{
    classes, function_component, html, html::Scope, platform::time::sleep, AttrValue, Callback,
    Component, Context, ContextProvider, Html, Properties,
};

const DEFAULT_INITIAL_POINTS: u32 = 0;
const DEFAULT_INITIAL_DURATION: u32 = 0;
const DEFAULT_INITIAL_TRIES: u32 = 6;
const DEFAULT_INITIAL_HELPS: u32 = 16;

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
    InfoButtonClicked,
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
    last_block: Option<Block>,
    game_status: GameStatus,
    duration: u32,
    points: u32,
    tries: u32,
    is_help_on: bool,
    is_help_disabled: bool,
    help_duration: u32,
    is_info_on: bool,
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
            last_block: None,
            game_status: GameStatus::Standby,
            duration: DEFAULT_INITIAL_DURATION,
            points: DEFAULT_INITIAL_POINTS,
            tries: DEFAULT_INITIAL_TRIES,
            is_help_on: false,
            is_help_disabled: false,
            help_duration: DEFAULT_INITIAL_HELPS,
            is_info_on: false,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::NetworkButtonClicked(network) => {
                if self.network_state.is_active() {
                    let network_state = Rc::make_mut(&mut self.network_state);
                    network_state.status = NetworkStatus::Switching;
                    network_state.runtime = SupportedRuntime::from(network);
                    self.game_status = GameStatus::Resetting;
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
                    // debug!("_matches {:?}", self.matches);
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
                                    // verify if game is over
                                    if self.is_game_over() {
                                        if let Some(head) = self.blocks.get(0) {
                                            self.last_block = (*head).clone();
                                        }
                                        ctx.link().send_message(Msg::GameFinished);
                                    }
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
            Msg::InfoButtonClicked => {
                self.is_info_on = !self.is_info_on;
            }
            Msg::GameFinished => {
                info!("** Game Over **");
                info!("\n{}", self.share_message().unwrap_or_default());
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

                { self.game_view(ctx.link()) }

            </ContextProvider<Rc<NetworkState>>>
        }
    }
}

impl App {
    fn game_view(&self, link: &Scope<Self>) -> Html {
        html! {
            <>
                <div class="container">
                    <div class="header">
                        <h1 class="title">{"COREMATCH"}</h1>
                        <div class="subtitle">{"The Polkadot Matching Game"}</div>
                    </div>
                    <div class="content-stats">
                        { self.game_stats_view(link) }
                    </div>
                    <div class="content-wrapper">
                        <div class="content-menu">
                            { self.head_left_view(link) }
                            { self.head_right_view(link) }
                        </div>
                        <div class="content-body">
                            { self.right_menu_view(link) }
                            { self.info_view(link) }
                            {
                                match self.game_status {
                                    GameStatus::Resetting => { html! {  self.game_resetting_view(link) } }
                                    GameStatus::Over => { html! {  self.game_over_view(link) } }
                                    _ => { self.board_view(link) }
                                }
                            }
                        </div>
                        { self.footer_view() }
                    </div>
                </div>
            </>
        }
    }

    fn game_resetting_view(&self, _link: &Scope<Self>) -> Html {
        html! {
            <div class="game-resetting">
                <h4>{"loading..."}</h4>
            </div>
        }
    }

    fn game_over_view(&self, link: &Scope<Self>) -> Html {
        let play_again_onclick = link.callback(move |_| Msg::StartButtonClicked);
        let data = self.share_message().unwrap_or_default();
        html! {
            <div class="game-over">
                <h4>{"Game Over"}</h4>
                <div class="action">
                    <ActionButton label={"▶ play again"} disable={false} onclick={play_again_onclick} />
                    <ShareButton label={"↱ share"} {data} />
                </div>
            </div>
        }
    }

    fn board_view(&self, link: &Scope<Self>) -> Html {
        let network_class = Some(self.network_state.runtime.to_string().to_lowercase());
        html! {
            <div class="board">
                { for self.blocks.iter().enumerate().map(|(i, block_option)| {
                        if let Some(block) = block_option {
                            let block_clicked = link.callback(move |_| Msg::BlockClicked(i.clone()));
                            let block_animation_ended = link.callback(move |bn| Msg::BlockAnimationEnded(bn));
                            block.render(self.block_view.clone(), self.core_view.clone(), block_clicked.clone(), block_animation_ended.clone())
                        } else {
                            html! { <div class={classes!("corespace", Some(self.network_state.runtime.to_string().to_lowercase()), "empty")}></div> }
                        }
                    })
                }
            </div>
        }
    }

    fn head_left_view(&self, _link: &Scope<Self>) -> Html {
        match self.network_state.runtime {
            SupportedRuntime::Polkadot => {
                html! {
                    <a class="logo-network" href="https://polkadot.network" target="_blank">
                        <img class="icon-img" src="/images/Polkadot_Logo_Horizontal_Pink-Black.svg" alt="polkadot logo" />
                    </a>
                }
            }
            SupportedRuntime::Kusama => {
                html! {
                    <a class="logo-network" href="https://kusama.network" target="_blank">
                        <img class="icon-img" src="/images/KUSAMA_6.svg" alt="kusama logo" />
                    </a>
                }
            }
        }
    }

    fn head_right_view(&self, link: &Scope<Self>) -> Html {
        let option_click = link.callback(move |e| Msg::BlockViewClicked(e));
        let info_click = link.callback(move |_| Msg::InfoButtonClicked);

        html! {
            <div class="top-right-options">
                <BlockViewButton view={BlockView::Cores}
                    selected={self.block_view == BlockView::Cores}
                    onclick={option_click.clone()} />
                <BlockViewButton view={BlockView::Palette}
                    selected={self.block_view == BlockView::Palette}
                    onclick={option_click.clone()} />
                <InfoButton label="ⓘ" onclick={info_click} />
            </div>
        }
    }

    fn right_menu_view(&self, link: &Scope<Self>) -> Html {
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

    fn info_view(&self, link: &Scope<Self>) -> Html {
        let visible_class = if self.is_info_on {
            Some("visible")
        } else {
            Some("hidden")
        };

        html! {
            <div class={classes!("info-view", visible_class)}>
                <h5>{"Corematch"}</h5>
                <h6>{"What is this?"}</h6>
                <p>{"Corematch is a memory game where the player has to match the latest Polkadot (or Kusama) corespace usage in a 4x4 matrix."}</p>
                <h6>{"What are the rules?"}</h6>
                <p>{"The rules are straightforward: with every finalized block, the corespace usage is unveiled and displayed.
                Your mission is to earn points by spotting a matching pattern from the preceding blocks. Be careful though - a wrong block selection results in the loss of a try. 
                The game kicks of when you press '▶ start' and concludes when you exhaust all available tries."}</p>
                <p>{"When the game is over, share your results with friends and family. Challenge them to join you in the Corematch game and embark on a quest for the highest score."}</p>
                <p>{"Have fun and enjoy ✌️"}</p>
            </div>
        }
    }

    fn game_stats_view(&self, link: &Scope<Self>) -> Html {
        let start_onclick = link.callback(move |_| Msg::StartButtonClicked);
        let help_onclick = link.callback(move |_| Msg::HelpButtonClicked);

        html! {
            <table class="game-stats">
                <tr>
                    <th>{"Points"}</th>
                    <th>{"Duration"}</th>
                    <th>{"Tries"}</th>
                    <th>{"Helps"}</th>
                    { self.game_message_view(link) }
                </tr>
                <tr>
                    <td class="points">{self.points}</td>
                    <td class="duration">{self.duration}</td>
                    <td class="tries">{self.tries}</td>
                    <td class="help-on">{self.help_duration}</td>
                    <td class="action">
                        <ActionButton label={"▶ start"} disable={self.is_game_on()} onclick={start_onclick} />
                        <ActionButton label={"■□ helps"}
                            disable={!self.is_game_on() || self.is_help_on || self.help_duration == 0} onclick={help_onclick} />
                    </td>
                </tr>
            </table>
        }
    }

    fn game_message_view(&self, link: &Scope<Self>) -> Html {
        html! {
            <th class="message">
                { if self.is_game_on() { html! { <div class="game-on">{"It's ON!"}</div> } }
                  else {
                    html! { <div>{"Play Corematch!"}</div> }
                  }
                }
            </th>
        }
    }

    fn footer_view(&self) -> Html {
        html! {
            <footer class="footer">
                <div class="footer-content">
                    <div class="caption">{"■□ Corematch // Built by Turboflakes // Unstoppable by Polkadot"}</div>
                    <div class="caption">{"© 2023 TurboFlakes"}</div>
                </div>
                <div class="footer-icons">
                    <a class="logo" href="https://turboflakes.io" target="_blank">
                        <img class="icon-img" src="/images/logo_mark_black_subtract_turboflakes_.svg" alt="turboflakes logo" />
                    </a>
                    <a class="logo" href="https://github.com/turboflakes/corematch" target="_blank">
                        <img class="icon-img" src="/images/github.svg" alt="github logo" />
                    </a>
                </div>
            </footer>
        }
    }

    fn full_reset(&mut self) {
        self.reset();
        self.blocks = vec![None; 16];
    }

    fn reset(&mut self) {
        self.match_block = None;
        self.matches = BTreeMap::new();
        self.game_status = GameStatus::Standby;
        self.duration = DEFAULT_INITIAL_DURATION;
        self.points = DEFAULT_INITIAL_POINTS;
        self.tries = DEFAULT_INITIAL_TRIES;
        self.is_help_on = false;
        self.is_help_disabled = false;
        self.help_duration = DEFAULT_INITIAL_HELPS;
    }

    fn is_game_on(&self) -> bool {
        self.game_status == GameStatus::On
    }

    fn is_game_over(&self) -> bool {
        self.game_status == GameStatus::Over
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

    fn share_message(&self) -> Option<AttrValue> {
        if let Some(block) = &self.last_block {
            let mut data = Vec::new();
            data.push(format!(
                "■□ corematch.io {}/{}/{}\n",
                self.points, self.duration, block.block_number
            ));
            data.push(block.runtime.hashtag());
            Some(data.join("\n").into())
        } else {
            None
        }
    }
}
