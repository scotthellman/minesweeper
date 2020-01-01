use rand::thread_rng;
use rand::seq::SliceRandom;
use super::board::Board;
use super::board::KnowledgeState;
use super::board::Point;
use super::board::Content;
use super::ActionType;
use super::Agent;
use std::thread;
use std::time;
use std::collections::HashSet;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Constraint{
    missing_mines: usize,
    missing_empties: usize,
    constrained_points: HashSet<Point>
}

impl Constraint {
    fn decrement(&mut self, is_mine: bool){
        match is_mine {
            false => self.missing_empties -= 1,
            true => self.missing_mines -= 1
        }
    }

    fn increment(&mut self, is_mine: bool){
        match is_mine {
            false => self.missing_empties += 1,
            true => self.missing_mines += 1
        }
    }
}

pub struct ConstraintFrontier{
    points: Vec<Point>,
    missing_mines: usize,
    constraints: Vec<Constraint>
}

impl ConstraintFrontier{
    fn from_board(board: &Board) -> ConstraintFrontier{
        //identify frontier
        //build constraints
        let points: Vec<Point> = board.size.points().iter()
            .map(|point| board.retrieve_cell(point))
            .filter(|cell| !cell.knowledge.is_known() && board.has_known_neighbors(&cell.point))
            .map(|cell| cell.point.clone()) //TODO: not entirely sure why i'm not just using copy?
            .collect();

        let border_points: Vec<Point> = board.size.points().iter()
            .map(|point| board.retrieve_cell(point))
            .filter(|cell| cell.knowledge.is_known() && board.has_unknown_neighbors(&cell.point))
            .map(|cell| cell.point.clone()) //TODO: not entirely sure why i'm not just using copy?
            .collect();

        let remaining = board.remaining_mines();
        let missing_mines = {
            if remaining >= 0 {remaining as usize}
            else {0}
        };
        let constraints = ConstraintFrontier::build_constraints(board, &border_points);
        ConstraintFrontier{points, missing_mines, constraints}
    }

    fn build_constraints(board: &Board, points: &Vec<Point>) -> Vec<Constraint>{
        //each known border point imposes a constraint
        points.iter()
            .map(|point| {
                let cell = board.retrieve_cell(point);
                let unknown_neighbors: Vec<Point> = board.neighbor_cells_from_point(point).iter()
                    .filter(|cell| cell.knowledge.is_unknown())
                    .map(|cell| cell.point.clone())
                    .collect();
                let total_unknown = unknown_neighbors.len();
                let known_mines = board.count_assumed_mined_neighbors(point);
                let missing_mines = cell.mined_neighbor_count - known_mines;
                let missing_empties = total_unknown - missing_mines;
                let constrained_points: HashSet<Point> = unknown_neighbors.iter().map(|p| p.clone()).collect();
                println!("For {:?} we think there are {} mines and {} empties", point, missing_mines, missing_empties);
                Constraint{missing_mines, missing_empties, constrained_points}
            }).collect()
    }

    fn shuffled_frontier_points(&self) -> Vec<Point>{
        let mut frontier: Vec<Point> = self.points.iter().map(|p| p.clone()).collect();
        frontier.shuffle(&mut thread_rng());
        frontier
    }

    fn backtracking_search(&mut self, available_points: &[Point]) -> Option<Vec<Point>>{
        //a very naive backtracking implementation
        if self.missing_mines == 0{
            return None
        }
        match available_points.first(){
            None => Some(Vec::with_capacity(16)),
            Some(point) => {
                let possible = self.satisfying_assignments(&point);
                if possible.len() == 0 {
                    return None
                }
                for state in possible {
                    self.update_point(point, state);
                    match self.backtracking_search(&available_points[1..]){
                        None => self.undo_update(point, state),
                        Some(mut children) => {
                            if state{
                                children.push(point.clone());
                            }
                            return Some(children)
                        }
                    }
                }
                return None
            }
        }
    }

    fn satisfying_assignments(&self, point: &Point) -> Vec<bool> {
        let (can_be_mine, can_be_empty) = self.constraints.iter()
            //.filter(|constraint| constraint.missing_mines == 0 || constraint.missing_empties == 0)
            .filter(|constraint| constraint.constrained_points.contains(point))
            .fold((true, true), |acc, constraint| {
                ((acc.0 && constraint.missing_mines > 0), (acc.1 && constraint.missing_empties > 0))
            });
        let mut result = Vec::with_capacity(2);
        if can_be_mine {result.push(true)};
        if can_be_empty {result.push(false)};
        result
    }

    fn update_point(&mut self, point: &Point, mine: bool){
        // TODO: why did i decide to use bools instead of an enum
        // satisfying assignment should happen here. i'm just assuming it was called for now
        for i in 0..self.constraints.len(){
            if self.constraints[i].constrained_points.contains(point){
                self.constraints[i].decrement(mine);
            }
        }
        if mine {self.missing_mines -= 1};
    }

    fn undo_update(&mut self, point: &Point, mine: bool){
        for i in 0..self.constraints.len(){
            if self.constraints[i].constrained_points.contains(point){
                self.constraints[i].increment(mine);
            }
        }
        if mine {self.missing_mines += 1};
    }
}

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
        //let probabilities = NaiveAI::get_naive_mine_probabilities(board);
        let probabilities = NaiveAI::get_monte_carlo_probabilities(board);
        println!("probs are {:?}", probabilities);
        let mut actions: Vec<ActionType> = Vec::with_capacity(1);
        let mut highest_proba = 0.0;
        let mut best_point: Option<Point> = None;
        for (point, proba) in probabilities{
            if proba > highest_proba {
                highest_proba = proba;
                best_point = Some(point);
            }
            if proba == 0.0{
                actions.push(ActionType::Click(point));
            }
            if proba == 1.0{
                actions.push(ActionType::Flag(point));
            }
        }
        if actions.len() == 0{
            actions.push(ActionType::Flag(best_point.expect("so we just didn't have anything or something?")))
        }
        actions
        /*match NaiveAI::safest_frontier_click(board, probabilities){
            None => vec![],
            Some((point, proba)) => {
                println!("Naively estimating probability at {}", proba);
                vec![ActionType::Click(point)]
            }
        }
        */
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

    fn get_monte_carlo_probabilities(board: &Board) -> Vec<(Point, f32)>{
        // TODO: ok so this isn't really naive anymore is it
        let mut counts: HashMap<Point, usize> = HashMap::new();
        let rollouts = 10;
        let mut border_points: Vec<Point> = Vec::with_capacity(0);
        for i in 0..rollouts{
            println!("on {} rollout", i);
            let mut frontier = ConstraintFrontier::from_board(board);
            let points = frontier.shuffled_frontier_points();
            border_points = points.iter().map(|p| p.clone()).collect();
            let mined = frontier.backtracking_search(&points).expect("got a none back");
            for mine in mined{
                if counts.contains_key(&mine){
                    *counts.get_mut(&mine).unwrap() += 1;
                }
                else{
                    counts.insert(mine, 1);
                }
            }
        }
        border_points.into_iter()
            .map(|point| {
                let count = counts.get(&point).unwrap_or(&0);
                (point, (*count as f32)/(rollouts as f32))
            })
            .collect()
    }
}
