use std::sync::{Mutex, Arc};
use super::board::Board;
use super::board::Point;
use super::ActionType;
use super::Agent;
use super::constraint::Variable;
use super::constraint::Constraint;
use super::constraint::ConstraintSolver;
use super::constraint::RandomSelectionStrategy;
use std::thread;
use std::time;
use std::collections::HashSet;
use std::collections::HashMap;
use rand::seq::SliceRandom;

struct MineConstraint {
    expected_mines: i32,
    expected_empties: i32,
    constrained_points: Vec<Point>,
    global: bool //FIXME: this is lazy of me
}

impl Constraint<Point, bool> for MineConstraint {
    fn get_constrained_variable_ids(&self) -> Vec<Point>{
        self.constrained_points.iter().copied().collect()
    }

    fn check_constraint(&self, global_counts: &HashMap<bool, usize>,
                        variable_lookup: &HashMap<Point, Variable<Point,bool>>)-> bool {
        let (mined, empty) = if self.global {
            (*global_counts.get(&true).unwrap() as i32, *global_counts.get(&false).unwrap() as i32)
        } else {
            self.count_remaining_mined_and_empty(&variable_lookup)
        };
        mined <= self.expected_mines && empty <= self.expected_empties
    }

    fn consistent_states_for_variable(&self, variable_lookup: &HashMap<Point, Variable<Point, bool>>, _: &Point) -> Vec<bool>{
        let (mined, empty) = self.count_remaining_mined_and_empty(variable_lookup);
        let mut possible = Vec::with_capacity(2);
        if mined > 0 {
            possible.push(true);
        };
        if empty > 0 {
            possible.push(false);
        }
        possible
    }
}

impl MineConstraint {

    fn count_remaining_mined_and_empty(&self, variable_lookup: &HashMap<Point, Variable<Point, bool>>) -> (i32, i32) {
        self.constrained_points.iter()
            .map(|v_id| variable_lookup.get(v_id).expect("variable not in lookup"))
            .map(|variable| {
                match variable.value {
                    None => (0, 0),
                    Some(val) => {
                        if val {
                            (1, 0)
                        } else {
                            (0, 1)
                        }
                    }
                }
            })
            .fold((0, 0), |acc, next| (acc.0 + next.0, acc.1 + next.1))
    }
}

fn construct_global_constraint(board: &Board) -> MineConstraint{
    let expected_mines = board.remaining_mines(); //this is really just a number we can't exceed
    let expected_empties = board.unknown_count() as i32 - board.remaining_mines();
    MineConstraint{expected_mines, expected_empties, constrained_points: vec![], global: true}
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
    MineConstraint{expected_mines, expected_empties, constrained_points:unknown_neighbors, global: false}
}

fn build_constraint_solver(board: &Board) -> ConstraintSolver<Point, bool, RandomSelectionStrategy>
{
    let points: Vec<Point> = board.get_border_points(); //Not great i call this multiple times each search

    let constraining_points: HashSet<_> = points.iter()
        .flat_map(|point| board.neighbor_points(point))
        .collect();

    let mut constraints: Vec<Arc<dyn Constraint<Point, bool>  + Send + Sync>> = constraining_points.iter()
        .map(|point| board.retrieve_cell(&point))
        .filter(|cell| cell.is_known_unmined() && board.has_unknown_neighbors(&cell.point))
        .map(|cell| {
            let constraint = construct_constraint(&board, &cell.point);
            let r: Arc<dyn Constraint<Point, bool>  + Send + Sync> = Arc::new(constraint);
            r
        })
        .collect();

    constraints.push(Arc::new(construct_global_constraint(&board)));

    let variables = points.into_iter()
        .map(|point| Variable{id: point, value: None, possible: vec![false, true]})
        .collect();
    ConstraintSolver::new(variables, constraints, RandomSelectionStrategy{})
}


pub struct NaiveAI {
    move_queue: Vec<ActionType>,
    min_move_time: time::Duration,
    max_move_time: time::Duration
}

