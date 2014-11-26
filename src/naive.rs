use std::collections::{HashMap, RingBuf};
use super::{Lit, Negable, CNF, Clause, Interp};
use super::Satness;
use super::Satness::{UNSAT, SAT};

pub struct Solver {
    //Interpretation stack
    //the var we set to true, whether there was a post_conflict and the interp
    interp_stack: Vec<(Lit, bool, Interp)>,

    //Current interpretation
    curr_interp: Interp,
    //Are we post conflict
    post_confl: bool,

    //Clauses and constraints
    clss: CNF,

    //Queue for unit propagations
    prop_queue: RingBuf<Lit>,

}

impl Solver {
    pub fn create_solver(formula: CNF) -> Solver {
        Solver{
            curr_interp:HashMap::new(),
            interp_stack:Vec::new(),
            clss: formula, 
            post_confl: false,
            prop_queue: RingBuf::new()}
    }

    fn assign_true(&mut self, l: &Lit) {
        self.curr_interp.insert(l.id().to_string(), l.get_truth_as(true));
    }

    fn find_var(&self) -> Option<Lit> {
        for x in self.clss.iter().flat_map(|x| x.iter()) {
            if self.curr_interp.get(&x.id().to_string()).is_none() {
                return Some(x.clone());
            }
        }
        None
    }

    fn have_conflict(&self, lit: &Lit) -> bool {
        let m_cur_assn = self.curr_interp.get(&lit.id().to_string());
        let new_assn = lit.get_truth_as(true);
        m_cur_assn.is_some() &&  *m_cur_assn.unwrap() == !new_assn
    }

    fn propagate(&mut self) {
        for c in self.clss.iter() {
            let poss_unit = get_unit(c, &self.curr_interp);
            match poss_unit {
                Some(u) => {
                    info!("Found implied unit: {}", u);
                    self.prop_queue.push_back(u)
                }
                _    => {}
            }
        }

    }

    fn backtrack(&mut self, constr_lit: &Lit) -> bool {
        //just reverse the most recent non post_conflicted assignment
        //reversing also all the propagated stuff
        info!("Conflict at {}", constr_lit);
        //for one level change nothing
        loop {
            info!("Backtrack up a level");
            match self.interp_stack.pop() {
                Some((last, was_post_confl, a)) => {
                    info!("Unsetting {}", last);
                    if !was_post_confl {
                        self.curr_interp = a.clone();
                        self.assign_true(&last.not());
                        info!("Trying {}, set: {}  -> true", last.not(), last.id());
                        self.post_confl = true;
                        self.prop_queue.clear();
                        return true;
                    }
                },
                None => {
                    info!("Hit root level, UNSAT");
                    return false;
                }
            }
        }
    }
}

fn get_unit(c: &Clause, assigned: &Interp) -> Option<Lit> {
    //This functions should return Some(x)
    //iff
    //  x is the only uninterpreted var
    //  OR there are no uninterpreted vars
    //      AND x is the first var with interp(x) = 0
    //otherwise return None
    let (mut maybe_unit, mut have_unassned):(Option<Lit>, bool) = (None, false);
    debug!("Check for units in: {}", c);
    for l in c.iter() {
        match assigned.get(&l.id().to_string()) {
            Some(b) => {
                debug!("Var {} assigned {}", l, l.get_truth_as(*b));
                if l.get_truth_as(*b) {
                    debug!("Satisfies clause");
                    return None
                }
                else if maybe_unit.is_none() {
                    debug!("Store as last resort for satisfiability");
                    maybe_unit = Some(l.clone());
                }
            },
            None    => {
                if !have_unassned {
                    debug!("Found first unassigned variable");
                    have_unassned = true;
                    maybe_unit = Some(l.clone());
                }
                else {
                    debug!("Found second unassigned variable, no unit");
                    return None
                }
            }
        }
    }
    maybe_unit
}

pub fn solve_naive(s: &mut Solver) -> Satness {
    let units: Vec<Lit> = s.clss.iter().filter_map(
            |c| get_unit(c, &s.curr_interp)
        ).collect();

    for unit in units.iter() {
        info!("Found top level unit: {}", unit);
        s.prop_queue.push_back(unit.clone());
    }

    loop {
        match s.prop_queue.pop_front() {
            Some(constr_lit) => {
                if s.have_conflict(&constr_lit) {
                    //just reverse the most recent non post_conflicted assignment
                    //reversing also all the propagated stuff
                    let res = s.backtrack(&constr_lit);
                    if !res {
                        let reason = format!("Found conflict with {}",
                                             constr_lit.id());
                        return UNSAT(reason)
                    }
                }
                else {
                    info!("Processing: {}", constr_lit)
                        s.assign_true(&constr_lit);
                }
                info!("Propagate constraints");
                s.propagate();
            }
            None  =>
                // here we need to pick a new var
                // because we know nothing more is constrainted
                // pick it and add it to the stack
                match s.find_var() {
                    Some(actual_var) => {
                        info!("Trying {}, set: {} -> {}",
                              actual_var,
                              actual_var.id(),
                              actual_var.get_truth_as(true));
                        s.interp_stack.push(
                            (actual_var.clone(),
                             s.post_confl,
                             s.curr_interp.clone()
                            ));
                        s.assign_true(&actual_var);
                        s.post_confl = false;
                        info!("Propagate constraints");
                        s.propagate();
                    },
                    None          => return SAT(s.curr_interp.clone())
                }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::super::Lit::P;
    use super::super::Lit::N;

    use std::collections::HashMap;

    #[test]
    fn test_get_unit() {
        let clause1 = vec![P("X".to_string()), N("Y".to_string())];
        let mut assigned = HashMap::new();
        assigned.insert("X".to_string(), false);
        assert_eq!(Some(N("Y".to_string())), super::get_unit(&clause1, &assigned));
        assigned.insert("Y".to_string(), false);
        assert_eq!(None, super::get_unit(&clause1, &assigned));
        let clause2 = vec![P("X".to_string()), 
                            N("Y".to_string()),
                            P("Z".to_string())];
        assert_eq!(None, super::get_unit(&clause2, &assigned));
    }
}
