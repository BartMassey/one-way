// Copyright © 2019 Bart Massey
// [This program is licensed under the GPL version 3 or later.]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

mod field;
mod mob;
mod net;
mod random;
mod player;

pub use field::*;
pub use mob::*;
pub use net::*;
pub use player::*;
pub use random::*;

use std::borrow::BorrowMut;
use std::collections::HashMap;
pub use std::io::{self, Write};
pub use std::sync::{Arc, Mutex};

#[derive(Default)]
struct Game {
    next_player_id: u64,
    players: HashMap<u64, Player>,
    field: Field,
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
            let mut player = Player::new(remote.width);
            let mut posn = player.posn;
            while game.field.has_object(posn) {
                posn += 1;
            }
            player.posn = posn;
            game.players.insert(player_id, player);
            game.field.establish(posn + Player::MARGIN);
            player_id
        });
        loop {
            let optcmd = remote.read().unwrap();
            if let Some(cmd) = optcmd {
                let cmd = cmd.trim();
                match cmd {
                    "h" | "l" => self.with_game(|game| {
                        let player =
                            game.players.get_mut(&player_id).unwrap();
                        let off = match cmd {
                            "h" => -1,
                            "l" => 1,
                            _ => panic!("internal error: bad cmd"),
                        };
                        let mut move_player = true;
                        if let Some(posn) = offset(player.posn, off) {
                            move_player = !game.field.collide(posn);
                        }
                        if move_player {
                            if player.move_player(off) {
                                game.field.establish(player.posn + Player::MARGIN);
                            }
                        }
                    }),
                    "q" => {
                        self.with_game(|game| {
                            game.players.remove(&player_id).unwrap();
                        });
                        return;
                    }
                    _ => (),
                }
            }
            self.with_game(|game| {
                let player = game.players.get(&player_id).unwrap();
                // Absolute position of player in field coords.
                let posn = player.posn;
                // Absolute position of left edge in field coords.
                let left = posn - player.left;
                // Width of display in characters.
                let width = player.width as usize;
                // Absolute position of right edge in field coords.
                let right = left + width;
                let mut board = game.field.render(left, right);
                assert_eq!(board.len(), width);
                for (_, p) in game.players.iter() {
                    if p.posn >= left && p.posn < right {
                        board[p.posn - left] = '@';
                    }
                }
                let render: String = board.into_iter().collect();
                write!(remote, "\r{}", render).unwrap();
                write!(remote, "\r{}", &render[0..posn - left])
                    .unwrap();
            });
        }
    }
}

fn main() {
    GameHandle::default().run();
}
