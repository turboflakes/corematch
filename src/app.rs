use crate::account::{Account, AccountState, AccountStatus, SigningStatus};
use crate::account_provider::AccountProvider;
use crate::block::{Block, BlockNumber, BlockView, Corespace};
use crate::block_timer::BlockTimer;
use crate::block_timer::_Props::visible;
use crate::buttons::_LevelProps::level;
use crate::buttons::{
    ActionButton, BlockViewButton, IconButton, LevelButton, MintButton, NetworkButton, ShareButton,
    TextButton,
};
use crate::core::{Core, CoreView};
use crate::keyboard::SupportedKeys;
use crate::network::{
    generate_parachain_colors, NetworkState, NetworkStatus, ParachainColors, ParachainIds,
};
use crate::runtimes::polkadot::node_runtime::runtime_apis::babe_api::types::GenerateKeyOwnershipProof;
use crate::runtimes::support::{SupportedParachainRuntime, SupportedRelayRuntime};
use crate::subscription_provider::{SubscriptionId, SubscriptionProvider};
use crate::views::ColumnInfoView;
use futures::executor::block_on;
use gloo::events::EventListener;
use gloo::timers::callback::Timeout;
use log::{debug, info};
use std::{collections::BTreeMap, rc::Rc, time::Duration};
use subxt::utils::H256;
use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::{window, HtmlElement};
use yew::{
    classes,
    events::{FocusEvent, KeyboardEvent},
    function_component, html,
    html::Scope,
    platform::time::sleep,
    AttrValue, Callback, Component, Context, ContextProvider, Html, NodeRef, Properties,
};

const DEFAULT_INITIAL_POINTS: u32 = 0;
const DEFAULT_BASE_POINTS: u32 = 4;
const DEFAULT_INITIAL_DURATION: u32 = 0;
const DEFAULT_INITIAL_TRIES: u32 = 4;
const DEFAULT_INITIAL_HELPS: u32 = 8;

#[derive(Clone, PartialEq, Debug)]
pub enum BoardStatus {
    Game,
    Account,
    Options,
    Mint,
    About,
    Leaderboard,
}

#[derive(Clone, PartialEq)]
pub enum GameStatus {
    Init,
    // Ready: // TODO: after initial blocks loaded change status to Ready (game should be playable now)
    Ready,
    // Reload: game is in this status when network is being changed
    Reload,
    // Game is ON
    On,
    // Game is finished
    Over,
    // Game in transit to next level
    MoveTo(GameLevel),
}

impl std::fmt::Display for GameStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Init => write!(f, "Initializing"),
            Self::Ready => write!(f, "Ready for play"),
            Self::Reload => write!(f, "Reload"),
            Self::On => write!(f, "Is On!"),
            Self::Over => write!(f, "Is Over!"),
            Self::MoveTo(l) => write!(f, "Moving to {}", l),
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum GameLevel {
    Level1,
    Level2,
}

impl GameLevel {
    pub fn block_view(&self) -> BlockView {
        match &self {
            Self::Level1 => BlockView::Cores,
            Self::Level2 => BlockView::Cores,
        }
    }

    pub fn core_view(&self, opt: Option<ParachainColors>) -> CoreView {
        match &self {
            Self::Level1 => CoreView::Binary,
            Self::Level2 => {
                if let Some(colors) = opt {
                    CoreView::Multi(colors)
                } else {
                    CoreView::NotApplicable
                }
            }
        }
    }

    pub fn collected_points_per_level_minimum(&self) -> u32 {
        match &self {
            Self::Level1 => 32,
            _ => unimplemented!(),
        }
    }

    pub fn match_x_position(&self) -> u32 {
        match &self {
            Self::Level1 => 3,
            Self::Level2 => 0,
        }
    }

    pub fn class(&self) -> String {
        match &self {
            Self::Level1 => "level__1".to_string(),
            Self::Level2 => "level__2".to_string(),
        }
    }
}

impl std::fmt::Display for GameLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Level1 => write!(f, "Level 1"),
            Self::Level2 => write!(f, "Level 2"),
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum GameHelpStatus {
    On,
    NotAvailable,
    Available,
}

impl GameHelpStatus {
    pub fn is_on(&self) -> bool {
        *self == Self::On
    }

    pub fn is_available(&self) -> bool {
        *self == Self::Available
    }

    pub fn is_not_available(&self) -> bool {
        *self == Self::NotAvailable
    }
}

pub enum Msg {
    NetworkSubscriptionCreated(SubscriptionId),
    NetworkDataReceived((SubscriptionId, Block)),
    NetworkParachainsCollected(ParachainIds),
    NetworkButtonClicked(AttrValue),
    BlockClicked(usize),
    BlockPressed(usize),
    BlockMatched(usize),
    BlockMissed(usize),
    BlockAnimationEnded(BlockNumber),
    CheckGameStatus,
    StartButtonClicked,
    HelpButtonClicked,
    LevelButtonClicked(GameLevel),
    InfoButtonClicked,
    MintButtonClicked,
    NextLevel(GameLevel),
    NextLevelTimeout(GameLevel),
    //
    AccountsLoaded(Vec<Account>),
    AccountClicked(Account),
    //
    SigningFinished(SigningStatus),
    //
    KeyPressed(SupportedKeys),
}

type X = u8;
type Y = u8;
type Position = (X, Y);

