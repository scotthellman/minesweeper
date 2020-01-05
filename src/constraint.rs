use std::collections::HashMap;
use std::hash::Hash;

pub struct Variable<S: Hash + Eq, T> 
{
    pub id: S,
    pub value: Option<T>,
    possible: Vec<T>
}

pub trait Constraint<S: Hash + Eq, T> 
{
    fn get_constrained_variable_ids(&self) -> Vec<S>;
    fn check_constraint(&self, variable_lookup: &HashMap<S, Variable<S, T>>) -> bool;
}

pub struct ConstraintSolver<'a, S: Hash + Eq, T> 
{
    pub variable_lookup: HashMap<S, Variable<S, T>>,
    variable_to_constraints: HashMap<S, Vec<&'a Constraint<S, T>>>,
    constraints: Vec<&'a Constraint<S, T>>,
}

impl<'a, S: Hash + Eq, T> ConstraintSolver<'a, S, T> 
{
    pub fn new(variables: Vec<Variable<S, T>>, constraints: Vec<&'a Constraint<S, T>>) -> ConstraintSolver<'a, S, T>{
        //let variable_to_constraints:HashMap<String, Vec<&Constraint<T>>> = constraints.iter()
        let mut variable_to_constraints:HashMap<S, Vec<&Constraint<S, T>>> = HashMap::with_capacity(constraints.len());
        constraints.iter()
            .flat_map(|constraint| constraint.get_constrained_variable_ids().iter()
                                         .map(|v_id| (*v_id, *constraint)))
            .for_each(|(v_id, constraint)| {
                let group = variable_to_constraints.entry(v_id).or_insert(vec![]);
                group.push(constraint)
            });
        let variable_lookup = variables.into_iter()
            .map(|v| (v.id, v))
            .collect();
        ConstraintSolver{variable_lookup, variable_to_constraints, constraints}
    }

    pub fn backtrack(&mut self) -> Option<HashMap<S, T>>{
        let remaining_points: Vec<S> = self.variable_lookup.keys().map(|s| *s).collect();
        self._backtrack(&remaining_points[..])
    }

    fn _backtrack(&mut self, remaining_points: &[S]) -> Option<HashMap<S, T>> {
        // most naive thing to do is just use variables in order
        match remaining_points.first(){
            None => {
                let empty: HashMap<S, T> = HashMap::with_capacity(self.variable_lookup.len());
                Some(empty)
            } ,
            Some(v_id) => {
                let mut variable = self.variable_lookup.get(v_id).expect("variable lookup can't find variable");
                for state in variable.possible {
                    variable.value = Some(state);
                    if self.constraints_are_satisfied(&variable){
                        match self._backtrack(&remaining_points[1..]){
                            None => variable.value = None,
                            Some(mut children) => {
                                children.insert(variable.id, state); //TODO: feels a bit weird
                                return Some(children)
                            }
                        }
                    }
                }
                return None
            }
        }
    }

    fn constraints_are_satisfied(&self, variable: &Variable<S, T>) -> bool{
        match self.variable_to_constraints.get(&variable.id){
            None => true, //no constraints on the variable so go for it
            Some(constraints) => {
                constraints.iter().all(|constraint| constraint.check_constraint(&self.variable_lookup))
            }
        }
    }
}
