use std::collections::{HashMap, RingBuf};
use super::{Lit, Negable, CNF, Clause, Interp, SATSolver, Satness};
use super::Satness::{UNSAT, SAT};

pub struct Solver {
    //Interpretation stack
    //the var we set to true, whether this was after a conflict and the interp
    interp_stack: Vec<(Lit, bool, Interp)>,

    //Current interpretation
    curr_interp: Interp,

    //Clauses and constraints
    clss: CNF,

    //Queue for unit propagations
    prop_queue: RingBuf<Lit>,

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
        match assigned.get_val(l) {
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

impl Solver {
    fn assign_true(&mut self, l: &Lit) {
        self.curr_interp.assign_true(l);
    }

    fn find_var(&self) -> Option<Lit> {
        for x in self.clss.iter().flat_map(|x| x.iter()) {
            if self.curr_interp.get_val(x).is_none() {
                return Some(x.clone());
            }
        }
        None
    }

    fn have_conflict(&self, lit: &Lit) -> bool {
        let m_cur_assn = self.curr_interp.get_val(lit);
        let new_assn = lit.get_truth_as(true);
        m_cur_assn.is_some() &&  *m_cur_assn.unwrap() == !new_assn
    }

    fn propagate(&mut self) {
        for c in self.clss.iter() {
            let poss_unit = get_unit(c, &self.curr_interp);
            match poss_unit {
                Some(u) => {
                    let mut found = false;
                    for p in self.prop_queue.iter() {
                        if *p == u { found = true; }
                    }
                    if !found {
                        info!("Found implied unit: {} in {}", u, c);
                        self.prop_queue.push_back(u)
                    }
                }
                _    => {}
            }
        }

    }

    fn backtrack(&mut self) -> bool {
        //just reverse the most recent non post_conflicted assignment
        //reversing also all the propagated stuff
        //for one level change nothing
        loop {
            info!("Backtrack up a level");
            match self.interp_stack.pop() {
                Some((last, was_post_confl, a)) => {
                    info!("Unsetting {}", last);
                    if !was_post_confl {
                        self.curr_interp = a;
                        info!("Trying {}, set: {}  -> true", last.not(), last.id());
                        self.interp_stack.push(
                            (last.clone(),
                             true,
                             self.curr_interp.clone()
                            ));
                        let last_not = last.not();
                        self.assign_true(&last_not);
                        self.prop_queue.clear();
                        self.propagate();
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

impl SATSolver for Solver {
    fn create(formula: CNF) -> Solver {
        Solver{
            curr_interp: Interp(HashMap::new()),
            interp_stack:Vec::new(),
            clss: formula, 
            prop_queue: RingBuf::new()}
    }
    
    fn solve(&mut self) -> Satness {
        let units: Vec<Lit> = self.clss.iter().filter_map(
            |c| if c.len() == 1 {Some(c[0].clone())} else {None}
            ).collect();
        for unit in units.iter() {
            info!("Found top level unit: {}", unit);
            self.prop_queue.push_back(unit.clone());
        }

        loop {
            match self.prop_queue.pop_front() {
                Some(constr_lit) => {
                    if self.have_conflict(&constr_lit) {
                        //just reverse the most recent non post_conflicted assignment
                        //reversing also all the propagated stuff
                        info!("Conflict at {}", constr_lit);
                        let res = self.backtrack();
                        if !res {
                            let reason = format!("Found conflict with {}",
                                                 constr_lit.id());
                            return UNSAT(reason)
                        }
                    }
                    else {
                        info!("Processing from queue {}, set: {} -> {}",
                              constr_lit,
                              constr_lit.id(),
                              constr_lit.get_truth_as(true));
                        self.assign_true(&constr_lit);
                        self.propagate();
                    }
                    //info!("Propagate constraints");
                }
                None  =>
                    // here we need to pick a new var
                    // because we know nothing more is constrainted
                    // pick it and add it to the stack
                    match self.find_var() {
                        Some(actual_var) => {
                            info!("Trying {}, set: {} -> {}",
                                  actual_var,
                                  actual_var.id(),
                                  actual_var.get_truth_as(true));
                            self.interp_stack.push(
                                (actual_var.clone(),
                                 false,
                                 self.curr_interp.clone()
                                 ));
                            self.assign_true(&actual_var);
                            //info!("Propagate constraints");
                            self.propagate();
                        },
                        None          => return SAT(self.curr_interp.clone())
                    }
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::super::Lit::P;
    use super::super::Lit::N;
    use super::super::Interp;

    use std::collections::HashMap;

    #[test]
    fn test_get_unit() {
        let clause1 = vec![P("X".to_string()), N("Y".to_string())];
        let mut assigned = Interp(HashMap::new());
        assigned.assign_true(&N("X".to_string()));
        assert_eq!(Some(N("Y".to_string())), super::get_unit(&clause1, &assigned));
        assigned.assign_true(&N("Y".to_string()));
        assert_eq!(None, super::get_unit(&clause1, &assigned));
        let clause2 = vec![P("X".to_string()), 
                            N("Y".to_string()),
                            P("Z".to_string())];
        assert_eq!(None, super::get_unit(&clause2, &assigned));
    }
}
