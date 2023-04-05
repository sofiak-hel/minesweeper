//! This module contains everything related to specifically solving the
//! Constraint Satisfaction Problem.

use arrayvec::ArrayVec;
use miinaharava::minefield::{Cell, Coord, Minefield};
use std::{collections::HashSet, fmt::Debug};

use crate::ai::Decision;

/// Represents a single constraint where the variables represent tiles that are
/// still unknown to some degree, and the label represents the value that the
/// variables need to add up to.
///
/// In concrete terms, variables are hidden unflagged cells and the label is how many
/// mines are still undiscovered in said cells.
#[derive(Clone, PartialOrd, Ord, Eq)]
pub struct Constraint<const W: usize, const H: usize> {
    /// Value or label for the variables
    pub label: u8,
    /// List of coordinates to represent the variables that add up to the label.
    pub variables: ArrayVec<Coord<W, H>, 8>,
}

impl<const W: usize, const H: usize> Constraint<W, H> {
    /// TODO: Docs
    pub fn len(&self) -> usize {
        self.variables.len()
    }

    /// TODO: Docs
    pub fn is_empty(&self) -> bool {
        self.variables.is_empty()
    }

    /// TODO: Docs
    /// Maybe rename me?? Should actually be is_subset_OR_is_superset
    pub fn is_superset_of(&self, other: &Constraint<W, H>) -> bool {
        let mut a = self.variables.clone();
        let mut b = other.variables.clone();
        a.sort();
        b.sort();

        // TODO: OPTIMIZE ME LATER, IM SLOW
        b.iter().all(|item| a.contains(item))
        // let mut a_iter = a.iter();
        // 'outer: for other_item in &b {
        //     for item in a_iter.by_ref() {
        //         if item == other_item {
        //             // Should be break, but clippy actually maybe reports a
        //             // false negative here
        //             continue 'outer;
        //         }
        //     }
        //     return false;
        // }
        // true
    }

    /// TODO: Docs
    pub fn subtract(&mut self, other: &Constraint<W, H>) {
        self.variables.retain(|v| !other.variables.contains(v));
        self.label -= other.label;
    }
}

impl<const W: usize, const H: usize> Debug for Constraint<W, H> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} = ", self.label)?;
        for (i, coord) in self.variables.iter().enumerate() {
            write!(f, "{:?}", coord)?;
            if i < self.variables.len() - 1 {
                write!(f, " + ")?;
            }
        }
        write!(f, "(len: {})", self.variables.len())?;
        Ok(())
    }
}

impl<const W: usize, const H: usize> PartialEq for Constraint<W, H> {
    fn eq(&self, other: &Self) -> bool {
        let mut a = self.variables.clone();
        let mut b = other.variables.clone();
        a.sort();
        b.sort();
        a == b && self.label == other.label
    }
}

/// Custom error type for any failure states that might occur.
#[derive(Debug)]
pub enum CSPError {}

/// General state used for solving Constraint Satisfication Problem
#[derive(Debug, Clone, Default)]
pub struct ConstaintSatisficationState<const W: usize, const H: usize> {
    /// List of label-mine-location-constraints for a given state
    pub constraints: ConstraintSets<W, H>,
}

impl<const W: usize, const H: usize> ConstaintSatisficationState<W, H> {
    /// Constructs a CPS-state from a given minefield. Goes through all labels
    /// in the visual field and creates a constraint from them.
    pub fn from(minefield: &Minefield<W, H>) -> Self {
        let mut constraints = ConstraintSets(Vec::with_capacity(W * H));

        for (y, row) in minefield.field.iter().enumerate() {
            for (x, cell) in row.iter().enumerate() {
                if let Cell::Label(mut num) = cell {
                    let mut neighbors = ArrayVec::new();
                    for neighbor in Coord::<W, H>(x as u8, y as u8).neighbours().iter() {
                        match minefield.field.get(*neighbor) {
                            Cell::Flag => num -= 1,
                            Cell::Hidden => neighbors.push(*neighbor),
                            _ => {}
                        };
                    }
                    if num > 0 || !neighbors.is_empty() {
                        let constraint = Constraint {
                            label: num,
                            variables: neighbors,
                        };
                        constraints.insert(constraint);
                    }
                }
            }
        }
        ConstaintSatisficationState { constraints }
    }

