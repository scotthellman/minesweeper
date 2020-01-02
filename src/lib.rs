#[cfg(test)]
#[macro_use]
extern crate proptest;

pub mod board;
pub mod ai;
pub mod interaction;

use board::Point;


#[derive(Debug)]
pub enum ActionType {
    Click(Point),
    Chord(Point),
    Complete(Point),
    Flag(Point)
}

pub trait Agent {
    fn generate_move(&mut self, board: &board::Board) -> ActionType;
}

pub fn game_loop(agent: &mut impl Agent, board: &mut board::Board){
    while !board.is_won(){
        println!("{}", board);
        let mines = match agent.generate_move(board) {
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
