use std::fmt;
use rand::thread_rng;
use rand::seq::SliceRandom;

#[derive(Debug)]
enum Content {
    Mine,
    Empty
}

#[derive(Debug)]
struct Cell {
    content: Content,
    neighbors: usize,
    known: bool
}

impl Cell {
    fn create_empty() -> Cell {
        Cell{content: Content::Empty, neighbors: 0, known: false}
    }

    fn is_null_cell(&self) -> bool {
        if self.neighbors > 0{
            return false
        }
        match self.content {
            Content::Empty => true,
            Content::Mine => false
        }
    }

    fn to_str(&self) -> String {
        if !self.known{
            return String::from("■")
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

#[derive(Debug, Eq)]
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

    fn retrieve_cell(&mut self, point: &Point) -> &mut Cell{
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
                let mut cell =  self.retrieve_cell(&neighbor);
                cell.neighbors += 1;
            }
        }
        self.initialized = true;
    }

    fn probe(&mut self, point: &Point){
        if !&self.initialized {
            self.initialize(point);
        }
        self.reveal_point(point);
    }

    fn reveal_point(&mut self, point: &Point){
        let was_null = {
            let mut cell = self.retrieve_cell(point);
            if cell.known {
                return 
            }
            cell.known = true;
            cell.is_null_cell()
        };
        if was_null {
            self.propagate_knowledge(point);
        }
    }

    fn propagate_knowledge(&mut self, point: &Point) {
        for neighbor in self.neighbor_points(point){
            self.reveal_point(&neighbor);
        }
    }

    fn to_string(&self) -> String {
        let mut result = "".to_owned();
        for row in self.field.iter(){
            for cell in row{
                result += &cell.to_str()[..];
            }
            result += "\n"
        }
        result
    }
}

fn sample_points(size: &BoardSize, n: usize, disallowed: &Point) -> Vec<Point>{
    // TODO: handle n > area
    let mut possible: Vec<usize> = (0..size.area()).collect();
    possible.shuffle(&mut thread_rng());
    possible.iter().map(|&x| size.point_from_integer(x).expect("bad size!"))
                   .filter(|x| *x != *disallowed).take(n).collect()
}

fn main() {
    let size = BoardSize{
        width:9,
        height:9
    };
    let mut board = Board::new_from_size(size, 10);
    println!("{}", board);
    board.probe(&Point(0,1));
    println!("{}", board);
    board.probe(&Point(0,2));
    println!("{}", board);
    board.probe(&Point(1,1));
    println!("{}", board);
}