    /// Solves trivial cases, meaning that it will reveal all variables that
    /// have an obvious answer.
    pub fn solve_trivial_cases(&self) -> Result<Vec<Decision<W, H>>, CSPError> {
        let mut decisions = Vec::new();
        for constraint_set in &self.constraints.0 {
            for constraint in &constraint_set.constraints {
                if constraint.label as usize == constraint.variables.len() {
                    for variable in &constraint.variables {
                        decisions.push(Decision::Flag(*variable));
                    }
                }
                if constraint.label == 0 {
                    for variable in &constraint.variables {
                        decisions.push(Decision::Reveal(*variable));
                    }
                }
            }
        }
        decisions.sort();
        decisions.dedup();

        Ok(decisions)
    }
}

#[derive(Debug, Clone, Default)]
/// TODO: Docs
pub struct ConstraintSets<const W: usize, const H: usize>(Vec<CoupledConstraints<W, H>>);

impl<const W: usize, const H: usize> ConstraintSets<W, H> {
    /// TODO: Docs
    pub fn insert(&mut self, constraint: Constraint<W, H>) {
        // Returns mutably all the constraint sets that contain any of the
        // variables in the new constraints, and their indexes
        let (mut indexes, sets): (Vec<usize>, Vec<&mut CoupledConstraints<W, H>>) = self
            .0
            .iter_mut()
            .enumerate()
            .filter(|(_, constraint_set)| {
                constraint
                    .variables
                    .iter()
                    .any(|v| constraint_set.variables.contains(v))
            })
            .unzip();

        // Combine all retrieved constraints into the first constraint
        let constraints = sets.into_iter().reduce(|a, b| a.combine(b));

        // If a constraint set was found, insert the constraint set in it,
        // otherwise create a new set.
        if let Some(constraints) = constraints {
            constraints.insert(constraint);
        } else {
            self.0.push(CoupledConstraints::from(constraint))
        }

        // Remove all other constraint sets
        if !indexes.is_empty() {
            indexes.remove(0);
            for index in indexes.iter().rev() {
                self.0.remove(*index);
            }
        }
    }
}

/// Coupled Constraints
#[derive(Debug, Clone, Default)]
pub struct CoupledConstraints<const W: usize, const H: usize> {
    /// List of label-mine-location-constraints for a given state
    pub constraints: Vec<Constraint<W, H>>,
    /// List of all the variables that are in this set of coupled constraints
    pub variables: HashSet<Coord<W, H>>,
}

impl<const W: usize, const H: usize> CoupledConstraints<W, H> {
    /// TODO: Docs
    pub fn from(constraint: Constraint<W, H>) -> CoupledConstraints<W, H> {
        CoupledConstraints {
            variables: HashSet::from_iter(constraint.variables.clone().into_iter()),
            constraints: vec![constraint],
        }
    }

    /// TODO: Docs
    pub fn combine(
        &mut self,
        other: &mut CoupledConstraints<W, H>,
    ) -> &mut CoupledConstraints<W, H> {
        self.constraints.extend(other.constraints.iter().cloned());
        self.variables.extend(other.variables.iter().cloned());
        self.constraints.sort();
        self.constraints.dedup();
        self
    }

    /// TODO: Docs
    pub fn insert(&mut self, constraint: Constraint<W, H>) {
        self.variables.extend(constraint.variables.iter());
        if (constraint.label > 0 || !constraint.is_empty())
            && !self.constraints.contains(&constraint)
        {
            self.constraints.push(constraint);
            self.reduce();
        }
    }

    /// TODO: Docs
    pub fn reduce(&mut self) {
        let mut edited = true;
        while edited {
            edited = false;
            self.constraints.sort();
            self.constraints.dedup();
            self.constraints.sort_by_key(|i| i.len());

            for smallest_idx in 0..self.constraints.len() {
                let (smaller, others) = self.constraints.split_at_mut(smallest_idx + 1);
                let smallest = &mut smaller[smaller.len() - 1];

                for other in &mut *others {
                    if other.len() > smallest.len() && other.is_superset_of(smallest) {
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
}