pub struct App {
    board_status: BoardStatus,
    previous_board_status: Option<BoardStatus>,
    network_state: Rc<NetworkState>,
    blocks: Vec<Option<Block>>,
    match_position: Option<Position>,
    match_counter: u32,
    matches: BTreeMap<H256, u32>,
    previous_match_block: Option<Block>,
    game_status: GameStatus,
    game_level: GameLevel,
    duration: u32,
    points: u32,
    previous_points: u32,
    tries: u32,
    helps: u32,
    game_help_status: GameHelpStatus,
    account_state: Rc<AccountState>,
    keyboard_listener: Option<EventListener>,
    cursor_position: Position,
    timeout: Option<Timeout>,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        // Set default runtime as Polkadot
        let runtime = SupportedRelayRuntime::Polkadot;
        let runtime_callback = ctx.link().callback(Msg::NetworkDataReceived);
        let subscription_callback = ctx.link().callback(Msg::NetworkSubscriptionCreated);
        let parachains_callback = ctx.link().callback(Msg::NetworkParachainsCollected);
        // Initialized shared state
        let network_state = Rc::new(NetworkState::new(
            runtime.clone(),
            runtime_callback,
            subscription_callback,
            parachains_callback,
        ));

        // TODO: verify if account is available from localstorage
        let accounts_callback = ctx.link().callback(Msg::AccountsLoaded);
        let signing_callback = ctx.link().callback(Msg::SigningFinished);
        let account_state = Rc::new(AccountState::new(
            SupportedParachainRuntime::AssetHubPolkadot,
            accounts_callback.clone(),
            signing_callback.clone(),
        ));

