use minesweeper;

fn main() {
    let width = 9;
    let height = 9;
    let mine_count = 30;
    let mut board = minesweeper::board::Board::new_from_ints(width, height, mine_count).expect("no board!");
    let mut agent = minesweeper::ai::NaiveAI::new(10, 1000);
    //let mut agent = minesweeper::interaction::HumanAgent{};
    minesweeper::game_loop(&mut agent, &mut board);
}
