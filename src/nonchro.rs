use bit_set::BitSet;
use std::cmp::max;
use std::collections::VecDeque;
use vec_map::Entry::{Occupied, Vacant};
use vec_map::VecMap;

use super::Satness;
use super::Satness::{SAT, UNSAT};
use super::{Clause, Id, Interp, Lit, Map, SATSolver, CNF};

use self::Safety::{Conflict, Safe};

//Watched clauses
#[derive(Debug)]
struct WatchedClause {
    indices: (usize, usize),
    cls: Clause,
}

struct FstOrSnd(bool);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PropRes {
    Conflict,
    True,
    Unit(Lit),
    NewWatch(Lit),
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
            &self.cls[self.indices.0]
        } else {
            &self.cls[self.indices.1]
        }
    }

    fn set_watched(&mut self, &FstOrSnd(watch_info): &FstOrSnd, ind: usize) {
        if watch_info {
            self.indices.1 = ind;
        } else {
            self.indices.0 = ind;
        }
    }

    fn bcp(&mut self, interp: &Interp, lit: &Lit) -> PropRes {
        let watch_info = self.get_watch_info(lit);
        let other_lit = self.get_other_watched(&watch_info).clone();
        let other_val = interp.get_val(&other_lit);
        if other_val.is_some() && other_val.unwrap() {
            return PropRes::True;
        }
        let mut new_ind = None;
        for i in (0..self.cls.len()).filter(|&i| !self.is_watched(i)) {
            let test_lit = &self.cls[i];
            let curr_val = interp.get_val(test_lit);
            if curr_val.is_none() || curr_val.unwrap() {
                //debug!("Found new watchee: {:?}", test_lit);
                new_ind = Some(i);
                break;
            }
        }
        match new_ind {
            None => {
                if other_val.is_some() && !other_val.unwrap() {
                    PropRes::Conflict
                } else {
                    PropRes::Unit(other_lit)
                }
            }
            Some(ind) => {
                self.set_watched(&watch_info, ind);
                PropRes::NewWatch(self.cls[ind].clone())
            }
        }
    }
}

fn add_watched(watches: &mut VecMap<Vec<usize>>, lit: &Lit, ind: usize) {
    let id = lit.not().as_usize();
    match watches.entry(id) {
        Vacant(entry) => {
            entry.insert(vec![ind]);
        }
        Occupied(entry) => entry.into_mut().push(ind),
    }
}

