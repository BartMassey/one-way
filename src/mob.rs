use crate::*;

pub struct Mob {
    pub id: u64,
    pub posn: usize,
    health: u64,
}

impl Mob {
    pub fn new(id: u64, posn: usize) -> Self {
        Mob {
            id,
            posn,
            health: random(3) + 3,
        }
    }

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
