use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::Hash;
use std::rc::Rc;
use std::fmt::Debug;

#[derive(Debug)]
pub struct Variable<S: Hash + Eq + Copy + Debug, T: Copy + Debug> 
{
    pub id: S,
    pub value: Option<T>,
    pub possible: Vec<T>
}

pub trait Constraint<S: Hash + Eq + Copy + Debug, T: Copy + Debug> 
{
    fn get_constrained_variable_ids(&self) -> Vec<S>;
    fn check_constraint(&self, variable_lookup: &HashMap<S, Variable<S, T>>) -> bool;
    fn consistent_states_for_variable(&self, variable_lookup: &HashMap<S, Variable<S, T>>, v_id: &S) -> Vec<T>;
}

pub struct ConstraintSolver< S: Hash + Eq + Copy + Debug, T: Copy + Debug> 
{
    pub variable_lookup: HashMap<S, Variable<S, T>>,
    variable_to_constraints: HashMap<S, Vec<Rc<dyn Constraint<S, T>>>>,
}

impl<S: Hash + Eq + Copy + Debug, T: Copy + Debug> ConstraintSolver<S, T> 
{
    pub fn new(variables: Vec<Variable<S, T>>, constraints: Vec<Rc<dyn Constraint<S, T>>>) -> ConstraintSolver<S, T>{
        let mut variable_to_constraints:HashMap<S, Vec<Rc<dyn Constraint<S, T>>>> = HashMap::with_capacity(constraints.len());
        constraints.iter().for_each(|constraint| {
            constraint.get_constrained_variable_ids().iter().for_each( |v_id| {
                let group = variable_to_constraints.entry(*v_id).or_insert(vec![]);
                group.push(Rc::clone(constraint)) // i am baffled that group doesn't have to be mut?
            });
        });
        let variable_lookup = variables.into_iter()
            .map(|v| (v.id, v))
            .collect();
        ConstraintSolver{variable_lookup, variable_to_constraints}
    }

    pub fn backtrack(&mut self) -> Option<HashMap<S, T>>{
        let remaining_points: Vec<S> = self.variable_lookup.keys().map(|s| *s).collect();
        self._backtrack(&remaining_points[..])
    }

    fn set_variable_state(&mut self, v_id: &S, state: Option<T>){
        self.variable_lookup.get_mut(v_id).expect("variable lookup can't find variable").value = state;
    }

    fn _backtrack(&mut self, remaining_points: &[S]) -> Option<HashMap<S, T>> {
        // most naive thing to do is just use variables in order
        // TODO: A way to specify a strategy to use for point selection
        match remaining_points.first(){
            None => {
                let empty: HashMap<S, T> = HashMap::with_capacity(self.variable_lookup.len());
                Some(empty)
            } ,
            Some(v_id) => {
                let states = self.variable_lookup.get(v_id).unwrap().possible.to_vec();
                for state in states {
                    self.set_variable_state(v_id, Some(state));
                    if self.constraints_are_satisfied(v_id) && self.forward_check(v_id){
                        if let Some(mut children) = self._backtrack(&remaining_points[1..]){
                            children.insert(*v_id, state);
                            return Some(children)
                        }
                    }
                    self.set_variable_state(v_id, None)
                }
                return None
            }
        }
    }

    fn constraints_are_satisfied(&self, v_id: &S) -> bool{
        let variable = self.variable_lookup.get(v_id).unwrap();
        match self.variable_to_constraints.get(&variable.id){
            None => true, //no constraints on the variable so go for it
            Some(constraints) => {
                constraints.iter().all(|constraint| constraint.check_constraint(&self.variable_lookup))
            }
        }
    }

    fn forward_check(&self, v_id: &S) -> bool {
        let variable = self.variable_lookup.get(v_id).unwrap();
        match self.variable_to_constraints.get(&variable.id){
            None => true, //no constraints on the variable so go for it
            Some(constraints) => {
                constraints.iter()
                    .all(|constraint| {
                        constraint.get_constrained_variable_ids().iter().all(|v_id| {
                            constraint.consistent_states_for_variable(&self.variable_lookup, v_id).len() > 0
                        })
                    })
            }
        }
    }
}
