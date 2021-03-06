use rand::thread_rng;
use rand::seq::SliceRandom;
use std::collections::HashSet;
use std::collections::HashMap;
use std::fmt;
use itertools::Itertools;

#[derive(Debug, Clone)]
pub enum Content {
    Mine,
    Empty
}

#[derive(Debug, Clone)]
pub enum KnowledgeState {
    Unknown,
    Flag,
    Known
}

impl KnowledgeState {
    pub fn is_known(&self) -> bool{
        match *self {
            KnowledgeState::Known => true,
            _ => false
        }
    }

    pub fn is_flag(&self) -> bool{
        match *self {
            KnowledgeState::Flag => true,
            _ => false
        }
    }

    pub fn is_unknown(&self) -> bool{
        match *self {
            KnowledgeState::Unknown => true,
            _ => false
        }
    }
}

#[derive(Debug)]
pub struct Cell {
    pub content: Content,
    pub mined_neighbor_count: usize,
    pub knowledge: KnowledgeState,
    pub point: Point
}

impl Cell {
    fn create_empty(point: Point) -> Cell {
        Cell{content: Content::Empty, mined_neighbor_count: 0, knowledge: KnowledgeState::Unknown, point}
    }

    pub fn toggle_flag(&mut self){
        let new_state = match self.knowledge {
            KnowledgeState::Known => KnowledgeState::Known,
            KnowledgeState::Flag => KnowledgeState::Unknown,
            KnowledgeState::Unknown => KnowledgeState::Flag
        }; 
        self.knowledge = new_state;
    }

    fn is_assumed_mine(&self) -> bool {
        match (&self.knowledge, &self.content) {
            (KnowledgeState::Unknown, _) => false,
            (KnowledgeState::Flag, _) => true,
            (KnowledgeState::Known, Content::Mine) => true,
            _ => false
        }
    }

    pub fn is_known_unmined(&self) -> bool {
        match (&self.knowledge, &self.content) {
            (KnowledgeState::Known, Content::Empty) => true,
            _ => false
        }
    }

    fn to_str(&self) -> String {
        match (&self.knowledge, &self.content) {
            (KnowledgeState::Flag, _) => String::from("▶"),
            (KnowledgeState::Unknown, _) => String::from("□"),
            (_, Content::Mine) => String::from("X"),
            (_, Content::Empty) => {
                if self.mined_neighbor_count == 0{
                    String::from("_")
                }
                else{
                    self.mined_neighbor_count.to_string()
                }
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash, Copy)]
pub struct Point(pub usize, pub usize);

impl Point {
    pub fn distance(&self, other: &Point) -> usize{
        //l-inf norm seems most appropriate for minesweeper
        (self.0 as i64 - other.0 as i64).abs().max((self.1 as i64 - other.1 as i64).abs()) as usize
    }

}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Point({},{})", self.0, self.1)
    }
}

pub struct BoardSize {
    width: usize,
    height: usize
}

impl BoardSize {
    pub fn area(&self) -> usize {
        self.width * self.height
    }

    pub fn points(&self) -> Vec<Point> {
        (0..self.area()).filter_map(|x| self.point_from_integer(x)).collect()
    }

    pub fn point_from_integer(&self, x: usize) -> Option<Point> {
        if x >= self.area() {
            return None
        }
        Some(Point(x/self.width, x%self.width))
    }

    pub fn integer_from_point(&self, point: &Point) -> Option<usize> {
        let x = point.0*self.width + point.1 % self.width;
        if x > self.area(){
            None
        } else {
            Some(x)
        }
    }

    pub fn point_is_in_bounds(&self, point: &Point) -> bool {
        self.integer_from_point(point).is_some()
    }
}

fn sample_points(size: &BoardSize, n: usize, disallowed: &Point, disallowed_radius: usize) -> Option<Vec<Point>>{
    let mut possible: Vec<usize> = (0..size.area()).collect();
    possible.shuffle(&mut thread_rng());
    let possible: Vec<Point> = possible.iter().map(|&x| size.point_from_integer(x).expect("bad size!"))
                   .filter(|x| disallowed.distance(x) > disallowed_radius).take(n).collect();
    if possible.len() == n {
        Some(possible)
    } else {
        None
    }
}

pub struct Board {
    pub size: BoardSize,
    field: Vec<Cell>,
    pub mine_count: usize,
    pub initialized: bool,
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string_with_probabilities(&[]))
    }
}

