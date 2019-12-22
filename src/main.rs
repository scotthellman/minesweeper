use std::collections::HashSet;
use std::fmt;
use std::io;
use regex::Regex;
use rand::thread_rng;
use rand::seq::SliceRandom;

enum ActionType {
    Click(Point),
    Chord(Point),
    Complete(Point),
    Flag(Point)
}

impl ActionType {
    fn from_string(input: &str) -> Option<ActionType>{
        let re = Regex::new(r"(click|flag|chord|complete)\s(\d+)\s(\d+)").unwrap();
        match re.captures_iter(input).next() {
            None => None,
            Some(cap) => {
                let x: usize = cap[2].parse().expect("Expected a number");
                let y: usize = cap[3].parse().expect("Expected a number");
                let point = Point(x, y);
                ActionType::extract_type_from_string(&cap[1], point)
            }
        }
    }

    fn extract_type_from_string(input: &str, point: Point) -> Option<ActionType>{
        // there must be a better way
        if input == "click"{
            Some(ActionType::Click(point))
        }
        else if input == "chord"{
            Some(ActionType::Chord(point))
        }
        else if input == "complete"{
            Some(ActionType::Complete(point))
        }
        else if input == "flag"{
            Some(ActionType::Flag(point))
        }
        else {
            None
        }
    }
}

fn get_move() -> ActionType {
    println!("Please input your move: TYPE X Y");
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read");
    match ActionType::from_string(&input) {
        Some(action) => action,
        None => {
            println!("Must be of the form: TYPE X Y");
            get_move()
        }
    }
}

#[derive(Debug)]
enum Content {
    Mine,
    Empty
}

#[derive(Debug)]
struct Cell {
    content: Content,
    neighbors: usize,
    known: bool,
    flagged: bool

}

impl Cell {
    fn create_empty() -> Cell {
        Cell{content: Content::Empty, neighbors: 0, known: false, flagged: false}
    }

    fn is_assumed_mine(&self) -> bool {
        match self.content {
            Content::Mine => {
                self.known || self.flagged
            }
            Content::Empty => {
                self.flagged
            }
        }
    }

    fn is_known_unmined(&self) -> bool {
        match self.content {
            Content::Empty => {
                self.known
            }
            _ => false

        }
    }

    fn to_str(&self) -> String {
        if self.flagged{
            return String::from("▶")
        }
        if !self.known{
            return String::from("□")
        }
        match self.content {
            Content::Mine => String::from("X"),
            Content::Empty => {
                if self.neighbors == 0{
                    String::from("_")
                }
                else{
                    self.neighbors.to_string()
                }
            }
        }
    }
}

struct BoardSize {
    width: usize,
    height: usize
}

impl BoardSize {
    fn area(&self) -> usize {
        return self.width * self.height;
    }

    fn point_from_integer(&self, x: usize) -> Option<Point> {
        //nominally induces an ordering, might be useful...
        if x >= self.area() {
            return None
        }
        return Some(Point(x/self.width, x%self.width))
    }
}

#[derive(Debug, Eq, Clone, Hash)]
struct Point(usize, usize);

impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1
    }
}

struct Board {
    size: BoardSize,
    field: Vec<Vec<Cell>>,
    mine_count: usize,
    initialized: bool
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl Board {
    fn new_from_size(size: BoardSize, mine_count: usize) -> Board {
        let initialized = false;
        let mut field = Vec::with_capacity(size.height);
        for _ in 0..size.height {
            let mut row_vec = Vec::with_capacity(size.width);
            for _ in 0..size.width {
                row_vec.push(Cell::create_empty());
            }
            field.push(row_vec);
        }

        Board {size, field, mine_count, initialized}
    }

    fn retrieve_cell(&self, point: &Point) -> &Cell{
        &self.field[point.0][point.1]
    }

    fn retrieve_cell_mutable(&mut self, point: &Point) -> &mut Cell{
        &mut self.field[point.0][point.1]
    }

    fn neighbor_points(&self, point: &Point) -> Vec<Point>{
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

    fn initialize(&mut self, point: &Point){
        for point in sample_points(&self.size, self.mine_count, point){
            self.field[point.0][point.1].content = Content::Mine;
            for neighbor in self.neighbor_points(&point){
                let mut cell =  self.retrieve_cell_mutable(&neighbor);
                cell.neighbors += 1;
            }
        }
        self.initialized = true;
    }

    fn toggle_flag(&mut self, point: &Point){
        let mut cell = self.retrieve_cell_mutable(point);
        if !cell.known{  // flag and known gate each other, it's a bit weird
            cell.flagged = !cell.flagged;
        }
    }

    fn flag_neighbors(&mut self, point: &Point){
        let cell = self.retrieve_cell(point);
        let neighbors = self.neighbor_points(point);
        let ungood_points: Vec<&Point> = neighbors.iter()
            .filter(|neighbor| !self.retrieve_cell(neighbor).is_known_unmined())
            .collect();
        if ungood_points.len() == cell.neighbors{
            for neighbor in ungood_points{
                self.retrieve_cell_mutable(neighbor).flagged = true;
            }
        }
    }

    fn count_assumed_mined_neighbors(&self, point: &Point) -> usize{
        self.neighbor_points(point).iter()
            .map(|neighbor| self.retrieve_cell(neighbor).is_assumed_mine() as usize)
            .sum()
    }

    fn chord(&mut self, point: &Point) -> usize{
        let cell = self.retrieve_cell(point);
        if !cell.known{
            return 0
        }
        let mut hits = 0;
        let assumed = self.count_assumed_mined_neighbors(point);
        println!("while chording we saw {} vs {}", assumed, cell.neighbors);
        if self.count_assumed_mined_neighbors(point) == cell.neighbors {
            for neighbor in self.neighbor_points(point){
                hits += self.probe(&neighbor);
            }
        }
        hits
    }

    fn probe(&mut self, point: &Point) -> usize{
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
                        self.retrieve_cell(point).known as usize
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
                if !cell.known && cell.neighbors == 0 {
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
        if !(cell.known || cell.flagged){
            cell.known = true;
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

    fn is_won(&self) -> bool {
        // ideally this wouldn't be computed every single time
        // for now winning means revealing every safe
        let total = self.size.area() - self.mine_count;
        let found = self.field.iter().flatten()
            .filter(|cell| cell.is_known_unmined())
            .count();
        total == found
    }
}

fn sample_points(size: &BoardSize, n: usize, disallowed: &Point) -> Vec<Point>{
    // TODO: handle n > area
    let mut possible: Vec<usize> = (0..size.area()).collect();
    possible.shuffle(&mut thread_rng());
    possible.iter().map(|&x| size.point_from_integer(x).expect("bad size!"))
                   .filter(|x| *x != *disallowed).take(n).collect()
}

fn game_loop(board: &mut Board){
    while !board.is_won(){
        println!("{}", board);
        let mines = match get_move() {
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

fn main() {
    let size = BoardSize{
        width:9,
        height:9
    };
    let mut board = Board::new_from_size(size, 5);
    game_loop(&mut board);
}
