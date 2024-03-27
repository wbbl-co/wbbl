use crate::constraint_solver_constraints::Constraint::{
    SameCompositeSize, SameDimensionality, SameTypes,
};
use crate::constraint_solver_constraints::ConstraintApplicationResult::{
    Contradiction, Dirty, Unchanged,
};
use crate::data_types::{CompositeSize, Dimensionality};
use crate::graph_types::PortId;
use std::collections::{HashMap, HashSet, LinkedList};
use std::hash::Hash;

pub enum ConstraintApplicationResult {
    Dirty(LinkedList<PortId>),
    Unchanged,
    Contradiction,
}

pub trait HasCompositeSize {
    fn get_composite_size(&self) -> Option<CompositeSize>;
}

pub trait HasDimensionality {
    fn get_dimensionality(&self) -> Option<Dimensionality>;
}

pub trait HasRanking {
    fn get_rank(&self) -> i32;
}

pub trait PortConstraint {
    fn get_affected_ports(&self) -> HashSet<PortId>;
    fn apply<Value: Copy + Hash + Eq + HasCompositeSize + HasDimensionality + HasRanking>(
        &self,
        assignments: &mut HashMap<PortId, Value>,
        domains: &mut HashMap<PortId, Vec<Value>>,
    ) -> ConstraintApplicationResult;
}

#[derive(Clone, PartialEq, Eq)]
pub enum Constraint {
    SameTypes(SameTypesConstraint),
    SameDimensionality(SameDimensionalityConstraint),
    SameCompositeSize(SameCompositeSizeConstraint),
}

impl Constraint {
    pub fn apply<Value: Copy + Hash + Eq + HasCompositeSize + HasDimensionality + HasRanking>(
        &self,
        assignments: &mut HashMap<PortId, Value>,
        domains: &mut HashMap<PortId, Vec<Value>>,
    ) -> ConstraintApplicationResult {
        match self {
            SameDimensionality(sd) => sd.apply(assignments, domains),
            SameTypes(st) => st.apply(assignments, domains),
            SameCompositeSize(scs) => scs.apply(assignments, domains),
        }
    }