impl Board {
    pub fn new_from_ints(width: usize, height: usize, mine_count: usize) -> Option<Board>{
        let size = BoardSize{width, height};
        Board::new_from_size(size, mine_count)
    }

    pub fn new_with_mines(size: BoardSize, mines: &[Point]) -> Option<Board> {
        if mines.iter().filter(|point| !size.point_is_in_bounds(point)).count() > 0 {
            return None
        }
        let mut board = match Board::new_from_size(size, mines.len()){
            None => return None,
            Some(board) => board
        };
        board.initialized = true;
        mines.iter().for_each( |point| {
            board.set_point_as_mined(point);
        });
        Some(board)
    }

    pub fn new_from_size(size: BoardSize, mine_count: usize) -> Option<Board> {
        if mine_count > size.area() {return None}; //TODO: this is too liberal
        let initialized = false;
        let mut field = Vec::with_capacity(size.height);
        for i in 0..size.area() {
            let point = size.point_from_integer(i).expect("Somehow failed at constructing points on the board");
            field.push(Cell::create_empty(point));
        }

        Some(Board {size, field, mine_count, initialized})
    }


    pub fn retrieve_cell(&self, point: &Point) -> &Cell{
        let index = self.size.integer_from_point(point).expect("Bad point for retrieve_cell");
        &self.field[index]
    }

    fn retrieve_cell_mutable(&mut self, point: &Point) -> &mut Cell{
        let index = self.size.integer_from_point(point).expect("Bad point for retrieve_cell_mutable");
        &mut self.field[index]
    }

    // TODO: Ideally this is an iterator
    fn cells(&self) -> Vec<&Cell> {
        self.size.points().iter().map(|point| self.retrieve_cell(point)).collect()
    }

    pub fn unknown_count(&self) -> usize{
        self.cells().iter()
            .filter(|cell| !cell.knowledge.is_known())
            .count()
    }

    pub fn found_mines(&self) -> usize{
        self.field.iter()
            .filter(|cell| cell.is_assumed_mine())
            .count()
    }

    pub fn remaining_mines(&self) -> i32{
        self.mine_count as i32 - self.found_mines() as i32
    }

    pub fn neighbor_points(&self, point: &Point) -> Vec<Point>{
        let mut product = Vec::with_capacity(8);
        for i in -1..2{
            for j in -1..2{
                if i != 0 || j != 0 {
                    product.push((i, j))
                }
            }
        }
        product.iter()
               .map(|(x, y)| (x+(point.0 as i32), y+(point.1 as i32)))
               .filter(|(x, y)| *x >= 0 && *x < self.size.width as i32 && *y >= 0 && *y < self.size.height as i32)
               .map(|(x, y)| Point(x as usize, y as usize))
               .collect()
    }

    pub fn neighbor_cells_from_point(&self, point: &Point) -> Vec<&Cell>{
        self.neighbor_points(point).iter().map(|point| self.retrieve_cell(point)).collect()
    }

    pub fn neighbor_cells(&self, cell: &Cell) -> Vec<&Cell>{
        self.neighbor_cells_from_point(&cell.point)
    }

    fn set_point_as_mined(&mut self, point: &Point){
        {
            let mut cell =  self.retrieve_cell_mutable(point);
            cell.content = Content::Mine;
        }
        for neighbor in self.neighbor_points(&point){
            let mut cell =  self.retrieve_cell_mutable(&neighbor);
            cell.mined_neighbor_count += 1;
        }
    }

    fn initialize_from_point(&mut self, point: &Point){
        let mined_points = sample_points(&self.size, self.mine_count, point, 2).expect("failed to init mines");
        self.initialize_with_mines(&mined_points);
    }

    fn initialize_with_mines(&mut self, mined_points: &[Point]) {
        // At this point we are assuming that all the points are valid
        // which seems maybe not ideal?
        mined_points.iter().for_each(|point| self.set_point_as_mined(point));
        self.initialized = true;
    }

    pub fn get_unknown_points(&self) -> Vec<Point> {
        // TODO: very similar to get_border_points
        self.size.points().iter()
            .map(|point| self.retrieve_cell(point))
            .filter(|cell| cell.knowledge.is_unknown())
            .map(|cell| cell.point)
            .collect()
    }

    pub fn get_border_points(&self) -> Vec<Point>{
        self.size.points().iter()
            .map(|point| self.retrieve_cell(point))
            .filter(|cell| cell.knowledge.is_unknown() && self.has_known_neighbors(&cell.point))
            .map(|cell| cell.point)
            .collect()
    }