fn get_impl_clause<'a>(
    cls: &'a Clause,
    confl_lit: Option<&'a Lit>,
) -> Box<dyn Iterator<Item = Lit> + 'a> {
    match confl_lit {
        None => Box::new(cls.iter().map(|l| l.not())),
        Some(confl_val) => Box::new(cls.iter().filter_map(move |lit| {
            if lit == confl_val {
                None
            } else {
                Some(lit.not())
            }
        })),
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct DecLevel(usize);

type Implicant = Option<usize>;
type DecInfo = (DecLevel, Implicant);
type WatcherList = VecMap<Vec<usize>>;
type WatchedFormula = Vec<WatchedClause>;

pub struct Solver {
    //Interpretation stack
    //the var we set to true, whether this was after a conflict and the interp
    interp_stack: Vec<(Lit, Interp, Map<DecInfo>)>,

    //Current interpretation
    interp: Interp,

    //Clauses and constraints
    clss: WatchedFormula,

    //Queue for unit propagations
    prop_queue: VecDeque<(Lit, Implicant)>,

    watches: WatcherList,

    //tracking decision level and implicants
    track: Map<DecInfo>,
}

enum Safety {
    Safe,
    Conflict,
}

impl Solver {
    fn level(&self) -> DecLevel {
        DecLevel(self.interp_stack.len())
    }

    fn find_var(&self) -> Option<Lit> {
        for x in self.clss.iter().flat_map(|x| x.cls.iter()) {
            if self.interp.get_val(x).is_none() {
                return Some(x.clone());
            }
        }
        None
    }

    fn set_true(&mut self, lit: &Lit, cause: Implicant) {
        self.interp.set_true(lit);
        let dec_lvl = self.level();
        let &Id(id) = lit.id();
        self.track.insert(id, (dec_lvl, cause));
    }

    fn add_clause(&mut self, this_lit: &Lit, cls: Clause) -> usize {
        let new_ind: usize = self.clss.len();
        let mut indices = (0, 0);
        let mut max_dec = 0;
        for i in 0..cls.len() {
            let lit = &cls[i];
            let &Id(id) = lit.id();
            if lit == this_lit {
                indices.0 = i;
            } else {
                self.track.get(id).map(|&(DecLevel(dec), _)| {
                    if dec > max_dec {
                        max_dec = dec;
                        indices.1 = i;
                    }
                });
            }
        }
        add_watched(&mut self.watches, &cls[indices.0], new_ind);
        add_watched(&mut self.watches, &cls[indices.1], new_ind);
        self.clss.push(WatchedClause {
            indices: indices,
            cls: cls,
        });
        new_ind
    }

    fn trace_conflict(&self, curr_dec_lvl: &DecLevel, confl: &Clause) -> (Clause, DecLevel) {
        let mut learned = Vec::new();
        let mut back_lvl = DecLevel(0);

        let mut lit_queue: VecDeque<Lit> = get_impl_clause(confl, None).collect();

        let mut seen = BitSet::new();

        while let Some(lit) = lit_queue.pop_front() {
            let &Id(id) = lit.id();
            if let Some(&(ref dec_lvl, cause_)) = self.track.get(id) {
                if seen.contains(id) {
                    continue;
                } else {
                    seen.insert(id);
                }

                if let Some(cause) = cause_ {
                    if dec_lvl == curr_dec_lvl {
                        let next_cause = &self.clss[cause].cls;
                        for lit in get_impl_clause(next_cause, Some(&lit)) {
                            lit_queue.push_back(lit);
                        }
                        //skip below 2 statements
                        continue;
                    }
                }
                learned.push(lit.not());
                back_lvl = max(back_lvl, *dec_lvl);
            }
        }
        (learned, back_lvl)
    }

    fn check_watchers(&mut self, lit: Lit) -> Implicant {
        self.watches
            .get(lit.as_usize())
            .cloned()
            .and_then(|clause_inds| {
                let mut new_inds = Vec::new();
                let mut conflict_seen = None;
                for cls_ind in clause_inds {
                    if conflict_seen.is_some() {
                        new_inds.push(cls_ind);
                        continue;
                    }
                    let bcp_res = self.clss[cls_ind].bcp(&self.interp, &lit);
                    if let PropRes::NewWatch(new_lit) = bcp_res {
                        add_watched(&mut self.watches, &new_lit, cls_ind);
                    } else {
                        new_inds.push(cls_ind);
                        if let PropRes::Conflict = bcp_res {
                            conflict_seen = Some(cls_ind);
                        } else if let PropRes::Unit(unit_lit) = bcp_res {
                            self.prop_queue.push_back((unit_lit, Some(cls_ind)));
                        }
                    }
                }
                self.watches.insert(lit.as_usize(), new_inds);
                conflict_seen
            })
    }

    fn decide_var(&mut self, lit: Option<Lit>) -> Option<Safety> {
        lit.or_else(|| self.find_var()).map(|decision| {
            // here we need to pick a new var
            // because we know nothing more is constrainted
            // pick it and add it to the stack
            info!(
                "Trying {:?}, set: {:?} -> {:?}",
                decision,
                decision.id(),
                decision.eval(true)
            );
            self.interp_stack
                .push((decision.clone(), self.interp.clone(), self.track.clone()));
            self.process(decision, None)
        })
    }

    fn backtrack(&mut self, cause: Clause, DecLevel(back_lvl): DecLevel) -> Safety {
        self.interp_stack.truncate(back_lvl);
        match self.interp_stack.pop() {
            //here use the learned clause as a cause
            Some((last, interp, trace)) => {
                let last_not = last.not();
                self.interp = interp;
                self.track = trace;
                let new_ind = self.add_clause(&last_not, cause);
                self.interp_stack
                    .push((last.not(), self.interp.clone(), self.track.clone()));
                self.prop_queue.clear();
                self.process(last_not, Some(new_ind))
            }
            None => {
                info!("Hit root level, UNSAT");
                Conflict
            }
        }
    }

    fn handle_conflict(&mut self, conf_i: usize) -> (Clause, DecLevel) {
        let mut dec_lvl = self.level();
        let (mut cause, mut back_lvl) = self.trace_conflict(&dec_lvl, &self.clss[conf_i].cls);
        while back_lvl < dec_lvl {
            dec_lvl = back_lvl;
            let trace = self.trace_conflict(&dec_lvl, &cause);
            //uuugly
            cause = trace.0;
            back_lvl = trace.1;
        }
        (cause, back_lvl)
    }

    fn process(&mut self, constr_lit: Lit, cause: Implicant) -> Safety {
        self.set_true(&constr_lit, cause);
        match self.check_watchers(constr_lit) {
            None => Safe,
            Some(cls_ind) => {
                //use this to find the back_lvl and the REAL confl cause
                let (confl_cls, back_lvl) = self.handle_conflict(cls_ind);
                self.backtrack(confl_cls, back_lvl)
            }
        }
    }

    fn process_queue(&mut self) -> Safety {
        while let Some((constr_lit, cause)) = self.prop_queue.pop_front() {
            let process = self.process(constr_lit, cause);
            if let Conflict = process {
                return Conflict;
            }
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
        let clss = formula
            .into_iter()
            .map(|cls| {
                for lit in cls.iter().take(2) {
                    add_watched(&mut watches, lit, ind);
                }
                ind = ind + 1;
                if cls.len() > 1 {
                    WatchedClause {
                        indices: (0, 1),
                        cls: cls,
                    }
                } else {
                    WatchedClause {
                        indices: (0, 0),
                        cls: cls,
                    }
                }
            })
            .collect();
        //debug!("Watched: {:?}",clss);
        //debug!("Watches: {:?}",watches);
        Solver {
            interp: interp.unwrap_or_else(|| Interp(VecMap::new())),
            interp_stack: Vec::new(),
            clss,
            prop_queue: VecDeque::new(),
            track: VecMap::new(),
            watches,
        }
    }

    fn solve(&mut self) -> Satness {
        //handle top level units
        for unit in self.clss.iter().filter_map(|c| {
            if c.cls.len() == 1 {
                Some(c.cls[0].clone())
            } else {
                None
            }
        }) {
            info!("Found top level unit: {:?}", unit);
            self.prop_queue.push_back((unit, None))
        }

        //main loop
        loop {
            let processing = match self.process_queue() {
                Safe => match self.decide_var(None) {
                    None => return SAT(self.interp.clone()),
                    Some(safety) => safety,
                },
                e => e,
            };

            if let Conflict = processing {
                let reason = format!("Found conflict");
                return UNSAT(reason);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::Lit::{N, P};
    use super::super::{Id, SATSolver};
    use super::Solver;

    use env_logger;

    #[test]
    fn test_nonchro_backtrack() {
        env_logger::init();

        let _clause1 = vec![N(Id(1)), P(Id(2))];
        let _clause2 = vec![N(Id(1)), P(Id(3)), P(Id(9))];
        let _clause3 = vec![N(Id(2)), N(Id(3)), P(Id(4))];
        let _clause4 = vec![N(Id(4)), P(Id(5)), P(Id(10))];
        let _clause5 = vec![N(Id(4)), P(Id(6)), P(Id(11))];
        let _clause6 = vec![N(Id(5)), N(Id(6))];
        let _clause7 = vec![P(Id(1)), P(Id(7)), N(Id(12))];
        let _clause8 = vec![P(Id(1)), P(Id(8))];
        let _clause9 = vec![N(Id(7)), N(Id(8)), N(Id(13))];
        let _clause10 = vec![N(Id(12)), P(Id(13))];
        let clause11 = vec![P(Id(10)), N(Id(11))];
        let mut solver: Solver = Solver::create(
            vec![
                _clause1, _clause2, _clause3, _clause4, _clause5, _clause6, _clause7, _clause8,
                _clause9, _clause10, clause11,
            ],
            None,
        );
        solver.decide_var(Some(N(Id(9))));
        solver.process_queue();
        solver.decide_var(Some(P(Id(12))));
        solver.process_queue();
        solver.decide_var(Some(N(Id(10))));
        solver.process_queue();
        solver.decide_var(Some(P(Id(14))));
        solver.process_queue();
        solver.decide_var(Some(P(Id(15))));

        solver.decide_var(Some(P(Id(1))));
        solver.process_queue();
        /*solver.interp_stack.push(
            (N(Id(9)),
             false,
             solver.interp.clone()
             ));
        solver.set_true(&P(Id(1)), None);
        solver.set_true(&P(Id(5)), Some(0));
        solver.interp_stack.push(
            (N(Id(2)),
             false,
             solver.interp.clone()
             ));
        solver.set_true(&N(Id(2)), None);
        solver.set_true(&N(Id(3)), Some(2));
        solver.set_true(&P(Id(4)), Some(3));
        debug!("Trace: {:?}", solver.trace_conflict(&P(Id(3)), 1, 4));
        */
    }
}
