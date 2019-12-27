use minesweeper;

fn main() {
    let width = 9;
    let height = 9;
    let mine_count = 10;
    let mut board = minesweeper::board::Board::new_from_ints(width, height, mine_count);
    let mut agent = minesweeper::ai::NaiveAI::new();
    let mut agent = minesweeper::interaction::HumanAgent{};
    minesweeper::game_loop(&mut agent, &mut board);
}
