pub struct Player {
    // Position in field coordinates.
    pub posn: usize,
    // Line width in characters.
    pub width: u16,
    // Offset of player in range 0..width.
    pub left: usize,
    pub display_cache: String,
    pub posn_cache: usize,
}

pub fn offset(x: usize, dx: isize) -> Option<usize> {
    if dx < 0 && (x as isize) < -dx {
        None
    } else {
        Some((x as isize + dx) as usize)
    }
}

impl Player {
    // Desired left/right margin in characters.
    pub const MARGIN: usize = 3;

    pub fn new(width: Option<u16>) -> Self {
        Player {
            posn: 1,
            left: 1,
            width: width.unwrap_or(80),
            display_cache: String::new(),
            posn_cache: 0,
        }
    }

    pub fn move_player(&mut self, dirn: isize) -> bool {
        if let Some(posn) = offset(self.posn, dirn) {
            self.posn = posn;
            if let Some(left) = offset(self.left, dirn) {
                let margin = Player::MARGIN;
                self.left = left
                    .min(self.width as usize - margin)
                    .max(margin)
                    .min(posn);
            } else {
                self.left = 0;
            }
            true
        } else {
            false
        }
    }
}