        Self {
            board_status: BoardStatus::Game,
            previous_board_status: None,
            network_state,
            blocks: vec![None; 16],
            match_position: None,
            match_counter: 0,
            matches: BTreeMap::new(),
            previous_match_block: None,
            game_status: GameStatus::Init,
            game_level: GameLevel::Level1,
            duration: DEFAULT_INITIAL_DURATION,
            points: DEFAULT_INITIAL_POINTS,
            previous_points: DEFAULT_INITIAL_POINTS,
            tries: DEFAULT_INITIAL_TRIES,
            helps: DEFAULT_INITIAL_HELPS,
            game_help_status: GameHelpStatus::Available,
            account_state,
            keyboard_listener: None,
            cursor_position: (0, 0),
            timeout: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::NetworkButtonClicked(network) => {
                if self.network_state.is_active() {
                    let network_state = Rc::make_mut(&mut self.network_state);
                    network_state.status = NetworkStatus::Switching;
                    network_state.runtime = SupportedRelayRuntime::from(network);
                    // NOTE: if network (relay) changes than account_state runtime (asset-hub) also changes
                    let account_state = Rc::make_mut(&mut self.account_state);
                    account_state.runtime = network_state.runtime.asset_hub_runtime();

                    self.game_status = GameStatus::Reload;
                }
            }
            Msg::NetworkSubscriptionCreated(subscription_id) => {
                let network_state = Rc::make_mut(&mut self.network_state);
                network_state.subscription_id = Some(subscription_id);
                network_state.status = NetworkStatus::Active;
                // apply a full reset
                self.full_reset();
            }
            Msg::NetworkParachainsCollected(para_ids) => {
                let network_state = Rc::make_mut(&mut self.network_state);
                network_state.parachain_colors = generate_parachain_colors(para_ids.clone());
            }
            Msg::NetworkDataReceived((subscription_id, block)) => {
                // FOR TESTING ONLY -- start
                // if self.game_status == GameStatus::Minting {
                //     return true;
                // }
                // self.game_status = GameStatus::Over;
                // self.board_status = BoardStatus::Options;
                // FOR TESTING ONLY -- end

                if self.network_state.is_valid(subscription_id) {
                    // reset match block
                    self.reset_match_block();
                    // add latest block into the first position
                    self.blocks.insert(0, Some(block.clone()));
                    let block_hash = block.corespace_hash(self.game_level.clone());
                    // add match counter for block_hash_key
                    self.matches
                        .entry(block_hash)
                        .and_modify(|m| *m += 1)
                        .or_insert(1);
                    // oldest block gets removed
                    if self.blocks.len() > 16 {
                        if let Some(opt) = self.blocks.pop() {
                            if let Some(block) = opt {
                                let block_hash = block.corespace_hash(self.game_level.clone());
                                // subtract counter from block_hash_key
                                self.matches.entry(block_hash.clone()).and_modify(|m| {
                                    if *m >= 1 {
                                        *m -= 1
                                    }
                                });
                                // remove if counter is zero
                                if let Some(counter) = self.matches.get(&block_hash) {
                                    if *counter == 0 {
                                        self.matches.remove(&block_hash);
                                    }
                                }
                            }
                        }
                    }

                    if self.is_game_on() {
                        // guarantee that only the current cursor position is selected
                        let cursor_index = self.get_cursor_index();
                        for (i, opt) in self.blocks.iter_mut().enumerate() {
                            if let Some(block) = opt {
                                if self.game_status == GameStatus::On && cursor_index == i {
                                    block.selected();
                                } else {
                                    block.unselected();
                                    block.cleared();
                                }
                            }
                        }

                        // highlight matches if help is on
                        if self.game_help_status.is_on() {
                            let matches: Vec<_> = Vec::from_iter(self.matches.iter())
                                .iter()
                                .filter(|(_, counter)| **counter > 1)
                                .map(|(hash, _)| **hash)
                                .collect();

                            // highlight only the same pattern at a time
                            if matches.len() > 1 {
                                let mut help_matches_counter = 0;
                                if let Some(block_hash) = matches.get(matches.len() - 1) {
                                    for opt in self.blocks.iter_mut() {
                                        if let Some(block) = opt {
                                            if *block_hash
                                                == block.corespace_hash(self.game_level.clone())
                                            {
                                                if block.is_help_available() && !block.is_disabled()
                                                {
                                                    block.help();
                                                    help_matches_counter += 1;
                                                }
                                            }
                                        }
                                    }
                                    self.decr_help_matches(help_matches_counter);
                                }
                            }
                        }
                    }

                    // update game stats if game is on
                    self.incr_duration();
                }
            }
            Msg::BlockClicked(i) => {
                if self.is_game_on() {
                    let cursor_index = self.get_cursor_index();
                    if cursor_index != i {
                        self.unselect_block(cursor_index);
                    }
                    self.set_cursor_position(i);
                    self.select_block(i);
                }
            }
            Msg::BlockPressed(i) => {
                if self.is_game_on() {
                    let match_block = self.get_match_block();
                    if let Some(opt) = self.blocks.get(i) {
                        if let Some(block) = opt {
                            if block.is_matched() || block.is_disabled() {
                                return false;
                            }
                            if let Some(match_block) = match_block {
                                let corespace_hash = block.corespace_hash(self.game_level.clone());
                                if match_block.block_number == block.block_number {
                                    // unselect previous match block
                                    self.match_position = None;
                                    return false;
                                } else if match_block.corespace_hash(self.game_level.clone())
                                    == corespace_hash
                                {
                                    ctx.link().send_message(Msg::BlockMatched(i));
                                } else {
                                    ctx.link().send_message(Msg::BlockMissed(i));
                                }
                            } else {
                                // first block pressed is the one to be matched
                                let cursor_index = self.get_cursor_index();
                                self.set_match_position(cursor_index);
                            }
                        }
                    }
                }
            }
            Msg::BlockMatched(i) => {
                info!("Congrats, you found a match!");
                if let Some(opt) = self.blocks.get_mut(i) {
                    if let Some(block) = opt {
                        // highlight block matched
                        block.matched();
                    }
                }
                if let Some(i) = self.get_match_index() {
                    if let Some(opt) = self.blocks.get_mut(i) {
                        if let Some(block) = opt {
                            // highlight block matched
                            block.matched();
                            // remove from matches
                            let block_hash = block.corespace_hash(self.game_level.clone());
                            // subtract counter from block_hash_key
                            self.matches.entry(block_hash.clone()).and_modify(|m| {
                                if *m >= 2 {
                                    *m -= 2
                                }
                            });
                        }
                    }
                }
                // increase points
                self.match_succeed();
            }
            Msg::BlockMissed(i) => {
                info!("Wrong match!");
                if let Some(opt) = self.blocks.get_mut(i) {
                    if let Some(block) = opt {
                        block.missed();
                    }
                }
                if let Some(i) = self.get_match_index() {
                    if let Some(opt) = self.blocks.get_mut(i) {
                        if let Some(block) = opt {
                            // highlight block matched
                            block.missed();
                        }
                    }
                }
                // decrease attempts
                self.match_failed();
                // check status
                ctx.link().send_message(Msg::CheckGameStatus);
            }
            Msg::CheckGameStatus => {
                // verify if game is over
                if self.is_game_over() {
                    info!("** Game Over **");
                    // keep a copy of the last match block
                    if let Some(index) = self.get_match_index() {
                        if let Some(opt) = self.blocks.get(index) {
                            if let Some(match_block) = opt {
                                self.previous_match_block.replace(match_block.clone());
                                info!("\n{}", self.share_message().unwrap_or_default());
                                // clear selected block
                                let i = self.get_cursor_index();
                                self.unselect_block(i);
                                // show available options
                                self.board_status = BoardStatus::Options;
                            }
                        }
                    }
                }
                // reset match block
                self.reset_match_block();
            }
            Msg::NextLevel(next_level) => {
                self.game_status = GameStatus::MoveTo(next_level.clone());
                // restore helps at each new level
                self.helps = DEFAULT_INITIAL_HELPS;
                self.game_help_status = GameHelpStatus::Available;
                // set timeout to continue
                let handle = {
                    let link = ctx.link().clone();
                    Timeout::new(6000, move || {
                        link.send_message(Msg::NextLevelTimeout(next_level.clone()))
                    })
                };
                self.timeout = Some(handle);
            }
            Msg::NextLevelTimeout(next_level) => {
                self.game_level = next_level;
                self.game_status = GameStatus::On;
                self.timeout = None;
            }
            Msg::BlockAnimationEnded(block_number) => {
                if let Some(i) = self
                    .blocks
                    .iter()
                    .position(|opt| opt.clone().unwrap().block_number == block_number)
                {
                    if let Some(block_option) = self.blocks.get_mut(i) {
                        if let Some(block) = block_option {
                            if block.is_matched() {
                                // disable block
                                block.disabled();
                                // check if is time to move to next level
                                if self.is_next_level_available(GameLevel::Level1) {
                                    info!("Well Done! Level 2 available for playing.");
                                    ctx.link().send_message(Msg::NextLevel(GameLevel::Level2));
                                }
                            } else {
                                block.cleared();
                            }
                        }
                    }
                }
            }
            Msg::StartButtonClicked => {
                self.start();
            }
            Msg::HelpButtonClicked => {
                // reset matches map
                let mut matches: BTreeMap<H256, u32> = BTreeMap::new();
                for opt in self.blocks.iter() {
                    if let Some(block) = opt {
                        let block_hash = block.corespace_hash(self.game_level.clone());
                        matches
                            .entry(block_hash)
                            .and_modify(|m| *m += 1)
                            .or_insert(1);
                    }
                }
                self.matches = matches.clone();
                self.start_help();
            }
            Msg::InfoButtonClicked => {
                if self.board_status == BoardStatus::About {
                    if let Some(previous) = self.previous_board_status.clone() {
                        self.board_status = previous;
                    }
                } else {
                    self.previous_board_status = Some(self.board_status.clone());
                    self.board_status = BoardStatus::About;
                }
            }
            Msg::MintButtonClicked => {
                info!("MintButtonClicked");
                // TODO:
                // self.board_status = BoardStatus::Mint;
                // if self.account_state.is_none() {
                //     let account_state = Rc::make_mut(&mut self.account_state);
                //     account_state.status = AccountStatus::Requesting;
                // } else if self.account_state.is_available() {
                //     let results = self.game_results().unwrap();
                //     let account_state = Rc::make_mut(&mut self.account_state);
                //     account_state.status = AccountStatus::Signing(results);
                // }

                // TODO
                // 1. check if account is already loaded from pjs
                // 1.1 If Not in state launch pjs and list accounts for user to select
                // 1.2 If account in state send mint tx
                // if self.account_state.account.is_none() {
                //     ctx.link()
                //         .send_future(get_accounts().map(|accounts_or_err| match accounts_or_err {
                //             Ok(accounts) => Message::ReceivedAccounts(accounts),
                //             Err(err) => Message::Error(err),
                //         }));
                // }
            }
            Msg::LevelButtonClicked(game_level) => {
                self.game_level = game_level;
            }
            Msg::AccountsLoaded(accounts) => {
                let account_state = Rc::make_mut(&mut self.account_state);
                account_state.status = AccountStatus::Selection(accounts);
                // change board view to manage accounts
                self.previous_board_status = Some(self.board_status.clone());
                self.board_status = BoardStatus::Account;
            }
            Msg::AccountClicked(account) => {
                let account_state = Rc::make_mut(&mut self.account_state);
                account_state.account = Some(account);
                account_state.status = AccountStatus::Selected;
                //
                // change board to previous board or game view
                if let Some(previous) = self.previous_board_status.clone() {
                    self.board_status = previous;
                } else {
                    self.board_status = BoardStatus::Game;
                }
            }
            Msg::SigningFinished(status) => {
                info!("__SigningFinished {:?}", status);
                match status {
                    SigningStatus::Failed => {
                        // Restore game status and let user decide what to do next
                        self.game_status = GameStatus::Over;
                        //
                        let account_state = Rc::make_mut(&mut self.account_state);
                        account_state.status = AccountStatus::Selected;
                    }
                    _ => todo!(),
                }
                // let account_state = Rc::make_mut(&mut self.account_state);
                // account_state.status = AccountStatus::Selected;
                // // reset game
                // self.reset();
            }
            Msg::KeyPressed(key) => {
                match key {
                    SupportedKeys::Enter => {
                        if !self.is_game_on() {
                            self.start()
                        } else {
                            let i = self.get_cursor_index();
                            ctx.link().send_message(Msg::BlockPressed(i))
                        }
                    }
                    SupportedKeys::Space => {
                        if self.is_game_on() {
                            let i = self.get_cursor_index();
                            ctx.link().send_message(Msg::BlockPressed(i))
                        }
                        // TODO: if game over space could be used to restart the game
                        info!("Skip")
                    }
                    SupportedKeys::Up => match self.cursor_position.1 {
                        0 => self.move_cursor((self.cursor_position.0, 3)),
                        _ => self.move_cursor((self.cursor_position.0, self.cursor_position.1 - 1)),
                    },
                    SupportedKeys::Down => match self.cursor_position.1 {
                        3 => self.move_cursor((self.cursor_position.0, 0)),
                        _ => self.move_cursor((self.cursor_position.0, self.cursor_position.1 + 1)),
                    },
                    SupportedKeys::Left => match self.cursor_position.0 {
                        0 => self.move_cursor((3, self.cursor_position.1)),
                        _ => self.move_cursor((self.cursor_position.0 - 1, self.cursor_position.1)),
                    },
                    SupportedKeys::Right => match self.cursor_position.0 {
                        3 => self.move_cursor((0, self.cursor_position.1)),
                        _ => self.move_cursor((self.cursor_position.0 + 1, self.cursor_position.1)),
                    },
                    SupportedKeys::S => self.start(),
                    SupportedKeys::H => self.start_help(),
                    SupportedKeys::F => self.show_details(),
                    _ => info!("Skip"),
                };
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let network_state = self.network_state.clone();
        let account_state = self.account_state.clone();
        html! {
            <ContextProvider<Rc<NetworkState>> context={ network_state.clone() }>

                <ContextProvider<Rc<AccountState>> context={ account_state.clone() }>

                    { self.app_view(ctx.link()) }

                </ContextProvider<Rc<AccountState>>>

            </ContextProvider<Rc<NetworkState>>>
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            let document = window().unwrap().document().unwrap();
            let ct = ctx.link().to_owned();
            let listener = EventListener::new(&document, "keydown", move |event| {
                let event = event.dyn_ref::<KeyboardEvent>().unwrap_throw().to_owned();
                ct.send_message(Msg::KeyPressed(event.key().into()));
            });

            self.keyboard_listener.replace(listener);
        }
    }
}

impl App {
    fn app_view(&self, link: &Scope<Self>) -> Html {
        html! {
            <>
                <div class="container">
                    <div class="content__wrapper">
                        <div class="content__menu">
                            { self.head_left_view(link) }
                            // { self.head_right_view(link) }
                        </div>
                        <div class="content__body">
                            <div class="cb__left">
                                { self.left_top_view(link) }
                                { self.left_bottom_view(link) }
                            </div>
                            <div class="cb__middle">
                                {
                                    match self.board_status {
                                        BoardStatus::About => { html! {  self.about_view(link) } }
                                        BoardStatus::Options => { html! {  self.options_view(link) } }
                                        BoardStatus::Account => { html! {  self.accounts_view(link) } }
                                        _ => { self.game_view(link) }
                                    }
                                }
                            </div>
                            <div class="cb__right">
                                { self.right_top_view(link) }
                            </div>
                        </div>
                        { self.footer_view() }
                    </div>
                </div>
            </>
        }
    }

