use minesweeper;

fn main() {
    let width = 20;
    let height = 20;
    let mine_count = 50;
    let mut board = minesweeper::board::Board::new_from_ints(width, height, mine_count);
    let mut agent = minesweeper::ai::NaiveAI::new(500);
    //let mut agent = minesweeper::interaction::HumanAgent{};
    minesweeper::game_loop(&mut agent, &mut board);
}