    pub fn toggle_flag(&mut self, point: &Point){
        self.retrieve_cell_mutable(point).toggle_flag()
    }

    pub fn flag_neighbors(&mut self, point: &Point){
        let cell = self.retrieve_cell(point);
        let neighbors = self.neighbor_points(point);
        let ungood_points: Vec<&Point> = neighbors.iter()
            .filter(|point| !self.retrieve_cell(point).is_known_unmined())
            .collect();
        if ungood_points.len() == cell.mined_neighbor_count{
            for neighbor in ungood_points{
                self.retrieve_cell_mutable(neighbor).knowledge = KnowledgeState::Flag;
            }
        }
    }

    pub fn has_unknown_neighbors(&self, point: &Point) -> bool{
        self.neighbor_cells_from_point(point).iter()
            .filter(|cell| !cell.knowledge.is_known())
            .count() > 0
    }

    pub fn has_known_neighbors(&self, point: &Point) -> bool{
        self.neighbor_cells_from_point(point).iter()
            .filter(|cell| cell.knowledge.is_known())
            .count() > 0
    }

    pub fn count_assumed_mined_neighbors(&self, point: &Point) -> usize{
        self.neighbor_cells_from_point(point).iter()
            .filter(|neighbor| neighbor.is_assumed_mine())
            .count()
    }

    pub fn count_known_neighbors(&self, point: &Point) -> usize {
        self.neighbor_cells_from_point(point).iter()
            .filter(|neighbor| neighbor.knowledge.is_known())
            .count()
    }

    pub fn count_flagged_neighbors(&self, point: &Point) -> usize {
        self.neighbor_cells_from_point(point).iter()
            .filter(|neighbor| neighbor.knowledge.is_flag())
            .count()
    }

    pub fn count_unknown_neighbors(&self, point: &Point) -> usize {
        self.neighbor_cells_from_point(point).iter()
            .filter(|neighbor| neighbor.knowledge.is_unknown())
            .count()
    } //FIXME: ok these three could all be one function that takes a Content arg

    pub fn known_flaggable_neighbors(&self, point: &Point) -> Vec<Point> {
        let assumed_mined_neighbor_count = self.count_assumed_mined_neighbors(point);
        let unknown_neighbor_count = self.count_unknown_neighbors(point);
        let mined_neighbor_count = self.retrieve_cell(point).mined_neighbor_count;
        let remaining_mines = mined_neighbor_count as i32 - assumed_mined_neighbor_count as i32;

        if remaining_mines == unknown_neighbor_count as i32 {
            self.neighbor_points(point).into_iter()
                .filter(|point| self.retrieve_cell(point).knowledge.is_unknown())
                .collect()
        } else {
            vec![]
        }
    }

    pub fn known_safe_neighbors(&self, point: &Point) -> Vec<Point> {
        //TODO: so this is basically known_flaggable_neighbors
        let assumed_mined_neighbor_count = self.count_assumed_mined_neighbors(point);
        let mined_neighbor_count = self.retrieve_cell(point).mined_neighbor_count;
        let remaining_mines = mined_neighbor_count as i32 - assumed_mined_neighbor_count as i32;

        if remaining_mines == 0 as i32 {
            self.neighbor_points(point).into_iter()
                .filter(|point| self.retrieve_cell(point).knowledge.is_unknown())
                .collect()
        } else {
            vec![]
        }
    }

    pub fn chord(&mut self, point: &Point) -> usize{
        let cell = self.retrieve_cell(point);
        if !cell.knowledge.is_known(){
            return 0
        }
        let mut hits = 0;
        if self.count_assumed_mined_neighbors(point) == cell.mined_neighbor_count {
            for neighbor in self.neighbor_points(point){
                hits += self.probe(&neighbor);
            }
        }
        hits
    }

    pub fn probe(&mut self, point: &Point) -> usize{
        if !&self.initialized {
            self.initialize_from_point(point);
        }

        // overall a lot of this seems bad
        let mut region = HashSet::with_capacity(16);
        region.insert(point.clone());
        self.find_region(point.clone(), &mut region);

        region.iter()
            .map(|point| match self.reveal_point(point).content{
                    Content::Mine => {
                        self.retrieve_cell(point).knowledge.is_known() as usize
                    },
                    Content::Empty => 0
                })
            .sum()
    }

