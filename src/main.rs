// Copyright © 2019 Bart Massey
// [This program is licensed under the GPL version 3 or later.]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

mod net;
mod field;

pub use net::*;
pub use field::*;

use std::borrow::BorrowMut;
use std::collections::HashMap;
pub use std::io::{self, Write};
pub use std::sync::{Arc, Mutex};

struct PlayerState {
    posn: usize,
    width: u16,
}

impl PlayerState {
    fn new(width: Option<u16>) -> Self {
        PlayerState {
            posn: 1,
            width: width.unwrap_or(80),
        }
    }
}

#[derive(Default)]
struct GameState {
    next_player_id: u64,
    players: HashMap<u64, PlayerState>,
    field: Field,
}

impl GameState {
    fn get_player(&mut self, player_id: u64) -> &mut PlayerState {
        self.players.get_mut(&player_id).unwrap()
    }
}

#[derive(Default, Clone)]
struct GameHandle(Arc<Mutex<GameState>>);

impl GameHandle {
    fn init_game(
        &mut self,
        mut action: impl FnMut(&mut GameState) -> u64,
    ) -> u64 {
        let mut state = self.0.borrow_mut().lock().unwrap();
        action(&mut state)
    }

    fn with_game(&mut self, mut action: impl FnMut(&mut GameState)) {
        let mut state = self.0.borrow_mut().lock().unwrap();
        action(&mut state);
    }

    pub fn play(mut self, mut remote: Connection) {
        let player_id = self.init_game(|game| {
            let player_id = game.next_player_id + 1;
            game.next_player_id = player_id;
            let player = PlayerState::new(remote.width);
            game.players.insert(player_id, player);
            player_id
        });
        loop {
            let optcmd = remote.read().unwrap();
            match optcmd {
                Some(cmd) => {
                    let cmd = cmd.trim();
                    match cmd {
                        "l" | "r" => self.with_game(|game| {
                            let player = game.get_player(player_id);
                            match &*cmd {
                                "l" if player.posn > 0 => player.posn -= 1,
                                "l" => (),
                                "r" => player.posn += 1,
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
