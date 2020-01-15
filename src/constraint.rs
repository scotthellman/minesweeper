use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::Hash;
use std::rc::Rc;
use std::fmt::Debug;
use std::sync::Arc;

#[derive(Debug)]
pub struct Variable<S: Hash + Eq + Copy + Debug, T: Copy + Debug + Hash + Eq> 
{
    pub id: S,
    pub value: Option<T>,
    pub possible: Vec<T>
}

// TODO: this feels very java
pub trait SelectionStrategy<S: Hash + Eq + Copy + Debug, T: Copy + Debug + Hash + Eq> {
    fn get_next_index(&self, variable_lookup: &HashMap<S, Variable<S, T>>,
                      variable_to_constraints: &HashMap<S, Vec<Arc<dyn Constraint<S, T>  + Send + Sync>>>,
                      points: &[S], available_indices: &HashSet<usize>) -> Option<usize>;
}

pub struct RandomSelectionStrategy { }

impl<S, T> SelectionStrategy<S, T> for RandomSelectionStrategy where
    S: Copy + Debug + Hash + Eq,
    T: Copy + Debug + Hash + Eq
{
    fn get_next_index(&self, _: &HashMap<S, Variable<S, T>>,
                      _: &HashMap<S, Vec<Arc<dyn Constraint<S, T>  + Send + Sync>>>,
                      _: &[S], available_indices: &HashSet<usize>) -> Option<usize> {
        match available_indices.iter().next() {
            None => None,
            Some(&val) => Some(val)
        }
    }
}

pub struct DegreeSelectionStrategy { }

impl<S, T> SelectionStrategy<S, T> for DegreeSelectionStrategy where
    S: Copy + Debug + Hash + Eq,
    T: Copy + Debug + Hash + Eq
{
    fn get_next_index(&self, _: &HashMap<S, Variable<S, T>>,
                      variable_to_constraints: &HashMap<S, Vec<Arc<dyn Constraint<S, T>  + Send + Sync>>>,
                      points: &[S], available_indices: &HashSet<usize>) -> Option<usize> {
        let result = available_indices.iter()
            .map(|idx| {
                let count = match  variable_to_constraints.get(&points[*idx]) {
                    None => 0,
                    Some(constraints) => constraints.len()
                };
                (count, idx)
            })
            .max();
        match result {
            None => None,
            Some((_, idx)) => Some(*idx)
        }
    }
}


pub trait Constraint<S: Hash + Eq + Copy + Debug, T: Copy + Debug + Hash + Eq> 
{
    fn get_constrained_variable_ids(&self) -> Vec<S>;
    fn check_constraint(&self, global_counts: &HashMap<T, usize>,
                        variable_lookup: &HashMap<S, Variable<S,T>>)-> bool;
    fn consistent_states_for_variable(&self, variable_lookup: &HashMap<S, Variable<S, T>>, v_id: &S) -> Vec<T>;
}

pub struct ConstraintSolver< S: Hash + Eq + Copy + Debug, T: Copy + Debug + Hash + Eq, Strat: SelectionStrategy<S, T>> 
{
    pub variable_lookup: HashMap<S, Variable<S, T>>,
    variable_to_constraints: HashMap<S, Vec<Arc<dyn Constraint<S, T>  + Send + Sync>>>,
    pub global_counts: HashMap<T, usize>,
    selection_strategy: Strat
}

impl<S: Hash + Eq + Copy + Debug, T: Copy + Debug + Hash + Eq, Strat: SelectionStrategy<S, T>> ConstraintSolver<S, T, Strat> 
{
    pub fn new(variables: Vec<Variable<S, T>>,
               constraints: Vec<Arc<dyn Constraint<S, T>  + Send + Sync>>,
               selection_strategy: Strat) -> ConstraintSolver<S, T, Strat>{
        let mut variable_to_constraints:HashMap<S, Vec<Arc<dyn Constraint<S, T>  + Send + Sync>>> = HashMap::with_capacity(constraints.len());
        constraints.iter().for_each(|constraint| {
            constraint.get_constrained_variable_ids().iter().for_each( |v_id| {
                let group = variable_to_constraints.entry(*v_id).or_insert_with(|| vec![]);
                group.push(Arc::clone(constraint)) // i am baffled that group doesn't have to be mut?
            });
        });
        let global_counts = HashMap::with_capacity(2);

        let variable_lookup = variables.into_iter()
            .map(|v| (v.id, v))
            .collect();

        ConstraintSolver{variable_lookup, variable_to_constraints, global_counts, selection_strategy}
    }

    pub fn backtrack(&mut self) -> Option<HashMap<S, T>>{
        let remaining_points: Vec<S> = self.variable_lookup.keys().copied().collect();
        let mut indices: HashSet<usize> = (0..remaining_points.len()).collect();
        self._backtrack(&remaining_points, &mut indices)
    }

    fn set_variable_state(&mut self, v_id: &S, state: Option<T>){
        let var = self.variable_lookup.get_mut(v_id).expect("variable lookup can't find variable");
        if let Some(state) = var.value {
            let count = self.global_counts.entry(state).or_insert(1);
            *count -= 1;
        }
        if let Some(state) = state {
            let count = self.global_counts.entry(state).or_insert(0);
            *count += 1;
        }
        self.variable_lookup.get_mut(v_id).expect("variable lookup can't find variable").value = state;
    }

    fn _backtrack(&mut self, points: &[S], available_indices: &mut HashSet<usize>) -> Option<HashMap<S, T>> {
        match self.selection_strategy.get_next_index(&self.variable_lookup, &self.variable_to_constraints, points, available_indices) {
            None => {
                let empty: HashMap<S, T> = HashMap::with_capacity(self.variable_lookup.len());
                Some(empty)
            } ,
            Some(index) => {
                available_indices.remove(&index);
                let v_id = points[index];
                let states = self.variable_lookup.get(&v_id).unwrap().possible.to_vec();
                for state in states {
                    self.set_variable_state(&v_id, Some(state));
                    if self.constraints_are_satisfied(&v_id) && self.forward_check(&v_id){
                        if let Some(mut children) = self._backtrack(points, available_indices){
                            children.insert(v_id, state);
                            return Some(children)
                        }
                    }
                    self.set_variable_state(&v_id, None)
                }
                available_indices.insert(index);
                None
            }
        }
    }

    fn constraints_are_satisfied(&self, v_id: &S) -> bool{
        let variable = self.variable_lookup.get(v_id).unwrap();
        match self.variable_to_constraints.get(&variable.id){
            None => true, //no constraints on the variable so go for it
            Some(constraints) => {
                constraints.iter().all(|constraint| constraint.check_constraint(&self.global_counts, &self.variable_lookup))
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
                            !constraint.consistent_states_for_variable(&self.variable_lookup, v_id).is_empty()
                        })
                    })
            }
        }
    }
}
