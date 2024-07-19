use gtest::{Log, Program, System};
use pebbles_game_io::*;
use gstd::prelude::*;

#[test]
fn test_easy(){
    let sys = System::new();
    sys.init_logger();
    let program = Program::current(&sys);

    let user_id: u64 = 100001;

    // Init game（15 pebbles，3 pebbles per turn at most，easy）
    let result = program.send(
        user_id,
        PebblesInit {
            difficulty: DifficultyLevel::Hard,
            pebbles_count: 15,
            max_pebbles_per_turn: 3,
        },
    );

    // confirm Init
    assert!(!result.main_failed());

    let res = program.send(user_id, PebblesAction::Turn(3));
    assert!(!result.main_failed());

    // check player's operation
    let log = Log::builder().source(program.id()).dest(user_id).payload(PebblesAction::Turn(3));
    assert!(res.contains(&log));

    // game result
    let log = Log::builder().source(program.id()).dest(user_id).payload(PebblesEvent::Won(Player::User));
    println!("Expected log: {:?}", log);

    // check game state
    let log = Log::builder().source(program.id()).dest(user_id).payload(PebblesEvent::CounterTurn(3));
    println!("Expected log: {:?}", log);
    assert!(res.contains(&log));

    let log = Log::builder().source(program.id()).dest(user_id).payload(PebblesEvent::InvalidTurn);
    println!("Expected log: {:?}", log);

    // check game state 
    let state: GameState = program.read_state(()).expect("Failed to read state");
    assert_eq!(state.pebbles_remaining, 9);

    //player wins
    let log = Log::builder().source(program.id()).dest(user_id).payload(PebblesEvent::Won(Player::User));
    println!("Expected log: {:?}", log);

    // player quit
    let res = program.send(user_id, PebblesAction::GiveUp);
    assert!(!res.main_failed());
    let log = Log::builder().source(program.id()).dest(user_id).payload(PebblesEvent::Won(Player::Program));
    assert!(res.contains(&log));

    // restart game 
    let res = program.send(
        user_id,
        PebblesAction::Restart {
            difficulty: DifficultyLevel::Hard,
            pebbles_count: 20,
            max_pebbles_per_turn: 5,
        },
    );
    assert!(!res.main_failed());

    // check game state after restarting gmae
    let state: GameState = program.read_state(()).expect("Failed to read state");
    assert_eq!(state.pebbles_count, 20);
    assert_eq!(state.max_pebbles_per_turn, 5);
    assert_eq!(state.pebbles_remaining, 20);
    assert!(state.winner.is_none());

    let default_difficulty = DifficultyLevel::default(); 
    println!("{:?}", default_difficulty); // ouput：Easy

    let hard_difficulty = DifficultyLevel::Hard;
    println!("{:?}", hard_difficulty); // ouput：Hard
}

#[test]
fn test_hard(){
    let sys = System::new();
    sys.init_logger();
    let program = Program::current(&sys);

    let user_id: u64 = 100001;

    // Init game（10 pebbles，3 pebbles per turn at most，hard）
    let result = program.send(
        user_id,
        PebblesInit {
            difficulty: DifficultyLevel::Hard,
            pebbles_count: 4,
            max_pebbles_per_turn: 1,
        },
    );

    // Iinit game
    assert!(!result.main_failed());

    // player turn 1
    let state: GameState = program.read_state(()).expect("Failed to read state");
    println!("{:?}", state.pebbles_remaining);
    let res = program.send(user_id, PebblesAction::Turn(1));
    assert!(!res.main_failed());
    let state: GameState = program.read_state(()).expect("Failed to read state");
    println!("{:?}", state.pebbles_remaining);

    let res = program.send(user_id, PebblesAction::Turn(1));
    assert!(!res.main_failed());
    let state: GameState = program.read_state(()).expect("Failed to read state");
    println!("{:?}", state.pebbles_remaining);

    // check game state
    let state: GameState = program.read_state(()).expect("Failed to read state");
    println!("{:?}", state.winner);
}