impl Agent for NaiveAI {
    fn generate_move(&mut self, board: &Board) -> ActionType {
        let start = time::Instant::now();
        let result = match self.move_queue.pop(){
            Some(action) => action,
            None => {
                self.move_queue = self.generate_next_moves(board);
                self.move_queue.pop().expect("something weird happened and we have no moves")
            }
        };
        let now = time::Instant::now();
        let elapsed = now - start;
        println!("generated move in {:?}", elapsed);
        if elapsed < self.min_move_time{
            thread::sleep(self.min_move_time - elapsed);
        }
        result
    }
}

impl NaiveAI {

    pub fn new(min_move_time: u64, max_move_time: u64) -> NaiveAI{
        let mut move_queue = Vec::with_capacity(4);
        let min_move_time = time::Duration::from_millis(min_move_time);
        let max_move_time = time::Duration::from_millis(max_move_time);
        move_queue.push(ActionType::Click(Point(0, 0)));
        NaiveAI{move_queue, min_move_time, max_move_time}
    }

    pub fn generate_next_moves(&self, board: &Board) -> Vec<ActionType>{
        let safe_flags = NaiveAI::known_safe_flags(board);
        if !safe_flags.is_empty() {
            return safe_flags.iter().map(|point| ActionType::Flag(*point)).collect()
        }

        let safe_clicks = NaiveAI::known_safe_clicks(board);
        if !safe_clicks.is_empty() {
            return safe_clicks.iter().map(|point| ActionType::Click(*point)).collect()
        }

        let probabilities = self.get_monte_carlo_probabilities(board);
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
            if proba == 0.0 {
                actions.push(ActionType::Click(point));
            }
            if proba as usize == 1 {
                actions.push(ActionType::Flag(point));
            }
        }
        let best_point = best_point.unwrap_or_else(|| self.get_random_valid_click(board));
        if actions.is_empty() {
            actions.push(ActionType::Flag(best_point))
        }
        actions
    }

    fn get_random_valid_click(&self, board: &Board) -> Point {
        let points = board.get_unknown_points();
        *points.choose(&mut rand::thread_rng()).expect("There are no unknown points!")
    }

    fn known_safe_flags(board: &Board) -> HashSet<Point> {
        board.size.points().iter()
             .filter(|point| board.retrieve_cell(point).knowledge.is_known())
            .flat_map(|point| board.known_flaggable_neighbors(point))
            .collect()
    }

    fn known_safe_clicks(board: &Board) -> HashSet<Point> {
        board.size.points().iter()
             .filter(|point| board.retrieve_cell(point).knowledge.is_known())
            .flat_map(|point| board.known_safe_neighbors(point))
            .collect()
    }


    fn get_monte_carlo_probabilities(&self, board: &Board) -> Vec<(Point, f32)>{
        let start = time::Instant::now();

        // TODO: ok so this isn't really naive anymore is it
        let counts: Arc<Mutex<HashMap<Point, usize>>> = Arc::new(Mutex::new(HashMap::new()));
        let rollouts = Arc::new(Mutex::new(0));
        let threads = 4;
        let mut handles = vec![];
        let border_points: Vec<Point> = board.get_border_points();
        let max_move_time = self.max_move_time;
        for _ in 0..threads {
            let counts = Arc::clone(&counts);
            let rollouts = Arc::clone(&rollouts);
            let mut solver = build_constraint_solver(board);
            let handle = thread::spawn(move || {
                while time::Instant::now().duration_since(start) < max_move_time {
                    let assignments = solver.backtrack().expect("failed to find a solution");
                    let mut counts = counts.lock().unwrap();
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
                    let mut rollouts = rollouts.lock().unwrap();
                    *rollouts += 1;
                }
            });
            handles.push(handle);
        }
        for handle in handles {
            handle.join().unwrap();
        }
        let counts = counts.lock().unwrap();
        let rollouts = *rollouts.lock().unwrap();
        println!("We got {} rollouts", rollouts);
        thread::sleep(time::Duration::from_millis(1000));
        border_points.into_iter()
            .map(|point| {
                let count = counts.get(&point).unwrap_or(&0);
                (point, (*count as f32)/(rollouts as f32))
            })
            .collect()
    }
}