    pub fn get_affected_ports(&self) -> HashSet<PortId> {
        match self {
            SameDimensionality(sd) => sd.get_affected_ports(),
            SameTypes(st) => st.get_affected_ports(),
            SameCompositeSize(scs) => scs.get_affected_ports(),
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct SameTypesConstraint {
    pub ports: HashSet<PortId>,
}

impl PortConstraint for SameTypesConstraint {
    fn get_affected_ports(&self) -> HashSet<PortId> {
        self.ports.clone()
    }

    fn apply<Value: Copy + Hash + Eq + HasRanking>(
        &self,
        assignments: &mut HashMap<PortId, Value>,
        domains: &mut HashMap<PortId, Vec<Value>>,
    ) -> ConstraintApplicationResult {
        let this_assignments: Vec<Value> = self
            .get_affected_ports()
            .into_iter()
            .filter_map(|p| assignments.get(&p))
            .map(|a| *a)
            .collect();
        let mut changed: LinkedList<PortId> = LinkedList::new();

        match this_assignments.iter().next() {
            Some(t) => {
                if this_assignments.iter().any(|other_t| other_t != t) {
                    Contradiction
                } else {
                    for p in self.get_affected_ports() {
                        let empty_vec: Vec<Value> = Vec::new();
                        let domain = domains.get(&p).unwrap_or(&empty_vec);
                        if !(domain.contains(t)) {
                            return Contradiction;
                        }
                        if assignments.get(&p) != Some(t) {
                            changed.push_back(p.clone());
                            assignments.insert(p.clone(), *t);
                            domains.insert(p.clone(), vec![*t]);
                        }
                    }
                    if !changed.is_empty() {
                        return Dirty(changed);
                    }
                    return Unchanged;
                }
            }
            None => {
                let these_domains: Vec<HashSet<Value>> = self
                    .get_affected_ports()
                    .into_iter()
                    .map(|p| domains.get(&p).unwrap().iter().map(|a| *a).collect())
                    .collect();
                let empty_set: HashSet<Value> = HashSet::new();
                let mut common_domains: Vec<Value> = these_domains
                    .into_iter()
                    .reduce(|prev: HashSet<Value>, domain: HashSet<Value>| {
                        prev.intersection(&domain).map(|d| *d).collect()
                    })
                    .unwrap_or(empty_set)
                    .into_iter()
                    .collect();
                if common_domains.is_empty() {
                    return Contradiction;
                }
                let common_domain_count = common_domains.len();
                if common_domain_count == 1 {
                    let common_domain = *common_domains.first().unwrap();
                    for p in self.get_affected_ports() {
                        assignments.insert(p.clone(), common_domain);
                        domains.insert(p.clone(), common_domains.clone());
                        changed.push_back(p.clone());
                    }
                } else {
                    common_domains.sort_by(|a, b| a.get_rank().cmp(&b.get_rank()));
                    for p in self.get_affected_ports() {
                        if common_domain_count != domains.get(&p).unwrap().len() {
                            domains.insert(p.clone(), common_domains.clone());
                            changed.push_back(p.clone());
                        }
                    }
                }
                if !changed.is_empty() {
                    return Dirty(changed);
                }
                Unchanged
            }
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct SameDimensionalityConstraint {
    ports: HashSet<PortId>,
}

impl PortConstraint for SameDimensionalityConstraint {
    fn get_affected_ports(&self) -> HashSet<PortId> {
        self.ports.clone()
    }

    fn apply<Value: Copy + Hash + Eq + HasDimensionality>(
        &self,
        assignments: &mut HashMap<PortId, Value>,
        domains: &mut HashMap<PortId, Vec<Value>>,
    ) -> ConstraintApplicationResult {
        let assigned_dimensionalities: Vec<Dimensionality> = self
            .get_affected_ports()
            .into_iter()
            .filter_map(|p| assignments.get(&p).and_then(|a| a.get_dimensionality()))
            .collect();
        let mut changed: LinkedList<PortId> = LinkedList::new();
        let empty_vec: Vec<Value> = Vec::new();

        match assigned_dimensionalities.iter().next() {
            Some(d) => {
                if assigned_dimensionalities.iter().any(|other_d| other_d != d) {
                    Contradiction
                } else {
                    for p in self.get_affected_ports() {
                        let old_domains = domains.get(&p).unwrap_or(&empty_vec);
                        let new_domains: Vec<Value> = old_domains
                            .iter()
                            .filter(|old| old.get_dimensionality() == Some(*d))
                            .map(|d| *d)
                            .collect();
                        let new_domains_count = new_domains.len();

                        if new_domains_count == 0 {
                            return Contradiction;
                        }
                        let old_domains_count = old_domains.len();
                        if new_domains_count != old_domains_count {
                            changed.push_back(p.clone());
                            if new_domains_count == 1 {
                                assignments.insert(p.clone(), *new_domains.first().unwrap());
                            }
                            domains.insert(p.clone(), new_domains);
                        }
                    }
                    if !changed.is_empty() {
                        return Dirty(changed);
                    }
                    return Unchanged;
                }
            }
            None => {
                let empty_set: HashSet<Option<Dimensionality>> = HashSet::new();
                let dimensionalities_for_ports: Vec<HashSet<Option<Dimensionality>>> = self
                    .get_affected_ports()
                    .into_iter()
                    .map(|p| {
                        domains
                            .get(&p)
                            .unwrap_or(&empty_vec)
                            .iter()
                            .map(|d| d.get_dimensionality())
                            .collect()
                    })
                    .collect();
                let intersection_of_dimensionalities: HashSet<Option<Dimensionality>> =
                    dimensionalities_for_ports
                        .into_iter()
                        .reduce(|prev, next| prev.intersection(&next).map(|d| *d).collect())
                        .unwrap_or(empty_set);
                for p in self.get_affected_ports() {
                    let old_domains = domains.get(&p).unwrap_or(&empty_vec);
                    let new_domains: Vec<Value> = old_domains
                        .iter()
                        .filter(|old| {
                            intersection_of_dimensionalities.contains(&old.get_dimensionality())
                        })
                        .map(|d| *d)
                        .collect();
                    let new_domains_count = new_domains.iter().count();

                    if new_domains_count == 0 {
                        return Contradiction;
                    }
                    let old_domains_count = old_domains.len();
                    if new_domains_count != old_domains_count {
                        changed.push_back(p.clone());
                        if new_domains_count == 1 {
                            assignments.insert(p.clone(), *new_domains.first().unwrap());
                        }
                        domains.insert(p.clone(), new_domains);
                    }
                }
                if !changed.is_empty() {
                    return Dirty(changed);
                }
                Unchanged
            }
        }
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct SameCompositeSizeConstraint {
    ports: HashSet<PortId>,
}

impl PortConstraint for SameCompositeSizeConstraint {
    fn get_affected_ports(&self) -> HashSet<PortId> {
        return self.ports.clone();
    }

    fn apply<Value: Copy + Hash + Eq + HasCompositeSize>(
        &self,
        assignments: &mut HashMap<PortId, Value>,
        domains: &mut HashMap<PortId, Vec<Value>>,
    ) -> ConstraintApplicationResult {
        let assigned_composite_sizes: Vec<CompositeSize> = self
            .get_affected_ports()
            .into_iter()
            .filter_map(|p| assignments.get(&p).and_then(|a| a.get_composite_size()))
            .collect();
        let mut changed: LinkedList<PortId> = LinkedList::new();
        let empty_vec: Vec<Value> = Vec::new();

        match assigned_composite_sizes.iter().next() {
            Some(d) => {
                if assigned_composite_sizes.iter().any(|other_d| other_d != d) {
                    Contradiction
                } else {
                    for p in self.get_affected_ports() {
                        let old_domains = domains.get(&p).unwrap_or(&empty_vec);
                        let new_domains: Vec<Value> = old_domains
                            .iter()
                            .filter(|old| old.get_composite_size() == Some(*d))
                            .map(|d| *d)
                            .collect();
                        let new_domains_count = new_domains.iter().count();

                        if new_domains_count == 0 {
                            return Contradiction;
                        }
                        let old_domains_count = old_domains.len();
                        if new_domains_count != old_domains_count {
                            changed.push_back(p.clone());
                            if new_domains_count == 1 {
                                assignments.insert(p.clone(), *new_domains.first().unwrap());
                            }
                            domains.insert(p.clone(), new_domains);
                        }
                    }
                    if !changed.is_empty() {
                        return Dirty(changed);
                    }
                    Unchanged
                }
            }
            None => {
                let empty_set: HashSet<Option<CompositeSize>> = HashSet::new();
                let composite_sizes_for_ports: Vec<HashSet<Option<CompositeSize>>> = self
                    .get_affected_ports()
                    .into_iter()
                    .map(|p| {
                        domains
                            .get(&p)
                            .unwrap_or(&empty_vec)
                            .iter()
                            .map(|d| d.get_composite_size())
                            .collect()
                    })
                    .collect();
                let intersection_of_compoosite_sizes: HashSet<Option<CompositeSize>> =
                    composite_sizes_for_ports
                        .into_iter()
                        .reduce(|prev, next| prev.intersection(&next).map(|d| *d).collect())
                        .unwrap_or(empty_set);
                for p in self.get_affected_ports() {
                    let old_domains = domains.get(&p).unwrap_or(&empty_vec);
                    let new_domains: Vec<Value> = old_domains
                        .iter()
                        .filter(|old| {
                            intersection_of_compoosite_sizes.contains(&old.get_composite_size())
                        })
                        .map(|d| *d)
                        .collect();
                    let new_domains_count = new_domains.len();

                    if new_domains_count == 0 {
                        return Contradiction;
                    }
                    let old_domains_count = old_domains.iter().count();
                    if new_domains_count != old_domains_count {
                        changed.push_back(p.clone());
                        if new_domains_count == 1 {
                            assignments.insert(p.clone(), *new_domains.first().unwrap());
                        }
                        domains.insert(p.clone(), new_domains);
                    }
                }
                if !changed.is_empty() {
                    return Dirty(changed);
                }
                Unchanged
            }
        }
    }
}
