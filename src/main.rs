// Copyright © 2019 Bart Massey
// [This program is licensed under the GPL version 3 or later.]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

mod conn;
mod field;
mod game;
mod mob;
mod player;
mod random;

pub use conn::*;
pub use field::*;
pub use game::*;
pub use mob::*;
pub use player::*;
pub use random::*;

use std::borrow::BorrowMut;
use std::collections::HashMap;
pub use std::io::{self, Write};
pub use std::sync::{Arc, Mutex};

pub const MAX_HEALTH: u64 = 100;
pub const DOOR_POSN: usize = 500;

#[derive(Default, Clone)]
struct GameHandle(Arc<Mutex<Game>>);

impl GameHandle {
    fn with_game<T>(&mut self, mut action: impl FnMut(&mut Game) -> T) -> T {
        let mut state = self.0.borrow_mut().lock().unwrap();
        action(&mut state)
    }

    pub fn play(mut self, mut remote: Connection) {
        let player_id = self.with_game(|game| {
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
            let optcmd = match remote.read() {
                Ok(cmd) => cmd,
                Err(e) => {
                    eprintln!("net read error: {}", e);
                    return;
                }
            };
            if let Some(cmd) = optcmd {
                let cmd = cmd.trim();
                match cmd {
                    "h" | "l" => self.with_game(|game| {
                        let player = game.players.get_mut(&player_id).unwrap();
                        let off = match cmd {
                            "h" => -1,
                            "l" => 1,
                            _ => panic!("internal error: bad cmd"),
                        };
                        if let Some(posn) = offset(player.posn, off) {
                            if !game.field.collide(posn) && player.move_player(off) {
                                game.field.establish(player.posn + Player::MARGIN);
                            }
                        }
                    }),
                    "." => self.with_game(|game| game.rest()),
                    "q" => {
                        self.with_game(|game| {
                            game.players.remove(&player_id).unwrap();
                            if game.players.is_empty() {
                                writeln!(remote, "\rno more players, new game    \r").unwrap();
                                *game = Game::default();
                                return;
                            }
                            writeln!(remote, "\ryou quit, how sad    \r").unwrap();
                        });
                        return;
                    }
                    _ => continue,
                }
                self.with_game(|game| game.turn());
            }
            let done = self.with_game(|game| {
                if game.health == 0 {
                    writeln!(remote, "\rboard wipe, game over    \r").unwrap();
                    game.players.remove(&player_id).unwrap();
                    if game.players.is_empty() {
                        *game = Game::default();
                    }
                    return true;
                }
                let player = game.players.get(&player_id).unwrap();
                if player.posn >= DOOR_POSN {
                    game.players.remove(&player_id).unwrap();
                    if game.players.is_empty() {
                        writeln!(remote, "\ry'all escaped, win!    \r").unwrap();
                        *game = Game::default();
                        return true;
                    }
                    writeln!(remote, "\ryou escaped, one down    \r").unwrap();
                    return true;
                }
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
                let posn = player.posn;
                if posn != player.posn_cache || render != player.display_cache {
                    write!(remote, "\r{}", render).unwrap();
                    write!(remote, "\r{}", &render[0..posn - left]).unwrap();
                    let player = game.players.get_mut(&player_id).unwrap();
                    player.display_cache = render;
                    player.posn_cache = posn;
                }
                false
            });
            if done {
                return;
            }
        }
    }
}

impl RunConnection for GameHandle {
    fn run_connection(self, conn: Connection) {
        self.play(conn);
    }
}

fn main() {
    Connection::listen(GameHandle::default());
}
