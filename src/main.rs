use minesweeper;

fn main() {
    let width = 9;
    let height = 9;
    let mine_count = 3;
    let mut board = minesweeper::board::Board::new_from_ints(width, height, mine_count);
    minesweeper::ai_game_loop(&mut board);
}
