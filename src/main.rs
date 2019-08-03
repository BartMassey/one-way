mod net;

pub use net::*;

use std::borrow::BorrowMut;
use std::collections::HashMap;
pub use std::io::{self, Write};
pub use std::sync::{Arc, Mutex};

#[derive(Default)]
struct PlayerState {
    posn: usize,
}

#[derive(Default)]
struct GameState {
    next_player_id: u64,
    players: HashMap<u64, PlayerState>,
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
            let player = PlayerState::default();
            game.players.insert(player_id, player);
            player_id
        });
        loop {
            let cmd = remote.read().unwrap();
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
        }
    }
}

fn main() {
    GameHandle::default().run();
}
