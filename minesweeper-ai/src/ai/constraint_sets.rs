//! TODO: Docs
use super::{constraints::Constraint, coord_set::CoordSet, CellContent, Decision, KnownMinefield};

#[derive(Debug, Clone, Default)]
/// TODO: Docs
pub struct CoupledSets<const W: usize, const H: usize>(pub Vec<ConstraintSet<W, H>>);

impl<const W: usize, const H: usize> CoupledSets<W, H> {
    /// TODO: Docs
    #[must_use]
    pub fn insert(
        &mut self,
        constraint: Constraint<W, H>,
        known_minefield: &mut KnownMinefield<W, H>,
    ) -> Option<Vec<Decision<W, H>>> {
        // Returns mutably all the constraint sets that contain any of the
        // variables in the new constraints, and their indexes
        let (mut indexes, sets): (Vec<usize>, Vec<&mut ConstraintSet<W, H>>) = self
            .0
            .iter_mut()
            .enumerate()
            .filter(|(_, constraint_set)| {
                constraint
                    .variables
                    .iter()
                    .any(|v| constraint_set.variables.contains(*v))
            })
            .unzip();

        // Combine all retrieved constraints into the first constraint
        let constraint_set = sets.into_iter().reduce(|a, b| a.drain_from(b));

        // If a constraint set was found, insert the constraint set in it,
        // otherwise create a new set.
        let decisions = if let Some(set) = constraint_set {
            set.insert(constraint, known_minefield)
        } else {
            self.0.push(ConstraintSet::default());
            let set = self.0.get_mut(0).unwrap();
            set.insert(constraint, known_minefield)
        };

        // Remove all other constraint sets
        if !indexes.is_empty() {
            indexes.remove(0);
            for index in indexes.iter().rev() {
                self.0.remove(*index);
            }
        }

        decisions
    }

    /// TODO: Docs
    pub fn check_splits(&mut self) {
        let mut new_vec = Vec::new();
        while let Some(set) = self.0.pop() {
            new_vec.extend(set.check_splits());
        }
        self.0 = new_vec;
    }
}

/// Coupled Constraints
#[derive(Debug, Clone, Default)]
pub struct ConstraintSet<const W: usize, const H: usize> {
    /// List of label-mine-location-constraints for a given state
    pub constraints: Vec<Constraint<W, H>>,
    /// List of all the variables that are in this set of coupled constraints
    pub variables: CoordSet<W, H>,
}
impl<const W: usize, const H: usize> PartialEq for ConstraintSet<W, H> {
    fn eq(&self, other: &Self) -> bool {
        let mut a = self.constraints.clone();
        let mut b = other.constraints.clone();
        a.sort();
        b.sort();
        a == b && self.variables == other.variables
    }
}

impl<const W: usize, const H: usize> ConstraintSet<W, H> {
    /// TODO: Docs
    pub fn drain_from(&mut self, other: &mut ConstraintSet<W, H>) -> &mut ConstraintSet<W, H> {
        self.constraints.append(&mut other.constraints);
        self.variables.extend(&other.variables);
        self.constraints.sort();
        self.constraints.dedup();
        self
    }

    /// TOOD: Docs
    pub fn check_splits(self) -> Vec<ConstraintSet<W, H>> {
        let ConstraintSet {
            mut constraints,
            variables: _,
        } = self;

        let mut sets: Vec<ConstraintSet<W, H>> = Vec::new();

        'outer: while let Some(constraint) = constraints.pop() {
            for set in &mut sets {
                if constraint
                    .variables
                    .iter()
                    .any(|v| set.variables.contains(*v))
                {
                    set.variables
                        .insert_many(constraint.variables.iter().copied());
                    set.constraints.push(constraint);
                    continue 'outer;
                }
            }
            let mut variables = CoordSet::default();
            variables.insert_many(constraint.variables.iter().copied());
            sets.push(ConstraintSet {
                constraints: vec![constraint],
                variables,
            })
        }

