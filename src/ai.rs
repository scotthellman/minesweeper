use super::board::Board;
use super::board::KnowledgeState;
use super::board::Point;
use super::board::Content;
use super::ActionType;
use super::Agent;
use std::thread;
use std::time;

pub struct NaiveAI {
    move_queue: Vec<ActionType>,
    move_delay: u64
}

impl Agent for NaiveAI {
    fn generate_move(&mut self, board: &Board) -> ActionType {
        let move_delay = time::Duration::from_millis(self.move_delay);
        let start = time::Instant::now();

        match self.move_queue.pop(){
            Some(action) => action,
            None => {
                self.move_queue = NaiveAI::generate_next_moves(board);
                let now = time::Instant::now();
                let elapsed = now - start;
                println!("generated move in {:?}", elapsed);
                if move_delay > elapsed{
                    thread::sleep(move_delay - elapsed);
                }
                self.move_queue.pop().expect("something weird happened and we have no moves")
            }
        }
    }
}

impl NaiveAI {

    pub fn new(move_delay: u64) -> NaiveAI{
        let mut move_queue = Vec::with_capacity(4);
        move_queue.push(ActionType::Click(Point(0, 0)));
        NaiveAI{move_queue, move_delay}
    }

    pub fn generate_next_moves(board: &Board) -> Vec<ActionType>{
        let safe_flags = NaiveAI::known_safe_flags(board);
        if safe_flags.len() > 0{
            return safe_flags.iter().map(|point| ActionType::Flag(point.clone())).collect()
        }

        let safe_clicks = NaiveAI::known_safe_clicks(board);
        if safe_clicks.len() > 0{
            return safe_clicks.iter().map(|point| ActionType::Click(point.clone())).collect()
        }
        let probabilities = NaiveAI::get_naive_mine_probabilities(board);
        match NaiveAI::safest_frontier_click(board, probabilities){
            None => vec![],
            Some((point, proba)) => {
                println!("Naively estimating probability at {}", proba);
                vec![ActionType::Click(point)]
            }
        }
    }

    fn known_safe_flags(board: &Board) -> Vec<Point> {
        board.size.points().iter()
             .filter(|point| match board.retrieve_cell(point).knowledge{
                 KnowledgeState::Unknown => true,
                 _ => false
             })
             .map(|point| (point.clone(), NaiveAI::get_naive_mine_probability(board, &point, true)))
             .filter(|(_, proba)| *proba == 1.0)
             .map(|(point, _)| point.clone())
             .collect()
    }

    fn known_safe_clicks(board: &Board) -> Vec<Point> {
        board.size.points().iter()
             .filter(|point| board.retrieve_cell(point).knowledge.is_unknown())
             .map(|point| (point.clone(), NaiveAI::get_naive_mine_probability(board, &point, false)))
             .filter(|(_, proba)| *proba == 0.0)
             .map(|(point, _)| point.clone())
             .collect()
    }

    fn safest_frontier_click(board: &Board, point_probabilities: Vec<(Point, f32)>) -> Option<(Point, f32)>{
        point_probabilities.iter()
            .filter(|(point, _)| board.has_known_neighbors(point))
            .filter(|(_, proba)| *proba > 0.0 && *proba < 1.0 )
            .fold(None, |acc, (point, proba)| { //FIXME: painfully similar to the code in naive_mine_probability
                match acc {
                    None => Some((point.clone(), *proba)),
                    Some(acc) => {
                        let acc_proba = acc.1;
                        let result = {
                            if *proba == 0.0 {
                                (point.clone(), *proba)
                            } else {
                                if *proba < acc_proba {(point.clone(), *proba)} else {acc} //TODO: why do i need all these derefs
                                // oh i think it's iter vs into_iter
                            }
                        };
                        Some(result)
                    }
                }
            })
    }

    fn safest_click(point_probabilities: Vec<(Point, f32)>) -> Option<(Point, f32)>{
        point_probabilities.into_iter()
            .filter(|(_, proba)| *proba > 0.0 && *proba < 1.0 )
            .fold(None, |acc, (point, proba)| { //FIXME: painfully similar to the code in naive_mine_probability
                match acc {
                    None => Some((point.clone(), proba)),
                    Some(acc) => {
                        let acc_proba = acc.1;
                        let result = {
                            if proba == 0.0 {
                                (point.clone(), proba)
                            } else {
                                if proba < acc_proba {(point.clone(), proba)} else {acc}
                            }
                        };
                        Some(result)
                    }
                }
            })
    }

    fn get_naive_mine_probability(board: &Board, point: &Point, pessimistic: bool) -> f32 {
        let cell = board.retrieve_cell(point);
        if cell.knowledge.is_known() {
            return match cell.content {
                Content::Mine => 1.0,
                Content::Empty => 0.0
            }
        }
        if cell.knowledge.is_flag() {
            return 1.0
        }
        let probability = board.neighbor_cells_from_point(point).iter()
             .filter(|neighbor| neighbor.knowledge.is_known())
             .map(|neighbor| {
                 let flagged = board.count_flagged_neighbors(&neighbor.point);
                 let mined = board.count_assumed_mined_neighbors(&neighbor.point);
                 let unknown = board.count_unknown_neighbors(&neighbor.point);
                 (neighbor.mined_neighbor_count - mined) as f32/(unknown - flagged) as f32
             })
             .fold(None, |acc, proba| {
                 match acc {
                     None => Some(proba),
                     Some(acc) => {
                         let result = {
                             if proba == (pessimistic as usize) as f32 { //hmmmm
                                 proba
                             } else {
                                 if pessimistic {
                                     if proba > acc {proba} else {acc}
                                 }
                                 else {
                                     if proba < acc {proba} else {acc}
                                 }
                             }
                         };
                         Some(result)
                     }
                 }
             });
        match probability {
            Some(p) => p,
            None => (board.mine_count as f32) / (board.size.area() as f32)
        }
    }

    fn get_naive_mine_probabilities(board: &Board) -> Vec<(Point, f32)>{
        board.size.points().iter()
            .map(|point| (point.clone(), NaiveAI::get_naive_mine_probability(board, &point,false)))
            .collect()
    }
}
