use rand::thread_rng;
use std::rc::Rc;
use std::marker::PhantomData;
use rand::seq::SliceRandom;
use super::board::Board;
use super::board::KnowledgeState;
use super::board::Point;
use super::board::Content;
use super::ActionType;
use super::Agent;
use super::constraint::Variable;
use super::constraint::Constraint;
use super::constraint::ConstraintSolver;
use std::thread;
use std::time;
use std::collections::HashSet;
use std::collections::HashMap;
use itertools::Itertools;

struct MineConstraint {
    expected_mines: i32,
    expected_empties: i32,
    constrained_points: Vec<Point>
}

impl Constraint<Point, bool> for MineConstraint {
    fn get_constrained_variable_ids(&self) -> Vec<Point>{
        self.constrained_points.iter().map(|s| *s).collect()
    }

    fn check_constraint(&self, variable_lookup: &HashMap<Point, Variable<Point, bool>>) -> bool {
        let (mined, empty, unknown) = self.constrained_points.iter()
            .map(|v_id| variable_lookup.get(v_id).expect("variable not in lookup"))
            .map(|variable| {
                match variable.value {
                    None => (0, 0, 1),
                    Some(val) => {
                        match val {
                            false => (0, 1, 0),
                            true => (1, 0, 0)
                        }
                    }
                }
            })
            .fold((0, 0, 0), |acc, next| (acc.0 + next.0, acc.1 + next.1, acc.2 + next.2));
        mined <= self.expected_mines && empty <= self.expected_empties
    }
}

fn construct_global_constraint(board: &Board, available_points: &Vec<Point>) -> MineConstraint{
    let expected_mines = board.remaining_mines(); //this is really just a number we can't exceed
    let expected_empties = board.unknown_count() as i32 - board.remaining_mines();
    let constrained_points = available_points.iter().map(|p| *p).collect();
    MineConstraint{expected_mines, expected_empties, constrained_points}
}


fn construct_constraint(board: &Board, point: &Point) -> MineConstraint
{
    let cell = board.retrieve_cell(point);
    let unknown_neighbors: Vec<Point> = board.neighbor_cells_from_point(point).iter()
        .filter(|cell| cell.knowledge.is_unknown())
        .map(|cell| cell.point)
        .collect();
    let total_unknown = unknown_neighbors.len();
    let known_mines = board.count_assumed_mined_neighbors(point);
    let expected_mines = cell.mined_neighbor_count as i32 - known_mines as i32;
    let expected_empties = total_unknown as i32 - expected_mines as i32 ;
    MineConstraint{expected_mines, expected_empties, constrained_points:unknown_neighbors}
}

fn build_constraint_solver(board: &Board) -> ConstraintSolver<Point, bool>
{
    let points: Vec<Point> = board.get_border_points(); //Not great i call this multiple times each search

    let constraining_points: HashSet<_> = points.iter()
        .flat_map(|point| board.neighbor_points(point))
        .collect();

    let mut constraints: Vec<Rc<dyn Constraint<Point, bool>>> = constraining_points.iter()
        .map(|point| board.retrieve_cell(&point))
        .filter(|cell| cell.is_known_unmined() && board.has_unknown_neighbors(&cell.point))
        .map(|cell| {
            let constraint = construct_constraint(&board, &cell.point);
            let r: Rc<dyn Constraint<Point, bool>> = Rc::new(constraint);
            r
        })
        .collect();

    constraints.push(Rc::new(construct_global_constraint(&board, &points)));

    let variables = points.into_iter()
        .map(|point| Variable{id: point, value: None, possible: vec![false, true]})
        .collect();
    ConstraintSolver::new(variables, constraints)
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
        println!("probs are");
        println!("{}", board.to_string_with_probabilities(&probabilities));
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
        let rollouts = 20;
        let border_points: Vec<Point> = board.get_border_points();
        for _ in 0..rollouts{
            // FIXME need randomness
            let mut solver = build_constraint_solver(board);
            let assignments = solver.backtrack().expect("failed to find a solution");
            assignments.iter().for_each(|(point, mined)| {
                match mined {
                    false => {},
                    true => {
                        if counts.contains_key(&point){
                            *counts.get_mut(&point).unwrap() += 1;
                        }
                        else{
                            counts.insert(*point, 1);
                        }
                    }
                }
            });
        }
        border_points.into_iter()
            .map(|point| {
                let count = counts.get(&point).unwrap_or(&0);
                (point, (*count as f32)/(rollouts as f32))
            })
            .collect()
    }
}