    fn options_view(&self, link: &Scope<Self>) -> Html {
        let play_again_onclick = link.callback(move |_| Msg::StartButtonClicked);
        let mint_onclick = link.callback(move |_| Msg::MintButtonClicked);
        let data = self.share_message().unwrap_or_default();
        html! {
            <div class="gameover">
                <img class="corematch__icon" src="/images/corematch_icon_animated_gameover.svg" alt="corematch icon animated" />
                <div class="action">
                    <ActionButton label={"play again"} disable={false} onclick={play_again_onclick}>
                        <img class="icon" src="/images/start_icon_white_clear.svg" alt="start_icon" />
                    </ActionButton>
                    <ShareButton label={"share"} data={data.clone()}>
                        <img class="icon" src="/images/share_icon_white_clear.svg" alt="share_icon" />
                    </ShareButton>
                    <MintButton  label={"mint"} onclick={mint_onclick}>
                        <img class="icon" src="/images/mint_icon_white_clear.svg" alt="mint_icon" />
                    </MintButton>
                </div>
            </div>
        }
    }

    fn accounts_status_view(&self, msg: &str) -> Html {
        html! {
            <div class="status__msg">
                <h4>{msg}</h4>
            </div>
        }
    }

    fn list_accounts_view(&self, accounts: Vec<Account>, link: &Scope<Self>) -> Html {
        html! {
            <div>
                { for accounts.iter().map(|account| {
                        let acc = account.clone();
                        let account_clicked = link.callback(move |_| Msg::AccountClicked(acc.clone()));
                        account.render(account_clicked.clone())
                    })
                }
            </div>
        }
    }