    fn find_region(&self, point: Point, acc: &mut HashSet<Point>) {
        let neighbors = self.neighbor_points(&point);
        let cell = self.retrieve_cell(&point);
        if let Content::Empty = cell.content {
            if !cell.knowledge.is_known() && cell.mined_neighbor_count == 0 {
                for neighbor in neighbors{
                    if !acc.contains(&neighbor){
                        acc.insert(neighbor.clone());
                        self.find_region(neighbor, acc);
                    }
                }
            }
        };
    }

    fn reveal_point(&mut self, point: &Point) -> &Cell{
        let mut cell = self.retrieve_cell_mutable(point);
        if cell.knowledge.is_unknown(){
            cell.knowledge = KnowledgeState::Known;
        }
        cell
    }

    pub fn to_string_with_probabilities(&self, probabilities: &[(Point, f32)]) -> String {
        let proba_lookup: HashMap<Point, f32> = probabilities.iter()
            .map(|(p, f)| (*p, *f))
            .collect();
        let mut result = "  ".to_owned();
        for i in 0..self.size.width{
            result += &i.to_string()[..];
        }
        result += "\n";
        for i in 0..self.size.height{
            result += &i.to_string()[..];
            result += " ";
            for j in 0..self.size.width{
                let cell = self.retrieve_cell(&Point(i, j));
                let c = match proba_lookup.get(&cell.point){
                    None => cell.to_str(),
                    Some(p) => proba_to_char(*p)
                };
                result += &c[..];
            }
            result += "\n";
        }
        result
    }

    pub fn is_won(&self) -> bool {
        // ideally this wouldn't be computed every single time
        // for now winning means identifying every mine
        let total = self.mine_count;
        let found = self.found_mines();
        total == found
    }
}

fn proba_to_char(proba: f32) -> String{
    if proba == 0.0 {
        String::from("◌")
    } else if proba < 0.2 {
        String::from("-")
    } else if proba < 0.4 {
        String::from("=")
    } else if proba < 0.6 {
        String::from("▤")
    } else if proba < 0.8 {
        String::from("▦")
    } else if proba < 1.0 {
        String::from("▩")
    } else{
        String::from("●")
    }
}

#[cfg(test)]
use proptest::prelude::*;

#[cfg(test)]
mod cell_tests {
    use super::*;

    fn knowledge_states() -> [KnowledgeState; 3]{
        [KnowledgeState::Unknown, KnowledgeState::Flag, KnowledgeState::Known]
    }

    #[test]
    fn toggle_flag_correctness() {
        for start_state in knowledge_states().iter() {
            let mut cell = Cell::create_empty(Point(0, 0));
            cell.knowledge = start_state.clone();
            cell.toggle_flag();
            match (start_state, cell.knowledge){
                (KnowledgeState::Known, KnowledgeState::Known) => {},
                (KnowledgeState::Flag, KnowledgeState::Unknown) => {},
                (KnowledgeState::Unknown, KnowledgeState::Flag) => {},
                _ => panic!("got an unexpected toggle state")
            };
        }
    }
}

#[cfg(test)]
mod board_tests {
    use super::*;

    fn point_fits_on_board(point: &Point, board: &BoardSize) -> bool {
        point.0 < board.height && point.1 < board.width
    }

    fn valid_points_for_board(points: &[Point], board: &BoardSize) -> bool {
        // points should have length area() and every pair should appear once
        let points_count = points.len();
        if points.iter().any(|point| !point_fits_on_board(point, &board)) {
            return false
        }

        points.iter().dedup().count() == points_count
    }

