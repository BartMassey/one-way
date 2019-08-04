pub struct Player {
    pub posn: usize,
    pub width: u16,
}

impl Player {
    pub fn new(width: Option<u16>) -> Self {
        Player {
            posn: 1,
            width: width.unwrap_or(80),
        }
    }
}
