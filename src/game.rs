// Copyright © 2019 Bart Massey
// [This program is licensed under the GPL version 3 or later.]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

use crate::*;

pub struct Game {
    pub next_player_id: u64,
    pub players: HashMap<u64, Player>,
    pub field: Field,
    pub turns: u64,
    pub nmonsters: u64,
    pub health: u64,
}

impl Game {
    pub fn turn(&mut self) {
        self.turns += 1;

        let len = self.field.len() as u64;
        let nmonsters = self.nmonsters;
        if nmonsters < len / 20 && nmonsters < self.turns / 5 {
            let posn = random(len) as usize;
            if !self.field.has_object(posn) {
                self.field[posn].object = Some(Object::Monster(Mob::default()));
                self.nmonsters += 1;
            }
        }

        for (_, p) in self.players.iter() {
            for &posn in &[p.posn - 1, p.posn + 1] {
                if self.field.has_monster(posn) && self.health > 0 {
                    self.health -= 1;
                }
            }
        }
    }

    pub fn rest(&mut self) {
        let health = self.health;
        self.health = MAX_HEALTH.min(health + random(2));
    }
}

impl Default for Game {
    fn default() -> Self {
        Game {
            next_player_id: 1,
            players: HashMap::default(),
            field: Field::default(),
            turns: 0,
            nmonsters: 0,
            health: MAX_HEALTH,
        }
    }
}
