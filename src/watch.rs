use std::collections::{HashMap, RingBuf};
use std::collections::hash_map::Entry::{Occupied, Vacant};
use super::{Lit, Negable, Interp, CNF, Clause, SATSolver};
//use super::Satness;
//use super::Satness::{UNSaT, SAT};
use super::Satness;
use super::Satness::{SAT, UNSAT};

#[deriving(Show)]
pub struct WatchedClause {
    indices: [uint, ..2],
    pub cls: Clause
}

impl WatchedClause {
    pub fn get_watch_info(&self, lit: &Lit) -> bool {
        self.cls[self.indices[0]] != lit.not()
    }

    pub fn get_other(&self, w: bool) -> &Lit {
        if w {&self.cls[self.indices[0]]} else {&self.cls[self.indices[1]]}
    }

    pub fn get_this_other(&self, lit: &Lit) -> (bool, uint, uint) {
        if self.cls[self.indices[0]] == lit.not() {
            (false, self.indices[0], self.indices[1])
        } else {
            (true, self.indices[1], self.indices[0]) }
    }

    pub fn set_watched(&mut self, watch_info: &bool, ind: uint) {
        if *watch_info {
            self.indices[1] = ind;   
        } else {
            self.indices[0] = ind;
        }
    }

    pub fn is_watched_index(&self, test_ind: uint) -> bool {
        test_ind == self.indices[0] || test_ind == self.indices[1]
    }
}

pub struct Solver {
    //Interpretation stack
    //the var we set to true, whether this was after a conflict and the interp
    interp_stack: Vec<(Lit, bool, Interp)>,

    //Current interpretation
    curr_interp: Interp,

    //Clauses and constraints
    clss: Vec<WatchedClause>,

    //Queue for unit propagations
    prop_queue: RingBuf<Lit>,

    //Watch list
    watches: HashMap<Lit, Vec<uint>>

}


fn add_watched(watches: &mut HashMap<Lit, Vec<uint>>, lit_: &Lit, ind: uint) {
    let lit = lit_.not();
    match watches.entry(lit) {
        Vacant(entry) => {entry.set(vec![ind]);},
        Occupied(entry) => entry.into_mut().push(ind)
    }
}

impl Solver {
    fn find_var(&self) -> Option<Lit> {
        for x in self.clss.iter().flat_map(|x| x.cls.iter()) {
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

    fn propagate_clause(&mut self, lit: &Lit, cls_ind: &uint) -> bool {
        let cls = &mut self.clss[*cls_ind];
        let watch_info = cls.get_watch_info(lit);
        let other_lit = cls.get_other(watch_info).clone();
        let other_val = self.curr_interp.get_val(&other_lit);
        if other_val.is_some() && other_lit.get_truth_as(*other_val.unwrap()) {
            return true;
        }
        let mut new_ind = None;
        for i in range(0, cls.cls.len()).filter(|&i| !cls.is_watched_index(i)) {
            let test_lit = &cls.cls[i];
            let curr_val = self.curr_interp.get_val(test_lit);
            if curr_val.is_none() || test_lit.get_truth_as(*curr_val.unwrap()) {
                info!("Found new watcher: {}", test_lit);
                new_ind = Some(i);
                break;
            }
        }
        match new_ind {
            None      => {
                info!("Found unit: {}", other_lit);
                self.prop_queue.push_back(other_lit);
                true}
            Some(ind) => {
                cls.set_watched(&watch_info, ind);
                add_watched(&mut self.watches, &cls.cls[ind], *cls_ind);
                false
            }
        }
    }

    fn propagate(&mut self, lit: Lit) {
        let new_inds: Vec<uint> = match self.watches.get(&lit).cloned() {
            Some(clause_inds) => {
                clause_inds.into_iter().filter(|&cls_ind| {
                    debug!("Clause being watched: {}", cls_ind);
                    let ans = self.propagate_clause(&lit, &cls_ind); 
                    debug!("New watched: {}", self.watches);
                    debug!("New clauses: {}", self.clss);
                    ans
                }).collect()
            },
            None => return
        };
        self.watches.insert(lit, new_inds);
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
                        info!("Trying {}, set: {}  -> true",
                              last.not(),
                              last.id());
                        self.interp_stack.push(
                            (last.clone(),
                             true,
                             self.curr_interp.clone()
                            ));
                        let last_not = last.not();
                        self.curr_interp.assign_true(&last_not);
                        self.prop_queue.clear();
                        self.propagate(last_not);
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
        let mut watches = HashMap::<Lit, Vec<uint>>::new();
        let mut ind: uint = 0;
        let clss = formula.into_iter().map(|cls_|{
            for lit in cls_.iter().take(2) {
                add_watched(&mut watches, lit, ind);
            }
            ind = ind + 1;
            if cls_.len() > 1 {
                WatchedClause{indices: [0u,1u], cls: cls_} }
            else {
                WatchedClause{indices: [0u,0u], cls: cls_} }
        }).collect();
        info!("Watched: {}",clss);
        info!("Watches: {}",watches);
        Solver{
            curr_interp: Interp(HashMap::new()),
            interp_stack:Vec::new(),
            clss: clss, 
            prop_queue: RingBuf::new(),
            watches: watches}
    }

    fn solve(&mut self) -> Satness {
        let units: Vec<Lit> = self.clss.iter().filter_map(
            |c| if c.cls.len() == 1 {Some(c.cls[0].clone())} else {None}
            ).collect();

        for unit in units.into_iter() {
            info!("Found top level unit: {}", unit);
            self.prop_queue.push_back(unit);
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
                        self.curr_interp.assign_true(&constr_lit);
                        self.propagate(constr_lit);
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
                            self.curr_interp.assign_true(&actual_var);
                            //info!("Propagate constraints");
                            self.propagate(actual_var);
                        },
                        None          => return SAT(self.curr_interp.clone())
                    }
            }
        }
    }
}

