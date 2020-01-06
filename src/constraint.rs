use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;

pub struct Variable<S: Hash + Eq + Copy, T: Copy> 
{
    pub id: S,
    pub value: Option<T>,
    possible: Vec<T>
}

pub trait Constraint<S: Hash + Eq + Copy, T: Copy> 
{
    fn get_constrained_variable_ids(&self) -> Vec<S>;
    fn check_constraint(&self, variable_lookup: &HashMap<S, Variable<S, T>>) -> bool;
}

pub struct ConstraintSolver< S: Hash + Eq + Copy, T: Copy> 
{
    pub variable_lookup: HashMap<S, Variable<S, T>>,
    variable_to_constraints: HashMap<S, Vec<Rc<dyn Constraint<S, T>>>>,
    constraints: Vec<Rc<dyn Constraint<S, T>>>,
}

impl<S: Hash + Eq + Copy, T: Copy> ConstraintSolver<S, T> 
{
    pub fn new(variables: Vec<Variable<S, T>>, constraints: Vec<Rc<dyn Constraint<S, T>>>) -> ConstraintSolver<S, T>{
        //let variable_to_constraints:HashMap<String, Vec<&Constraint<T>>> = constraints.iter()
        let mut variable_to_constraints:HashMap<S, Vec<Rc<dyn Constraint<S, T>>>> = HashMap::with_capacity(constraints.len());
        constraints.iter()
            .flat_map(|constraint| constraint.get_constrained_variable_ids().iter()
                                         .map(|v_id| (*v_id, *constraint)))
            .for_each(|(v_id, constraint)| {
                let mut group = variable_to_constraints.entry(v_id).or_insert(vec![]);
                group.push(Rc::clone(&constraint));
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

    fn set_variable_state(&mut self, v_id: &S, state: Option<T>){
        self.variable_lookup.get_mut(v_id).expect("variable lookup can't find variable").value = state;
    }

    fn _backtrack(&mut self, remaining_points: &[S]) -> Option<HashMap<S, T>> {
        // most naive thing to do is just use variables in order
        match remaining_points.first(){
            None => {
                let empty: HashMap<S, T> = HashMap::with_capacity(self.variable_lookup.len());
                Some(empty)
            } ,
            Some(v_id) => {
                //let variable: &mut Variable<S, T> = self.variable_lookup.get_mut(v_id).expect("variable lookup can't find variable");
                let states = self.variable_lookup.get(v_id).unwrap().possible.to_vec();
                for state in states {
                    self.set_variable_state(v_id, Some(state));
                    //OK so I need to push all of these into methods on self
                    if self.constraints_are_satisfied(v_id){
                        match self._backtrack(&remaining_points[1..]){
                            None => self.set_variable_state(v_id, None),
                            Some(mut children) => {
                                children.insert(*v_id, state);
                                return Some(children)
                            }
                        }
                    }
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
}
