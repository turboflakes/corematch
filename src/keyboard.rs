use log::{debug, info};

#[derive(Debug, Clone, PartialEq)]
pub enum SupportedKeys {
    Up,
    Down,
    Left,
    Right,
    Enter,
    Space,
    N1,
    N2,
    N3,
    N4,
    NotSupported,
}

impl From<String> for SupportedKeys {
    fn from(v: String) -> Self {
        match v.as_str() {
            "ArrowUp" => Self::Up,
            "ArrowDown" => Self::Down,
            "ArrowLeft" => Self::Left,
            "ArrowRight" => Self::Right,
            "Enter" => Self::Enter,
            " " => Self::Space,
            "1" => Self::N1,
            "2" => Self::N2,
            "3" => Self::N3,
            "4" => Self::N4,
            _ => Self::NotSupported,
        }
    }
}
