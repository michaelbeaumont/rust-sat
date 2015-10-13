use vec_map::VecMap;
use vec_map::Entry::{Occupied, Vacant};
use std::collections::VecDeque;

use super::{Lit, Interp, CNF, Clause, SATSolver};
use super::Satness;
use super::Satness::{SAT, UNSAT};

use self::Safety::{Safe, Conflict};

//Watched clauses
#[derive(Debug)]
struct WatchedClause {
    indices: (usize, usize),
    cls: Clause
}

struct FstOrSnd(bool);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PropRes {
    Conflict,
    True,
    Unit(Lit),
    NewWatch(Lit)
}

impl WatchedClause {
    fn get_watch_info(&self, lit: &Lit) -> FstOrSnd {
        FstOrSnd(self.cls[self.indices.0] != lit.not())
    }

    fn is_watched(&self, test_ind: usize) -> bool {
        test_ind == self.indices.0 || test_ind == self.indices.1
    }

    fn get_other_watched(&self, &FstOrSnd(watch_info): &FstOrSnd) -> &Lit {
        if watch_info {
            &self.cls[self.indices.0]}
        else {
            &self.cls[self.indices.1]}
    }

    fn set_watched(&mut self, &FstOrSnd(watch_info): &FstOrSnd, ind: usize) {
        if watch_info {
            self.indices.1 = ind;}
        else {
            self.indices.0 = ind;}
    }

    fn bcp(&mut self,
               interp: &Interp,
               lit: &Lit)
               -> PropRes
    {
        let watch_info = self.get_watch_info(lit);
        let other_lit = self.get_other_watched(&watch_info).clone();
        let other_val = interp.get_val(&other_lit);
        if other_val.is_some() && other_val.unwrap() {
            return PropRes::True;}
        let mut new_ind = None;
        for i in (0..self.cls.len()).filter( |&i| !self.is_watched(i)) {
            let test_lit = &self.cls[i];
            let curr_val = interp.get_val(test_lit);
            if curr_val.is_none() || curr_val.unwrap() {
                //debug!("Found new watchee: {:?}", test_lit);
                new_ind = Some(i);
                break;}}
        match new_ind {
            None => {
                if other_val.is_some() && !other_val.unwrap() {
                    PropRes::Conflict }
                else {
                    PropRes::Unit(other_lit)}}
            Some(ind) => {
                self.set_watched(&watch_info, ind);
                PropRes::NewWatch(self.cls[ind].clone())}}
    }
}

fn add_watched(watches: &mut VecMap<Vec<usize>>,
               lit: &Lit,
               ind: usize)
{
    let id = lit.not().as_usize();
    match watches.entry(id) {
        Vacant(entry) => {entry.insert(vec![ind]);},
        Occupied(entry) => entry.into_mut().push(ind)}
}

type WatcherList = VecMap<Vec<usize>>;
type WatchedFormula = Vec<WatchedClause>;

pub struct Solver {
    //Interpretation stack
    //the var we set to true, whether this was after a conflict and the interp
    interp_stack: Vec<(Lit, bool, Interp)>,

    //Current interpretation
    interp: Interp,

    //Clauses and constraints
    clss: WatchedFormula,

    //Queue for unit propagations
    prop_queue: VecDeque<Lit>,

    //Watch list
    watches: WatcherList,

}

enum Safety {
    Safe,
    Conflict
}

impl Solver {
    fn find_var(&self) -> Option<Lit> {
        for x in self.clss.iter().flat_map( |x| x.cls.iter()) {
            if self.interp.get_val(x).is_none() {
                return Some(x.clone());}}
        None
    }

