// Copyright © 2019 Bart Massey
// [This program is licensed under the GPL version 3 or later.]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

mod net;
mod field;
mod player;

pub use net::*;
pub use field::*;
pub use player::*;

use std::borrow::BorrowMut;
use std::collections::HashMap;
pub use std::io::{self, Write};
pub use std::sync::{Arc, Mutex};

#[derive(Default)]
struct Game {
    next_player_id: u64,
    players: HashMap<u64, usize>,
    field: Field,
}

impl Game {
    fn get_player(&mut self, player_id: u64) -> Option<&mut Player> {
        let &loc = self.players.get(&player_id)?;
        if let Some(Object::Hero(player)) = self.field[loc].object.as_mut() {
            Some(player)
        } else {
            panic!("player has gone missing");
        }
    }
}

#[derive(Default, Clone)]
struct GameHandle(Arc<Mutex<Game>>);

impl GameHandle {
    fn init_game(
        &mut self,
        mut action: impl FnMut(&mut Game) -> u64,
    ) -> u64 {
        let mut state = self.0.borrow_mut().lock().unwrap();
        action(&mut state)
    }

    fn with_game(&mut self, mut action: impl FnMut(&mut Game)) {
        let mut state = self.0.borrow_mut().lock().unwrap();
        action(&mut state);
    }

    pub fn play(mut self, mut remote: Connection) {
        let player_id = self.init_game(|game| {
            let player_id = game.next_player_id + 1;
            game.next_player_id = player_id;
            let player = Player::new(remote.width);
            let posn = player.posn;
            game.players.insert(player_id, posn);
            game.field.insert(Object::Hero(player), posn);
            player_id
        });
        loop {
            let optcmd = remote.read().unwrap();
            match optcmd {
                Some(cmd) => {
                    let cmd = cmd.trim();
                    match cmd {
                        "h" | "l" => self.with_game(|game| {
                            let player = game.get_player(player_id).unwrap();
                            match cmd {
                                "h" if player.posn > 0 => player.posn -= 1,
                                "h" => (),
                                "l" => player.posn += 1,
                                _ => panic!("internal error: bad cmd"),
                            }
                            write!(remote, "posn {:10}\r", player.posn)
                                .unwrap();
                        }),
                        "q" => {
                            self.with_game(|game| {
                                game.players.remove(&player_id).unwrap();
                            });
                            return;
                        }
                        c => write!(remote, "{}?\r", c).unwrap(),
                    }
                },
                None => write!(remote, ".\r").unwrap(),
            }
        }
    }
}

fn main() {
    GameHandle::default().run();
}
