#![no_std]

use gstd::{exec::random,debug, msg, prelude::*};
use pebbles_game_io::*;

static  mut PEBBLES_GAME: Option<GameState> = None;

#[no_mangle]
extern "C" fn init() {
    let init_message: PebblesInit = msg::load().expect("Unsuccessful to load init message!");
    let subject: [u8; 32] = array::from_fn(|i| i as u8 + 1);
    let first_player =  match random(subject) {
        Ok((_, num)) => {
            if num % 2 == 0 {
                Player::User
            } else {
                Player::Program
            }
        }
        Err(_) => {
            Player::Program
        }
    };
    let game_state = GameState{
        pebbles_count: init_message.pebbles_count,
        max_pebbles_per_turn: init_message.max_pebbles_per_turn,
        pebbles_remaining: init_message.pebbles_count,
        difficulty: init_message.difficulty,
        first_player,
        winner: None,
    };

    unsafe {PEBBLES_GAME = Some(game_state)};
}

#[no_mangle]
extern "C" fn handle() {
    let action: PebblesAction = msg::load().expect("Unsuccessful to load message!");

    fn get_random_u32(min_value: Option<u32>, max_value: Option<u32>, game_state: &GameState) -> u32 {
        if game_state.pebbles_remaining == 0 {
            return 0;
        }

        let salt = msg::id();
        let (hash, _num) = random(salt.into()).expect("get_random_u32_in_range(): random call failed");
        let random_number = u32::from_le_bytes([hash[0], hash[1], hash[2], hash[3]]);

        let base_random = match (min_value, max_value) {
            (Some(min), Some(max)) => (random_number % (max - min + 1)) + min,
            (Some(min), None) => random_number.max(min),
            (None, Some(max)) => random_number % (max + 1),
            (None, None) => random_number,
        };

        let base_random = base_random.min(game_state.pebbles_remaining); 

        match game_state.difficulty {
            DifficultyLevel::Hard => {
                let max_pebbles_plus_one = game_state.max_pebbles_per_turn + 1;
                let candidate_value = ((base_random / max_pebbles_plus_one) + 1) * max_pebbles_plus_one;

                let candidate_value = candidate_value.min(game_state.pebbles_remaining);
                if let (Some(min), Some(max)) = (min_value, max_value) {
                    if candidate_value >= min && candidate_value <= max {
                        return candidate_value;
                    }
                } else if let Some(min) = min_value {
                    if candidate_value >= min {
                        return candidate_value;
                    }
                } else if let Some(max) = max_value {
                    if candidate_value <= max {
                        return candidate_value;
                    }
                }
                base_random
            },
            DifficultyLevel::Easy => base_random,
        }.min(game_state.pebbles_remaining).min(max_value.unwrap_or(u32::MAX)).max(min_value.unwrap_or(0))
    }

    let game_state = unsafe { PEBBLES_GAME.as_mut().expect("Game is not initialized") };

    match action {
        PebblesAction::Turn(pebbles) => {
            debug!("User pebbles {pebbles}");
            if pebbles < 1 || pebbles > game_state.max_pebbles_per_turn {
                msg::reply(PebblesEvent::InvalidTurn, 0).expect("Unsuccessful to reply");
                return;
            }

            if pebbles > game_state.pebbles_remaining {
                msg::reply(PebblesEvent::InvalidTurn, 0).expect("Unsuccessful to reply");
                return;
            }

            game_state.pebbles_remaining -= pebbles;

            if game_state.pebbles_remaining == 0 {
                game_state.winner = Some(Player::User);  
                msg::reply(PebblesEvent::Won(Player::User), 0).expect("Unsuccessful to reply");
                return;
            }

            let program_pebbles = match game_state.difficulty {
                DifficultyLevel::Easy => get_random_u32(Some(1), Some(game_state.max_pebbles_per_turn),game_state),
                DifficultyLevel::Hard => {
                    let program_pebbles = get_random_u32(Some(1), Some(game_state.max_pebbles_per_turn),game_state);
                    debug!("program pebbles {program_pebbles}");
                    program_pebbles
                }
            };
            game_state.pebbles_remaining -= program_pebbles;

            if game_state.pebbles_remaining == 0 {
                game_state.winner = Some(Player::Program);
                msg::reply(PebblesEvent::Won(Player::Program), 0).expect("Unsuccessful to reply");
            } else{
                msg::reply(PebblesEvent::CounterTurn(program_pebbles), 0).expect("Unsuccessful to reply");
            }
        }
        PebblesAction::GiveUp => {
            game_state.winner = Some(Player::Program);
            msg::reply(PebblesEvent::Won(Player::Program), 0).expect("Unable to reply");
        }
        PebblesAction::Restart { difficulty, pebbles_count, max_pebbles_per_turn } => {
            game_state.pebbles_count = pebbles_count;
            game_state.max_pebbles_per_turn = max_pebbles_per_turn;
            game_state.pebbles_remaining = pebbles_count;
            game_state.difficulty = difficulty;
            let subject: [u8; 32] = array::from_fn(|i| i as u8 + 1);
            game_state.first_player = match random(subject) {
                Ok((_, num)) => {
                    if num % 2 == 0 {
                        Player::User
                    } else {
                        Player::Program
                    }
                }
                Err(_) => {
                    Player::Program
                }
            };
            game_state.winner = None;
        }
    }
}

#[no_mangle]
extern "C" fn state() {
    let game_state = unsafe {
        PEBBLES_GAME.as_ref().expect("Game has not initialized!")
    };
    msg::reply(game_state, 0).expect("Unsuccessful to reply!");
}