    fn check_watchers(&mut self, lit: Lit) -> Option<usize> {
        self.watches.get(&lit.as_usize()).cloned().and_then(
            |clause_inds| {
                let mut new_inds = Vec::new();
                let mut conflict_seen = None;
                for cls_ind in clause_inds {
                    if conflict_seen.is_some() {
                        new_inds.push(cls_ind);
                        continue;}
                    let bcp_res =
                        self.clss[cls_ind].bcp(
                            &self.interp,
                            &lit);
                    if let PropRes::NewWatch(new_lit) = bcp_res {
                        add_watched(
                            &mut self.watches,
                            &new_lit,
                            cls_ind);
                    }
                    else {
                        new_inds.push(cls_ind);
                        if let PropRes::Conflict = bcp_res {
                            conflict_seen = Some(cls_ind);
                        }
                        else if let PropRes::Unit(unit_lit) = bcp_res {
                            self.prop_queue.push_back(
                                (unit_lit));
                        }
                    }
                }
                self.watches.insert(lit.as_usize(), new_inds);
                conflict_seen
        })
    }

    fn backtrack(&mut self) -> Safety {
        //just reverse the most recent non post_conflicted assignment
        //reversing also all the propagated stuff
        //for one level change nothing
        loop {
            info!("Backtrack up a level");
            match self.interp_stack.pop() {
                Some((last, post_confl, a)) => {
                    info!("Unsetting {:?}", last);
                    if !post_confl {
                        self.interp = a;
                        info!("Trying {:?}, set: {:?}  -> true",
                              last.not(),
                              last.id());
                        self.interp_stack.push(
                            (last.clone(),
                             true,
                             self.interp.clone()
                            ));
                        self.prop_queue.clear();
                        self.process(last.not());
                        return Safe;
                    }
                },
                None => {
                    info!("Hit root level, UNSAT");
                    return Conflict;
                }
            }
        }
    }

    fn decide_var(&mut self, lit: Option<Lit>) -> Option<Safety> {
        lit.or_else(|| self.find_var()).map(
            |decision| {
            // here we need to pick a new var
            // because we know nothing more is constrainted
            // pick it and add it to the stack
                info!("Trying {:?}, set: {:?} -> {:?}",
                      decision,
                      decision.id(),
                      decision.eval(true));
                self.interp_stack.push(
                    (decision.clone(),
                     false,
                     self.interp.clone(),
                     ));
                self.process(decision)})
    }

    fn process(&mut self,
               constr_lit: Lit)
               -> Safety
    {
        self.interp.set_true(&constr_lit);
        match self.check_watchers(constr_lit) {
            None => Safe,
            Some(_) => self.backtrack()
        }
    }

    fn process_queue(&mut self) -> Safety {
        while let Some(constr_lit) = self.prop_queue.pop_front() {
            let process = self.process(constr_lit);
            if let Conflict = process { return Conflict }
        }
        Safe
    }
}

impl SATSolver for Solver {
    fn create(formula: CNF, interp: Option<Interp>) -> Solver {
        //info!("{:?}", formula);
        //let mut watches = HashMap::<Lit, Vec<usize>>::new();
        let mut watches = VecMap::new();
        let mut ind: usize = 0;
        let clss = formula.into_iter().map(|cls|{
            for lit in cls.iter().take(2) {
                add_watched(&mut watches, lit, ind);}
            ind = ind + 1;
            if cls.len() > 1 {
                WatchedClause{indices: (0,1), cls: cls} }
            else {
                WatchedClause{indices: (0,0), cls: cls} }})
            .collect();
        //debug!("Watched: {:?}",clss);
        //debug!("Watches: {:?}",watches);
        Solver{
            interp: interp.unwrap_or_else(|| Interp(VecMap::new())),
            interp_stack: Vec::new(),
            clss: clss,
            prop_queue: VecDeque::new(),
            watches: watches
        }
    }

    fn solve(&mut self) -> Satness {
        //handle top level units
        for unit in self.clss.iter().filter_map(
            |c|
            if c.cls.len() == 1 {
                Some(c.cls[0].clone())}
            else {None})
        {
            info!("Found top level unit: {:?}", unit);
            self.prop_queue.push_back(unit)}

        //main loop
        loop {
            let processing = match self.process_queue() {
                Safe =>
                    match self.decide_var(None) {
                        None => return SAT(self.interp.clone()),
                        Some(safety) => safety},
                e => e};

            if let Conflict = processing {
                let reason = format!("Found conflict");
                return UNSAT(reason)
            }
        }
    }
}