    fn accounts_view(&self, link: &Scope<Self>) -> Html {
        html! {
            <div class="accounts">
                {
                    match &self.account_state.status {
                        AccountStatus::Requesting => { html! {  self.accounts_status_view("loading...") } }
                        AccountStatus::Signing(_) => { html! {  self.accounts_status_view("signing...") } }
                        AccountStatus::Selection(accounts) => { html! {  self.list_accounts_view(accounts.clone(), link) } }
                        _ => unimplemented!()
                    }
                }
            </div>
        }
    }

    fn score_view(&self, link: &Scope<Self>) -> Html {
        let visible_class = if self.is_game_on() {
            Some("visible")
        } else {
            Some("hidden")
        };
        html! {
            <span class={classes!("score__info", visible_class)}>
                <span>{"points: "} <b>{format!("{} ", self.points)}</b></span>
                <span>{"duration: "} <b>{format!("{} ", self.duration)}</b></span>
                <span>{"attempts: "} <b>{format!("{} ", self.tries)}</b></span>
                <span>{"helps: "} <b>{format!("{} ", self.helps)}</b></span>
                { self.block_countdown_view(link)}
            </span>
        }
    }

    fn block_countdown_view(&self, _link: &Scope<Self>) -> Html {
        // reset countdown every time a new block is added to the board
        let block_number = if let Some(opt) = self.blocks.get(0) {
            if let Some(block) = opt {
                Some(block.block_number)
            } else {
                None
            }
        } else {
            None
        };
        html! { <BlockTimer block_number={block_number.clone()} visible={self.is_game_on()} /> }
    }

    fn _match_caption_view(&self, _link: &Scope<Self>) -> Html {
        let visible_class = if self.is_help_on() {
            Some("visible")
        } else {
            Some("hidden")
        };
        html! { <span class={classes!("help__info", visible_class)}>{format!("{} highlights left!", self.helps)} </span> }
    }

    fn attempts_column_view(&self, _link: &Scope<Self>) -> Html {
        let visible_class = if self.is_game_on() {
            Some("visible")
        } else {
            Some("hidden")
        };
        let box_class: Option<AttrValue> = None;
        let value = self.tries.clone();
        html! { <ColumnInfoView max={DEFAULT_INITIAL_TRIES} {value} title="attempts left!"
        class={visible_class} position_class={Some("left")} {box_class} /> }
    }

    fn helps_column_view(&self, link: &Scope<Self>) -> Html {
        let visible_class = if self.is_game_on() {
            Some("visible")
        } else {
            Some("hidden")
        };
        let box_class = if self.is_help_on() {
            Some("is__on")
        } else {
            None
        };

        let value = self.helps.clone();
        html! { <ColumnInfoView max={DEFAULT_INITIAL_HELPS} {value} title="helps left!"
        class={visible_class} position_class={Some("right")} {box_class} /> }
    }

