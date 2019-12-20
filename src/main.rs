use rand::thread_rng;
use rand::seq::SliceRandom;

struct BoardSize {
    width: u32,
    height: u32
}

impl BoardSize {
    fn area(&self) -> u32 {
        return self.width * self.height;
    }

    fn point_from_integer(&self, x: u32) -> Option<Point> {
        //nominally induces an ordering, might be useful...
        if x >= self.area() {
            return None
        }
        return Some(Point(x/self.width, x%self.width))
    }
}

#[derive(Debug)]
struct Point(u32, u32);

fn sample_points(size: &BoardSize, n: usize) -> Vec<Point>{
    // handle n > area
    let mut possible: Vec<u32> = (0..size.area()).collect();
    possible.shuffle(&mut thread_rng());
    possible.iter().map(|&x| size.point_from_integer(x).expect("bad size!")).take(n).collect()
}

fn main() {
    let size = BoardSize{
        width:3,
        height:3
    };
    let sampled = sample_points(&size, 5);
    println!("{:?}", sampled)
}
