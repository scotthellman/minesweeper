use minesweeper;

fn main() {
    let width = 20;
    let height = 20;
    let mine_count = 60;
    let mut board = minesweeper::board::Board::new_from_ints(width, height, mine_count);
    let mut agent = minesweeper::ai::NaiveAI::new(1000);
    //let mut agent = minesweeper::interaction::HumanAgent{};
    minesweeper::game_loop(&mut agent, &mut board);
}
