use regex::Regex;
use std::io;
use super::board::Board;
use super::board::Point;
use super::Agent;
use super::ActionType;

pub struct HumanAgent {
}

impl Agent for HumanAgent {
    fn generate_move(&mut self, board: &Board) -> ActionType {
        println!("Please input your move: TYPE X Y");
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read");
        match HumanAgent::action_from_string(&input) {
            Some(action) => action,
            None => {
                println!("Must be of the form: TYPE X Y");
                self.generate_move(board)
            }
        }
    }
}

impl HumanAgent {
    fn action_from_string(input: &str) -> Option<ActionType>{
        let re = Regex::new(r"(click|flag|chord|complete)\s(\d+)\s(\d+)").unwrap();
        match re.captures_iter(input).next() {
            None => None,
            Some(cap) => {
                let x: usize = cap[2].parse().expect("Expected a number");
                let y: usize = cap[3].parse().expect("Expected a number");
                let point = Point(x, y);
                HumanAgent::extract_type_from_string(&cap[1], point)
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
