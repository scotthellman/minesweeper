pub mod board;
pub mod ai;
mod interaction;
use std::thread;
use std::time;

use interaction::ActionType;

pub fn game_loop(board: &mut board::Board){
    while !board.is_won(){
        println!("{}", board);
        let mines = match interaction::get_move() {
            ActionType::Click(point) => {
                board.probe(&point)
            }
            ActionType::Flag(point) => {
                board.toggle_flag(&point);
                0
            }
            ActionType::Complete(point) => {
                board.flag_neighbors(&point);
                0
            }
            ActionType::Chord(point) => {
                board.chord(&point)
            }
        };
        if mines > 0 {
            break
        }
    }
    println!("{}", board);
    if board.is_won(){
        println!("you win!");
    }
    else{
        println!("you lose");
    }
}

// TODO: generalize this so AI and normal are the same main loop
pub fn ai_game_loop(board: &mut board::Board){
    while !board.is_won(){
        println!("{}", board);
        let ten_millis = time::Duration::from_millis(1000);
        thread::sleep(ten_millis);
        let moves = ai::generate_move(board);
        let mut mines = 0;
        for action in moves {
            mines = match action {
                    ActionType::Click(point) => {
                        board.probe(&point)
                    }
                    ActionType::Flag(point) => {
                        board.toggle_flag(&point);
                        0
                    }
                    ActionType::Complete(point) => {
                        board.flag_neighbors(&point);
                        0
                    }
                    ActionType::Chord(point) => {
                        board.chord(&point)
                    }
                };
            }
        if mines > 0{
            break;
        }
    }
    println!("{}", board);
    if board.is_won(){
        println!("you win!");
    }
    else{
        println!("you lose");
    }
}
