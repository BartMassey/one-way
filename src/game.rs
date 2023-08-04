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
    pub monsters: HashMap<u64, Mob>,
    pub next_monster_id: u64,
    pub health: u64,
}

impl Game {
    pub fn turn(&mut self) {
        self.turns += 1;

        let len = self.field.len();
        let nmonsters = self.monsters.len();
        if nmonsters < len / 20 && nmonsters < self.turns as usize / 5 {
            let posn = random(len as u64) as usize;
            if !self.field.has_object(posn) {
                let id = self.next_monster_id;
                self.next_monster_id += 1;
                self.field[posn].object = Some(Object::Monster(id));
                self.monsters.insert(id, Mob::new(id, posn));
            }
        }

        for (_, p) in self.players.iter() {
            for &posn in &[p.posn - 1, p.posn + 1] {
                if self.field.has_monster(posn) && self.health > 0 {
                    self.health -= 1;
                }
            }
        }

        for m in self.monsters.values_mut() {
            let posn = m.posn;
            let new_posn = m.get_move();
            if new_posn == posn {
                continue;
            }
            if self.field[new_posn].top().is_some() {
                continue;
            }
            assert_eq!(self.field[posn].top(), Some(&Object::Monster(m.id)));
            self.field[posn].object = None;
            self.field[new_posn].object = Some(Object::Monster(m.id));
            m.posn = new_posn;
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
            monsters: HashMap::default(),
            next_monster_id: 1,
            health: MAX_HEALTH,
        }
    }
}
