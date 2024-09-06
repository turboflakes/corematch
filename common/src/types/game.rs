use crate::components::block::BlockView;
use crate::components::core::CoreView;
use crate::types::network::ParachainColors;

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
