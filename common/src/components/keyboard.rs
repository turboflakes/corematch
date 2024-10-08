#[derive(Debug, Clone, PartialEq)]
pub enum SupportedKeys {
    // Arrow keys -> Move the cursor in the matrix
    Up,
    Down,
    Left,
    Right,
    // 'Enter' -> Validate if Cell selected is a pair
    Enter,
    // 'Space' -> Validate if Cell selected is a pair
    Space,
    // 'S' -> Start game
    S,
    // 'H' -> Help/highlight matches
    H,
    // 'F' -> Flip cell and show block details
    F,
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
            "Space" => Self::Space,
            " " => Self::Space,
            "S" => Self::S,
            "s" => Self::S,
            "H" => Self::H,
            "h" => Self::H,
            "F" => Self::F,
            "f" => Self::F,
            _ => Self::NotSupported,
        }
    }
}
