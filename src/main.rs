use std::fmt;
use rand::thread_rng;
use rand::seq::SliceRandom;

enum Content {
    Mine,
    Empty
}

struct Cell {
    content: Content,
    neighbors: usize,
    known: bool
}

impl Cell {
    fn create_empty() -> Cell {
        Cell{content: Content::Empty, neighbors: 0, known: false}
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

    fn neighbors(&self, point: &Point) -> Vec<Point>{
        let indices: [i32; 3] = [-1, 0, 1];
        indices.iter().zip(indices.iter())
               .filter(|(&x, &y)| x !=0 || y != 0)
               .map(|(&x, &y)| (x+(point.0 as i32), y+(point.1 as i32)))
               .filter(|(x, y)| *x >= 0 && *x < self.size.width as i32 && *y >= 0 && *y < self.size.height as i32)
               .map(|(x, y)| Point(x as usize, y as usize))
               .collect()
    }

    fn initialize(&mut self, point: &Point){
        for point in sample_points(&self.size, self.mine_count, point){
            self.field[point.0][point.1].content = Content::Mine;
            for neighbor in self.neighbors(&point){
                let mut cell =  self.retrieve_cell(&neighbor);
                cell.neighbors += 1;
            }
        }
    }

    fn probe(&mut self, point: &Point) -> Option<&Cell>{
        // this needs to return with a) what was there b) if progress was made
        if !&self.initialized {
            self.initialize(point);
        }
        let mut cell = self.retrieve_cell(point);
        if cell.known {
            return None
        }
        cell.known = true;
        Some(cell)
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
        width:3,
        height:3
    };
    let mut board = Board::new_from_size(size, 5);
    println!("{}", board);
    board.probe(&Point(0,1));
    println!("{}", board);
    board.probe(&Point(0,2));
    println!("{}", board);
    board.probe(&Point(1,1));
    println!("{}", board);
}
