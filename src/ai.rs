use std::rc::Rc;
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
        let (mined, empty) = self.count_remaining_mined_and_empty(variable_lookup);
        mined <= self.expected_mines && empty <= self.expected_empties
    }

    fn consistent_states_for_variable(&self, variable_lookup: &HashMap<Point, Variable<Point, bool>>, v_id: &Point) -> Vec<bool>{
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
                        match val {
                            false => (0, 1),
                            true => (1, 0)
                        }
                    }
                }
            })
            .fold((0, 0), |acc, next| (acc.0 + next.0, acc.1 + next.1))
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

        match self.move_queue.pop(){
            Some(action) => action,
            None => {
                let start = time::Instant::now();
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
