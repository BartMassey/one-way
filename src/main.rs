// Copyright © 2019 Bart Massey
// [This program is licensed under the GPL version 3 or later.]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

/*!
*One Way Out* (OWO) is a multiplayer telnet game. The game
*state is in a [Game] struct that is initialized at game
*start and managed by a [GameHandle] wrapper. The
*[GameHandle] implements the play loop in its [play()]
*function.

At startup, a new [GameHandle] is created. Then the
connection listener is started. As players connect, they are
placed into the game and given a dedicated client proxy.

OWO is a "turn-based" game, with the provision that the
world updates when *any* player acts. This "time only moves
when *someone* moves" model is a bit unusual. During the
turn, the game simulation steps forward a step, regardless
of the amount of elapsed time since the last turn.
*/

mod conn;
mod field;
mod game;
mod mob;
mod player;

pub use conn::*;
pub use field::*;
pub use game::*;
pub use mob::*;
pub use player::*;

use std::borrow::BorrowMut;
use std::collections::HashMap;
pub use std::io::{self, Write};
pub use std::sync::{Arc, Mutex};

use fastrand::u64 as random_u64;

pub fn random(r: u64) -> u64 {
    random_u64(0..r)
}

/// The "health" and other player characteristics are common
/// across all players in the instance; the individual
/// players are proxy avatars that have only their own
/// position and actions. This is the maximum health that
/// the player can have.
pub const MAX_HEALTH: u64 = 100;


/// The game is won by traversing this distance (in tiles)
/// to the exit door.
pub const DOOR_POSN: usize = 500;

/// This contains all of the game state during a game.  Its
/// refcount will go to zero only when the game is
/// over. Individual client proxies must lock it to act.
#[derive(Default, Clone)]
struct GameHandle(Arc<Mutex<Game>>);

impl GameHandle {
    /// Execute some game action code under the state lock.
    fn with_game<T>(&mut self, mut action: impl FnMut(&mut Game) -> T) -> T {
        let mut state = self.0.borrow_mut().lock().unwrap();
        action(&mut state)
    }

    /// The main play loop for a single player client avatar.
    ///
    /// Note that the "player" is an avatar of the single
    /// notional entity here. A player has a unique position
    /// and can take unique actions.  A player is associated
    /// with a unique remote connection.
    pub fn play(mut self, mut remote: Connection) {
        // Start the player as far to the left as feasible,
        // then set up their view.
        let player_id = self.with_game(|game| {
            let player_id = game.next_player_id;
            game.next_player_id = player_id + 1;
            let mut player = Player::new(player_id, remote.width);
            let mut posn = player.posn;
            while game.field.has_object(posn) {
                posn += 1;
            }
            player.posn = posn;
            game.players.insert(player_id, player);
            game.field[posn].object = Some(Object::Player(player_id));
            game.field.establish(posn + Player::MARGIN);
            player_id
        });

        // Read and execute player actions.
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
                        // Movement command.
                        let player = game.players.get_mut(&player_id).unwrap();
                        let off = match cmd {
                            "h" => -1,
                            "l" => 1,
                            _ => panic!("internal error: bad cmd"),
                        };

                        //  Act on command.
                        if let Some(new_posn) = offset(player.posn, off) {
                            let clear = match game.field[new_posn].top() {
                                Some(Object::Monster(id)) => {
                                    // Combat.
                                    let mob = game.monsters.get_mut(id).unwrap();
                                    if !mob.hit() {
                                        // Killed the monster.
                                        game.monsters.remove(id);
                                        game.field[new_posn].object = None;
                                        true
                                    } else {
                                        false
                                    }
                                }
                                // Movement blocked.
                                Some(_) => false,
                                // Just move.
                                _ => true,
                            };

                            // If successfully moved, set up position and view.
                            if clear {
                                player.adjust_display(off);
                                let posn = player.posn;
                                game.field.establish(new_posn + Player::MARGIN);
                                player.posn = new_posn;
                                game.field[posn].object = None;
                                game.field[new_posn].object = Some(Object::Player(player.id));
                            }
                        }
                    }),
                    // Rest.
                    "." => self.with_game(|game| game.rest()),
                    // Quit the game.
                    "q" => {
                        self.with_game(|game| {
                            let player = &game.players[&player_id];
                            game.field[player.posn].object = None;
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
                    // Ignore random commands.
                    _ => continue,
                }

                // Run the rest of the game turn.
                self.with_game(|game| game.turn());
            }

            // Check for player dead or game win.
            let done = self.with_game(|game| {
                if game.health == 0 {
                    // Only one player, and they died.
                    writeln!(remote, "\rboard wipe, game over    \r").unwrap();
                    game.players.remove(&player_id).unwrap();
                    if game.players.is_empty() {
                        *game = Game::default();
                    }
                    return true;
                }
                let player = game.players.get(&player_id).unwrap();
                if player.posn >= DOOR_POSN {
                    // This player avatar escaped the game.
                    game.players.remove(&player_id).unwrap();
                    if game.players.is_empty() {
                        // Every player avatar escaped the game.
                        writeln!(remote, "\ry'all escaped, win!    \r").unwrap();
                        *game = Game::default();
                        return true;
                    }
                    writeln!(remote, "\ryou escaped, one down    \r").unwrap();
                    return true;
                }

                // Render player scene.
                //
                // Absolute position of player in field coords.
                let posn = player.posn;
                // Absolute position of left edge in field coords.
                let left = posn - player.left;
                // Width of display in characters.
                let width = player.width as usize;
                // Absolute position of right edge in field coords.
                let right = left + width;
                // Render player board view.
                let mut board = game.field.render(left, right);

                // Render player icon.
                assert_eq!(board.len(), width);
                for (_, p) in game.players.iter() {
                    if p.posn >= left && p.posn < right {
                        board[p.posn - left] = '@';
                    }
                }

                // Set up the render and send it.
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

            // Player and maybe game over.
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
