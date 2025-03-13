//! "Monster OBject" / "MOBile": MOB. This is the MOB state
//! and methods for manipulating it.

use crate::*;

/// MOB state.
pub struct Mob {
    /// MOB id. Globally unique across all sessions.
    pub id: u64,
    /// MOB position in field coordinates.
    pub posn: usize,
    /// MOB health.
    health: u64,
}

impl Mob {
    /// Make a new MOB with the given ID and position, and
    /// with random health.
    pub fn new(id: u64, posn: usize) -> Self {
        Mob {
            id,
            posn,
            health: random(3) + 3,
        }
    }

    /// Take a hit.
    pub fn hit(&mut self) -> bool {
        let hit = random(3);
        if hit >= self.health {
            self.health = 0;
            false
        } else {
            self.health -= hit;
            true
        }
    }

    /// MOB "AI". Heuristically decide how to move the MOB.
    /// Returns the new position.
    pub fn get_move(&self) -> usize {
        let posn = self.posn;
        let dirn = random(3);
        match dirn {
            0 => {
                if posn == 0 {
                    return posn;
                }
                posn - 1
            }
            1 => posn,
            2 => posn + 1,
            _ => panic!("internal error: weird mob move"),
        }
    }
}
