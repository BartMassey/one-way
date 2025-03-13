//! Player avatar state and implementation. This is only
//! those attributes unique to each client.

/// Player avatar state.
pub struct Player {
    /// Player id. Globally unique across all sessions.
    pub id: u64,
    /// Position in field coordinates.
    pub posn: usize,
    /// Terminal line width in characters.
    pub width: u16,
    /// Offset of player in terminal view in range 0..width.
    pub left: usize,
    /// Player prior terminal view line. Used for refresh.
    pub display_cache: String,
    /// Player prior terminal position. Used for refresh.
    pub posn_cache: usize,
}

/// Offset `x` by `dx`. Return `None` if the offset would be
/// off the board, else the new position.
pub fn offset(x: usize, dx: isize) -> Option<usize> {
    if dx < 0 && (x as isize) < -dx {
        None
    } else {
        Some((x as isize + dx) as usize)
    }
}

impl Player {
    /// Desired left/right margin on terminal in characters.
    pub const MARGIN: usize = 3;

    /// Make a new player state with the given `id` and `width`.
    pub fn new(id: u64, width: Option<u16>) -> Self {
        Player {
            id,
            posn: 1,
            left: 1,
            width: width.unwrap_or(80),
            display_cache: String::new(),
            posn_cache: 0,
        }
    }

    /// Slide the terminal view window for the player to
    /// where it is supposed to be.
    pub fn adjust_display(&mut self, dirn: isize) {
        if let Some(posn) = offset(self.posn, dirn) {
            if let Some(left) = offset(self.left, dirn) {
                let margin = Player::MARGIN;
                self.left = left.min(self.width as usize - margin).max(margin).min(posn);
            } else {
                self.left = 0;
            }
        } else {
            panic!("tried to adjust display, badly");
        }
    }
}
