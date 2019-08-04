use crate::*;

pub struct Mob {
    health: u64,
}

impl Default for Mob {
    fn default() -> Self {
        Mob {
            health: random(3) + 3,
        }
    }
}

impl Mob {
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
}