    proptest! {
        #[test]
        fn area_correctness(width in 0..1000usize, height in 0..1000usize) {
            prop_assert_eq!(BoardSize{width, height}.area(), width * height);

        }

        #[test]
        fn point_from_integer_correctness(x in any::<usize>(), width in 0..1000usize, height in 0..1000usize) {
            let board = BoardSize{width, height};
            match board.point_from_integer(x) {
                None => prop_assert!(x >= width * height),
                Some(point) => {
                    prop_assert!(point.0 == x/width && point.0 < height);
                    prop_assert!(point.1 == x%width && point.1 < height);
                }
            }
        }

        #[test]
        fn test_points(width in 0..100usize, height in 0..100usize) {
            let board = BoardSize{width, height};
            let points = board.points();
            let points_count = points.len();
            prop_assert_eq!(points_count, board.area());
            valid_points_for_board(&points, &board);
        }

        #[test]
        fn distance_to_self_is_zero(x in any::<usize>(), y in any::<usize>()) {
            let point = Point(x, y);
            prop_assert_eq!(point.distance(&point), 0);
            prop_assert_eq!(point, point);
        }

        #[test]
        fn distance_is_symmetric(x1 in 0..1000usize, y1 in 0..1000usize,
                                 x2 in 0..1000usize, y2 in 0..1000usize) {
            let point1 = Point(x1, y1);
            let point2 = Point(x2, y2);
            prop_assert_eq!(point1.distance(&point2), point2.distance(&point1));
        }

        #[test]
        fn test_partial_eq(x1 in 0..1000usize, y1 in 0..1000usize,
                           x2 in 0..1000usize, y2 in 0..1000usize) {
            let point1 = Point(x1, y1);
            let point2 = Point(x2, y2);
            let distance = point1.distance(&point2);
            if point1 == point2 {
                prop_assert_eq!(distance, 0)
            } else {
                prop_assert_ne!(distance, 0)
            }
        }

        #[test]
        fn test_sample_points(width in 0..100usize, height in 0..100usize,
                              x in 0..100usize, y in 0..100usize,
                              num_mines in 0..10000usize, disallowed_radius in 0..100usize) {
            let boardsize = BoardSize{width, height};
            let point = Point(x, y);
            match sample_points(&boardsize, num_mines, &point, disallowed_radius){
                None => {
                    let failure_conditions = point_fits_on_board(&point, &boardsize)
                        || boardsize.area() < (disallowed_radius*2+1).pow(2) + num_mines;
                    prop_assert!(failure_conditions);
                },
                Some(points) => {
                    prop_assert_eq!(points.len(), num_mines);
                    valid_points_for_board(&points, &boardsize);
                }
            }
        }

        #[test]
        fn test_new_from_int(width in 0..100usize, height in 0..100usize, mine_count in 0..10000usize) {
            match Board::new_from_ints(width, height, mine_count) {
                None => { 
                    prop_assert!(mine_count > width * height);
                },
                Some(board) => {
                    prop_assert!(!board.initialized);
                    prop_assert_eq!(board.mine_count, mine_count);
                    prop_assert_eq!(board.size.width, width);
                    prop_assert_eq!(board.size.height, height);
                    let points: Vec<Point> = board.cells().into_iter().map(|c| c.point).collect();
                    prop_assert_eq!(points.len(), board.size.area());
                    prop_assert!(valid_points_for_board(&points, &board.size));
                }
            }
        }

        #[test]
        fn test_retrieve_cell(width in 0..100usize, height in 0..100usize) {
            let board = Board::new_from_ints(width, height, 0).unwrap();
            let points: Vec<Point> = board.cells().into_iter().map(|c| c.point).collect();
            for point in points {
                let retrieved_point = board.retrieve_cell(&point).point;
                prop_assert_eq!(point, retrieved_point);
            }
        }

        #[test]
        fn test_unknown_count(width in 1..20usize, height in 1..20usize) {
            let mut board = Board::new_from_ints(width, height, 0).unwrap();
            let points: Vec<Point> = board.cells().into_iter().map(|c| c.point).collect();
            let mut unknown_points = points.len();
            prop_assert_eq!(board.unknown_count(), unknown_points);
            for point in points {
                board.retrieve_cell_mutable(&point).knowledge = KnowledgeState::Known;
                unknown_points -= 1;
                prop_assert_eq!(board.unknown_count(), unknown_points);
            }
        }

        #[test]
        fn test_found_mines_and_remaining_mines(width in 1..20usize, height in 1..20usize) {
            let mine_count = 1;
            let mut board = Board::new_from_ints(width, height, mine_count).unwrap();
            let mut mine_count = mine_count as i32;
            let points: Vec<Point> = board.cells().into_iter().map(|c| c.point).collect();
            let mut flagged_points = 0;
            prop_assert_eq!(board.found_mines(), flagged_points);
            prop_assert_eq!(board.remaining_mines(), mine_count);
            for point in points {
                board.retrieve_cell_mutable(&point).knowledge = KnowledgeState::Flag;
                flagged_points += 1;
                mine_count -= 1;
                prop_assert_eq!(board.found_mines(), flagged_points);
                prop_assert_eq!(board.remaining_mines(), mine_count);
            }
        }

        #[test]
        fn test_neighbor_methods(width in 1..20usize, height in 1..20usize) {
            let mine_count = 1;
            let board = Board::new_from_ints(width, height, mine_count).unwrap();
            let points: Vec<Point> = board.size.points();
            for point in points {
                let all_distance_one = board.neighbor_points(&point).iter()
                    .all(|neighbor| (point.distance(neighbor) == 1));
                prop_assert!(all_distance_one);
            }
        }

    }
}
