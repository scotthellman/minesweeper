pub mod board;
mod interaction;

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