    fn game_view(&self, link: &Scope<Self>) -> Html {
        let is_game_on_class = if self.is_game_on() {
            Some("is__on")
        } else {
            None
        };

        match &self.game_status {
            GameStatus::MoveTo(game_level) => html! {
                <>
                    <div class={classes!("gameboard", "move__to")}>
                        <h4>{format!("{} Next!", game_level.clone())}</h4>
                    </div>
                </>
            },
            GameStatus::Reload => html! {
                <>
                    <div class={classes!("gameboard", "reloading")}>
                        <h5>{format!("RELOADING")}</h5>
                    </div>
                </>
            },
            _ => html! {
                <>
                    { self.score_view(link) }
                    // { self.base_points_view(link) }
                    { self.attempts_column_view(link) }
                    { self.helps_column_view(link) }
                    <div class={classes!("gameboard", is_game_on_class, self.game_level.class(), self.match_class())}>
                        { for self.blocks.iter().enumerate().map(|(i, block_option)| {
                                if let Some(block) = block_option {
                                    let block_clicked = link.callback(move |_| Msg::BlockClicked(i.clone()));
                                    let block_dblclicked = link.callback(move |_| Msg::BlockPressed(i.clone()));
                                    let block_animation_ended = link.callback(move |bn| Msg::BlockAnimationEnded(bn));
                                    block.render(
                                        self.game_level.core_view(Some(self.network_state.parachain_colors.clone())),
                                        block_clicked.clone(),
                                        block_dblclicked.clone(),
                                        block_animation_ended.clone()
                                    )
                                } else {
                                    html! { <div class={classes!("corespace", Some(self.network_state.runtime.to_string().to_lowercase()), "empty")}></div> }
                                }
                            })
                        }
                    </div>
                </>
            },
        }
    }

    fn head_left_view(&self, _link: &Scope<Self>) -> Html {
        html! {
            <div class="header">
                <img class="corematch__logo" src="/images/corematch_logo_full.svg" alt="corematch logo" />
            </div>
        }
    }

    fn head_right_view(&self, link: &Scope<Self>) -> Html {
        html! { self.game_stats_view(link) }
    }

    fn left_top_view(&self, link: &Scope<Self>) -> Html {
        html! {
            <div class="top">
                {
                    match self.network_state.runtime {
                        SupportedRelayRuntime::Polkadot => {
                            html! {
                                <a class="logo__polkadot" href="https://polkadot.network" target="_blank">
                                    <img class="logo__img" src="/images/polkadot_logo_vertical.svg" alt="polkadot logo" />
                                </a>
                            }
                        }
                        SupportedRelayRuntime::Kusama => {
                            html! {
                                <a class="logo__kusama" href="https://kusama.network" target="_blank">
                                    <img class="logo__img" src="/images/kusama_logo_vertical.svg" alt="kusama logo" />
                                </a>
                            }
                        }
                    }
                }
            </div>
        }
    }

    fn left_bottom_view(&self, link: &Scope<Self>) -> Html {
        let network_state = self.network_state.clone();
        let network_onclick = link.callback(move |e| Msg::NetworkButtonClicked(e));

        let visible_class = if self.network_state.is_active() {
            Some("visible")
        } else {
            Some("hidden")
        };

        html! {
            <div class="bottom">
                <SubscriptionProvider>
                    { match network_state.runtime {
                        SupportedRelayRuntime::Polkadot => html! {
                            <NetworkButton switch_to="kusama" class={visible_class} onclick={network_onclick.clone()} >
                                <img class="icon__img" src="/images/kusama_icon.svg" alt="kusama logo" />
                            </NetworkButton>
                        },
                        SupportedRelayRuntime::Kusama => html! {
                            <NetworkButton switch_to="polkadot" class={visible_class} onclick={network_onclick.clone()} >
                                <img class="icon__img" src="/images/polkadot_icon_white.svg" alt="polkadot logo" />
                            </NetworkButton>
                        }
                    }}
                </SubscriptionProvider>
            </div>
        }
    }

    fn right_top_view(&self, link: &Scope<Self>) -> Html {
        html! {
            <div class="top">
                { self.game_commands_view(link) }
                // <AccountProvider />
            </div>
        }
    }