        sets
    }

    /// TODO: Docs
    #[must_use]
    pub fn insert(
        &mut self,
        mut constraint: Constraint<W, H>,
        known_field: &mut KnownMinefield<W, H>,
    ) -> Option<Vec<Decision<W, H>>> {
        if !constraint.is_empty() && !self.constraints.contains(&constraint) {
            let decisions = ConstraintSet::solve_trivial_constraint(&mut constraint, known_field);
            if !decisions.is_empty() {
                Some(decisions)
            } else {
                self.variables
                    .insert_many(constraint.variables.iter().copied());
                self.constraints.push(constraint);
                None
            }
        } else {
            None
        }
    }

    /// TODO: Docs
    #[must_use]
    pub fn clear_known_variables(
        &mut self,
        known_field: &KnownMinefield<W, H>,
    ) -> Vec<Decision<W, H>> {
        let mut decisions = Vec::new();
        for (exists, coord) in self.variables.iter_mut() {
            if let CellContent::Known(val) = known_field.get(coord) {
                let mut idx = 0;
                while let Some(constraint) = self.constraints.get_mut(idx) {
                    while let Some(idx) = constraint.variables.iter().position(|v| *v == coord) {
                        constraint.variables.remove(idx);
                        constraint.label -= val as u8;
                    }
                    if constraint.is_empty() {
                        self.constraints.remove(idx);
                    } else {
                        idx += 1;
                    }
                }

                *exists = false;
                if val {
                    decisions.push(Decision::Flag(coord));
                } else {
                    decisions.push(Decision::Reveal(coord));
                }
            }
        }

        decisions
    }

    /// Solves trivial cases, meaning that it will reveal all variables that
    /// have an obvious answer.
    #[must_use]
    pub fn solve_trivial_cases(
        &mut self,
        known_field: &mut KnownMinefield<W, H>,
    ) -> Vec<Decision<W, H>> {
        let mut decisions = Vec::new();
        let mut idx = 0;
        while let Some(constraint) = self.constraints.get_mut(idx) {
            let d = ConstraintSet::solve_trivial_constraint(constraint, known_field);
            if !d.is_empty() {
                decisions.extend(d);
                self.constraints.remove(idx);
            } else {
                idx += 1;
            }
        }
        decisions
    }

    /// TODO: Docs
    #[must_use]
    pub fn solve_trivial_constraint(
        constraint: &mut Constraint<W, H>,
        known_field: &mut KnownMinefield<W, H>,
    ) -> Vec<Decision<W, H>> {
        let mut decisions = Vec::new();
        let mut old_decision_len = 0;

        while {
            let mut idx = 0;
            while let Some(var) = constraint.variables.get(idx) {
                if let CellContent::Known(val) = known_field.get(*var) {
                    constraint.label -= val as u8;
                    constraint.variables.remove(idx);
                } else {
                    idx += 1;
                }
            }

            if constraint.label == 0 {
                for variable in &constraint.variables {
                    known_field.set(*variable, CellContent::Known(false));
                    decisions.push(Decision::Reveal(*variable));
                }
            } else if constraint.label as usize == constraint.variables.len() {
                for variable in &constraint.variables {
                    known_field.set(*variable, CellContent::Known(true));
                    decisions.push(Decision::Flag(*variable));
                }
            } else {
            }

            old_decision_len > decisions.len()
        } {
            old_decision_len = decisions.len();
        }
        decisions
    }

    /// TODO: Docs
    pub fn reduce(&mut self) {
        let mut edited = true;
        while edited {
            edited = false;
            // TODOS:
            // 1. self.constraints could be a HashSet with a priority queue?
            // 2. tests are broken because constraints aren't checked for
            //    duplicates again
            // 3. make tests for CoordSet
            // 4. implement Ord/partialOrd for constraint, which #1 orders by
            //    length, and when len are the same, order by the default method
            //    (?) for dedup
            // 5. Might be possible to combine trivial-solving and clearing known variables
            self.constraints.sort_by_key(|i| i.len());

            for smallest_idx in 0..self.constraints.len() {
                let (smaller, others) = self.constraints.split_at_mut(smallest_idx + 1);
                let smallest = &mut smaller[smaller.len() - 1];

                if !smallest.is_empty() {
                    for other in &mut *others {
                        if other.len() > smallest.len() && other.is_superset_of(smallest) {
                            #[cfg(test)]
                            dbg!(&other, &smallest);
                            other.subtract(smallest);
                            edited = true;
                        }
                    }
                    if edited {
                        break;
                    }
                }
            }
        }

        self.constraints.sort();
        self.constraints.dedup();
    }
}