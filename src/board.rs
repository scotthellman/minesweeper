use rand::thread_rng;
use rand::seq::SliceRandom;
use std::collections::HashSet;
use std::fmt;

#[derive(Debug)]
pub enum Content {
    Mine,
    Empty
}

#[derive(Debug)]
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

    fn is_known_unmined(&self) -> bool {
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
#[derive(Debug, Eq, Clone, Hash)]
pub struct Point(pub usize, pub usize);

impl Point {
    pub fn distance(&self, other: &Point) -> usize{
        //l-inf norm seems most appropriate for minesweeper
        (self.0 as i64 - other.0 as i64).abs().max((self.1 as i64 - other.1 as i64).abs()) as usize
    }
}


impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1
    }
}

pub struct BoardSize {
    width: usize,
    height: usize
}

impl BoardSize {
    pub fn area(&self) -> usize {
        return self.width * self.height;
    }

    pub fn points(&self) -> Vec<Point> {
        (0..self.area()).filter_map(|x| self.point_from_integer(x)).collect()
    }

    pub fn point_from_integer(&self, x: usize) -> Option<Point> {
        if x >= self.area() {
            return None
        }
        return Some(Point(x/self.width, x%self.width))
    }
}

fn sample_points(size: &BoardSize, n: usize, disallowed: &Point, disallowed_radius: usize) -> Vec<Point>{
    // TODO: handle n > area
    let mut possible: Vec<usize> = (0..size.area()).collect();
    possible.shuffle(&mut thread_rng());
    possible.iter().map(|&x| size.point_from_integer(x).expect("bad size!"))
                   .filter(|x| disallowed.distance(x) > disallowed_radius).take(n).collect()
}
pub struct Board {
    pub size: BoardSize,
    field: Vec<Vec<Cell>>,
    pub mine_count: usize,
    pub initialized: bool,
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl Board {
    pub fn new_from_ints(width: usize, height: usize, mine_count: usize) -> Board{
        let size = BoardSize{width, height};
        Board::new_from_size(size, mine_count)
    }

    pub fn new_from_size(size: BoardSize, mine_count: usize) -> Board {
        let initialized = false;
        let mut field = Vec::with_capacity(size.height);
        for i in 0..size.height {
            let mut row_vec = Vec::with_capacity(size.width);
            for j in 0..size.width {
                row_vec.push(Cell::create_empty(Point(i, j)));
            }
            field.push(row_vec);
        }

        Board {size, field, mine_count, initialized}
    }

    pub fn retrieve_cell(&self, point: &Point) -> &Cell{
        &self.field[point.0][point.1]
    }

    fn retrieve_cell_mutable(&mut self, point: &Point) -> &mut Cell{
        &mut self.field[point.0][point.1]
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

    pub fn has_known_neighbors(&self, point: &Point) -> bool{
        self.neighbor_points(point).iter()
            .map(|point| self.retrieve_cell(point))
            .filter(|cell| cell.knowledge.is_known())
            .count() > 0
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

    pub fn neighbor_cells(&self, cell: &Cell) -> Vec<&Cell>{
        self.neighbor_points(&cell.point).iter().map(|point| self.retrieve_cell(point)).collect()
    }

    fn initialize(&mut self, point: &Point){
        for point in sample_points(&self.size, self.mine_count, point, 3){ //FIXME: hardcoding the radius
            self.field[point.0][point.1].content = Content::Mine;
            for neighbor in self.neighbor_points(&point){
                let mut cell =  self.retrieve_cell_mutable(&neighbor);
                cell.mined_neighbor_count += 1;
            }
        }
        self.initialized = true;
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

    pub fn count_assumed_mined_neighbors(&self, point: &Point) -> usize{
        self.neighbor_points(point).iter()
            .map(|neighbor| self.retrieve_cell(neighbor).is_assumed_mine() as usize)
            .sum()
    }

    pub fn count_known_neighbors(&self, point: &Point) -> usize {
        self.neighbor_points(point).iter()
            .map(|point| self.retrieve_cell(point))
            .filter(|neighbor| neighbor.knowledge.is_known())
            .count()
    }

    pub fn count_flagged_neighbors(&self, point: &Point) -> usize {
        self.neighbor_points(point).iter()
            .map(|point| self.retrieve_cell(point))
            .filter(|neighbor| neighbor.knowledge.is_flag())
            .count()
    }

    pub fn count_unknown_neighbors(&self, point: &Point) -> usize {
        self.neighbor_points(point).len() - self.count_known_neighbors(point)
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
            self.initialize(point);
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
        match cell.content {
            Content::Empty => {
                if !cell.knowledge.is_known() && cell.mined_neighbor_count == 0 {
                    for neighbor in neighbors{
                        if !acc.contains(&neighbor){
                            acc.insert(neighbor.clone());
                            self.find_region(neighbor, acc);
                        }
                    }
                }
            }
            _ => { }
        };
    }

    fn reveal_point(&mut self, point: &Point) -> &Cell{
        let mut cell = self.retrieve_cell_mutable(point);
        if cell.knowledge.is_unknown(){
            cell.knowledge = KnowledgeState::Known;
        }
        cell
    }

    fn to_string(&self) -> String {
        let mut result = "  ".to_owned();
        for i in 0..self.size.width{
            result += &i.to_string()[..];
        }
        result += "\n";
        for (i, row) in self.field.iter().enumerate() {
            result += &i.to_string()[..];
            result += " ";
            for cell in row{
                result += &cell.to_str()[..];
            }
            result += "\n";
        }
        result
    }

    pub fn is_won(&self) -> bool {
        // ideally this wouldn't be computed every single time
        // for now winning means revealing every safe
        let total = self.size.area() - self.mine_count;
        let found = self.field.iter().flatten()
            .filter(|cell| cell.is_known_unmined())
            .count();
        total == found
    }
}