    fn about_view(&self, link: &Scope<Self>) -> Html {
        html! {
            <div class={classes!("game__about")}>
                <h6>{"What is this?"}</h6>
                <p>{"Corematch is a memory game where players must spot a matching pattern to earn points.
                    The board game holds a maximum of sixteen square objects ― "}<b><i>{"Boxes"}</i></b>{" ― arranjed in a 4x4 matrix."}</p>
                <h6>{"Where does the pattern come from?"}</h6>
                <p>{"The pattern is crafted from the Polkadot decentralized multi-core architecture. Depending on the selected chain, the pattern reflects the "}
                    <a class="link" href="https://wiki.polkadot.network/docs/polkadot-direction#core-usage-in-polkadot-10" target="_blank">{"core usage"}</a>
                    {" of either Polkadot or Kusama protocol on every finalized block."}
                </p>
                <p>{"Each box represents a finalized block, engraved with the number of cores available on chain, where each core is colored based on its usage.
                    In the current version - Polkadot v1.0, each core can only exist in two states: empty or full."}</p>
                <h6>{"What are the game rules?"}</h6>
                <p>{"The mission is to earn as many points as possible by spotting one or more matches between the predefined box and the others in 6 seconds.
                    If there is more than a pair, points are powered up.
                    However, a wrong box selection leads to a loss, and the game concludes if you make four incorrect selections."}</p>
                <h6>{"How to play?"}</h6>
                <p>{"You can play using either the mouse or the keyboard. If you opt for the mouse, double-click the left mouse button on top of the spotted matching box.
                    Alternatively, if you choose the keyboard, move around the selected box with the arrow keys and press 'enter' when you spot a matching one."}</p>
                <p>{"You can start playing by pressing the 'S' key or the button "}
                    <span><img class="icon__img" src="/images/start_icon.svg" alt="start_game" /></span>
                    {". During gameplay, you can make use of four helps by pressing the 'H' key or the button "}
                    <span><img class="icon__img" src="/images/match_icon.svg" alt="show_matches" /></span>
                    {", which highlights up to four matches to assist you in spotting them on time."}</p>
                <p>{"The box to be matched can be changed by pressing the numeric keys '1-4', with each selection yielding different points."}</p>
                <p>{"There are currently two levels at play: Level 1 is a multi-core binary representation of the network core usage.
                    Level 2 is a multi-core colorful representation based on parachain Ids and their respective core assignment.
                    Level 2 is available as soon as a minimum of 64 points are reached and you can switch bettwen levels by pressing the respective level buttons "}
                    <span><img class="icon__img" src="/images/level1_icon.svg" alt="level 1" /></span>{" "}
                    <span><img class="icon__img" src="/images/level2_icon.svg" alt="level 2" /></span>
                </p>
                <h6>{"Game Over - What can I do?"}</h6>
                <p>{"When the game is over, press the share button "}
                    <span><img class="icon__img" src="/images/share_icon.svg" alt="share results" /></span>
                    {" and share results with friends and family. Challenge them to join you in the Corematch game and embark them to explore about "}<a class="link" href="https://polkadot.network/" target="_blank">{"Polkadot's technology"}</a>{" and learn how to build on Polkadot."}</p>
                <h6>{"Mint Results - Coming Soon!"}</h6>
                <p>{"If you would like your score to show up in the leadearboard, press the mint button "}
                    <span><img class="icon__img" src="/images/mint_icon.svg" alt="mint results" /></span>
                {" and you will be prompt to connnect an Asset Hub account and sign the transaction to mint the results. This account will be entitled to a soulbound NFT and it will hold your results history."}</p>
                <h6>{"What comes next?"}</h6>
                <p>{"Corematch patterns will evolve into beautiful, colorful, core compositions, alongside Polkadot evolution. Explore more about Polkadot direction "}<a class="link" href="https://wiki.polkadot.network/docs/polkadot-direction#agile-composable-computer" target="_blank">{"here"}</a>{"."}</p>
                <p>{"If you've read this far, we hope you enjoy our work and may it serve as inspiration for fellow tinkerers out there."}</p>
                <p>{"Play on repeat, have fun and enjoy ✌️"}<br/>{"Paulo // Turboflakes"}</p>
            </div>
        }
    }

    fn game_stats_view(&self, link: &Scope<Self>) -> Html {
        html! {
            <table class="game__stats">
                <tr>
                    <th>{"Points"}</th>
                    <th>{"Duration"}</th>
                    <th>{"Attempts"}</th>
                </tr>
                <tr>
                    <td class="points">{self.points}</td>
                    <td class="duration">{self.duration}</td>
                    <td class="attempts">{self.tries}</td>
                </tr>
            </table>
        }
    }

    fn game_commands_view(&self, link: &Scope<Self>) -> Html {
        let start_onclick = link.callback(move |_| Msg::StartButtonClicked);
        let help_onclick = link.callback(move |_| Msg::HelpButtonClicked);
        let option_click = link.callback(move |e| Msg::LevelButtonClicked(e));
        let about_click = link.callback(move |_| Msg::InfoButtonClicked);

        html! {
            <div class="game__commands">
                <IconButton disable={self.is_game_on()} onclick={start_onclick}>
                    <img class="icon__img" src="/images/start_icon.svg" alt="start_game" title="Start Playing!" />
                </IconButton>
                <IconButton
                    disable={!self.is_game_on() || self.is_help_on() || self.helps == 0} onclick={help_onclick}>
                    <img class="icon__img"  src="/images/match_icon.svg" alt="show_matches" title="Highlight matches!" />
                </IconButton>
                <LevelButton level={GameLevel::Level2} disable={!self.is_level_x_completed(GameLevel::Level1) || self.game_level == GameLevel::Level2} onclick={option_click.clone()}>
                    <img class="icon__img"  src="/images/level2_icon.svg" alt="level 2" title="Play Level 2" />
                </LevelButton>
                <LevelButton level={GameLevel::Level1} disable={!self.is_game_on() || self.game_level == GameLevel::Level1} onclick={option_click.clone()}>
                    <img class="icon__img"  src="/images/level1_icon.svg" alt="level 1" title="Play Level 1" />
                </LevelButton>
                // <LevelButton level={GameLevel::Level0} disable={!self.is_game_on() || self.game_level == GameLevel::Level0} onclick={option_click.clone()}>
                //     <img class="icon__img"  src="/images/level0_icon.svg" alt="block_view" title="Play Level 0" />
                // </LevelButton>
                <IconButton disable={false} onclick={about_click}>
                    <img class="icon__img"  src="/images/question_icon.svg" alt="game_info" title="About Corematch" />
                </IconButton>
            </div>
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
                <div>
                    <div class="caption">{"Built by Turboflakes // Unstoppable by Polkadot"}</div>
                    <div class="caption">{"Corematch © 2024 Turboflakes"}</div>
                </div>
                <div class="footer__icons">
                    <a class="logo" href="https://turboflakes.io" target="_blank">
                        <img class="icon__img" src="/images/turboflakes_icon.svg" alt="turboflakes logo" />
                    </a>
                    <a class="logo" href="https://github.com/turboflakes/corematch" target="_blank">
                        <img class="icon__img" src="/images/github_icon.svg" alt="github logo" />
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
        self.reset_blocks();
        self.game_status = GameStatus::Ready;
        self.game_level = GameLevel::Level2;
        self.duration = DEFAULT_INITIAL_DURATION;
        self.points = DEFAULT_INITIAL_POINTS;
        self.tries = DEFAULT_INITIAL_TRIES;
        self.helps = DEFAULT_INITIAL_HELPS;
        self.game_help_status = GameHelpStatus::Available;
        self.cursor_position = (0, 0);
    }

    fn reset_blocks(&mut self) {
        for opt in self.blocks.iter_mut() {
            if let Some(block) = opt {
                block.reset();
            }
        }
    }

    fn is_game_on(&self) -> bool {
        match &self.game_status {
            GameStatus::On => true,
            GameStatus::MoveTo(_) => true,
            _ => false,
        }
    }

    fn is_game_over(&self) -> bool {
        self.game_status == GameStatus::Over
    }

    fn is_help_on(&self) -> bool {
        self.game_help_status.is_on()
    }

    fn is_next_level_available(&self, current_level: GameLevel) -> bool {
        self.game_level == current_level
            && self.previous_points < self.game_level.collected_points_per_level_minimum()
            && self.points >= self.game_level.collected_points_per_level_minimum()
    }

    fn is_level_x_completed(&self, game_level: GameLevel) -> bool {
        self.points >= game_level.collected_points_per_level_minimum()
    }

    fn start(&mut self) {
        if !self.is_game_on() {
            self.reset();
            self.previous_board_status = Some(self.board_status.clone());
            self.board_status = BoardStatus::Game;
            self.game_status = GameStatus::On;
            self.game_level = GameLevel::Level1;
        }
    }

    fn reset_match_block(&mut self) {
        self.match_counter = 0;
        self.match_position = None;
    }

    fn get_match_block(&self) -> Option<Block> {
        if let Some(index) = self.get_match_index() {
            if let Some(opt) = self.blocks.get(index) {
                if let Some(match_block) = opt {
                    return Some(match_block.clone());
                }
            }
        }
        None
    }

    fn get_match_index(&self) -> Option<usize> {
        if let Some(position) = self.match_position {
            return Some((position.1 * 4 + position.0).into());
        }
        None
    }

    fn set_match_position(&mut self, i: usize) {
        self.match_position = Some((
            (i % 4).try_into().expect("usize with incorrect value"),
            (i / 4).try_into().expect("usize with incorrect value"),
        ));
    }

    pub fn match_class(&self) -> String {
        if let Some(index) = self.get_match_index() {
            return format!("match__{}", index);
        }
        "".to_string()
    }

    fn unselect_block(&mut self, i: usize) {
        if let Some(opt) = self.blocks.get_mut(i) {
            if let Some(block) = opt {
                block.unselected();
            }
        }
    }

    fn select_block(&mut self, i: usize) {
        if self.is_game_on() {
            if let Some(opt) = self.blocks.get_mut(i) {
                if let Some(block) = opt {
                    block.selected();
                }
            }
        }
    }

    fn move_cursor(&mut self, new_position: Position) {
        if self.is_game_on() && new_position != self.cursor_position {
            // clear previous selection
            let i = self.get_cursor_index();
            self.unselect_block(i);
            // set new position
            self.cursor_position = new_position;
            // highlight the new block
            let i = self.get_cursor_index();
            self.select_block(i);
        }
    }

    fn incr_cursor_position(&mut self) {
        if self.is_game_on() {
            self.cursor_position = if self.cursor_position.0 < 3 {
                (self.cursor_position.0 + 1, self.cursor_position.1)
            } else {
                if self.cursor_position.1 < 3 {
                    (0, self.cursor_position.1 + 1)
                } else {
                    (0, 0)
                }
            };
        }
    }

    fn get_cursor_index(&self) -> usize {
        (self.cursor_position.1 * 4 + self.cursor_position.0).into()
    }

    fn set_cursor_position(&mut self, i: usize) {
        self.cursor_position = (
            (i % 4).try_into().expect("usize with incorrect value"),
            (i / 4).try_into().expect("usize with incorrect value"),
        );
    }

    fn match_succeed(&mut self) {
        if self.is_game_on() {
            self.incr_points();
            self.match_counter += 1;
        }
    }

    fn match_failed(&mut self) {
        if self.is_game_on() {
            self.decr_tries();
        }
    }

    fn incr_points(&mut self) {
        if self.is_game_on() {
            let base: u32 = 2;
            self.previous_points = self.points;
            self.points += DEFAULT_BASE_POINTS * base.pow(self.match_counter);
        }
    }

    fn incr_duration(&mut self) {
        if self.is_game_on() {
            self.duration += 1;
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
        if self.is_game_on() && self.game_help_status.is_available() {
            self.game_help_status = GameHelpStatus::On;
        }
    }

    fn show_details(&mut self) {
        if self.is_game_on() {
            let i = self.get_cursor_index();
            if let Some(opt) = self.blocks.get_mut(i) {
                if let Some(block) = opt {
                    // Note: only flip if an animation is not undergoing
                    if !block.is_anim_live() {
                        block.flipped();
                    }
                }
            }
        }
    }

    fn decr_help_matches(&mut self, v: u32) {
        if self.is_help_on() && self.helps > 0 {
            for _n in 0..v {
                self.helps -= 1;
                if self.helps == 0 {
                    self.game_help_status = GameHelpStatus::NotAvailable;
                    break;
                }
            }
        }
    }

    fn share_message(&self) -> Option<AttrValue> {
        let game_results = self.game_results().unwrap_or_default();
        if let Some(block) = &self.previous_match_block {
            let mut data = Vec::new();
            data.push(format!("■□ corematch.io {}\n", game_results));
            data.push(block.runtime.hashtag());
            Some(data.join("\n").into())
        } else {
            None
        }
    }

    fn game_results(&self) -> Option<AttrValue> {
        if let Some(block) = &self.previous_match_block {
            Some(format!("{}/{}/{}", self.points, self.duration, block.block_number).into())
        } else {
            None
        }
    }
}
