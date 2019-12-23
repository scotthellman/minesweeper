use minesweeper;

fn main() {
    let width = 9;
    let height = 9;
    let mine_count = 5;
    let mut board = minesweeper::board::Board::new_from_ints(width, height, mine_count);
    minesweeper::game_loop(&mut board);
}
