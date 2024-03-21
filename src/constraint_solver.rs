use crate::constraint_solver::ConstraintSolverError::ContradictionFound;
use crate::constraint_solver_constraints::{
    Constraint, ConstraintApplicationResult, HasCompositeSize, HasDimensionality, HasRanking,
};
use crate::data_types::{AbstractDataType, ConcreteDataType};
use crate::graph_types::PortId;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::hash::Hash;

#[derive(Serialize, Deserialize)]
pub enum ConstraintSolverError {
    ContradictionFound,
}

fn propagate_constraints<Value>(
    start_port: PortId,
    assignments: &mut HashMap<PortId, Value>,
    domains: &mut HashMap<PortId, Vec<Value>>,
    constraints: &HashMap<PortId, Vec<Constraint>>,
) -> Result<(), ConstraintSolverError>
where
    Value: Copy + Hash + Eq + HasCompositeSize + HasDimensionality + HasRanking,
{
    let mut queue: VecDeque<PortId> = VecDeque::new();
    queue.push_front(start_port);
    let empty_constraints: Vec<Constraint> = vec![];
    while !queue.is_empty() {
        let current_port = queue.pop_front().unwrap();
        let constraints_for_current_port =
            constraints.get(&current_port).unwrap_or(&empty_constraints);
        for constraint in constraints_for_current_port {
            match constraint.apply(assignments, domains) {
                ConstraintApplicationResult::Dirty(dirty_ports) => {
                    for p in dirty_ports {
                        queue.push_front(p);
                    }
                }
                ConstraintApplicationResult::Unchanged => (),
                ConstraintApplicationResult::Contradiction => return Err(ContradictionFound),
            }
        }
    }
    Ok(())
}

fn assign_types<Value>(
    i: usize,
    topologically_ordered_ports: &Vec<PortId>,
    assignments: &mut HashMap<PortId, Value>,
    domains: &mut HashMap<PortId, Vec<Value>>,
    constraints: &HashMap<PortId, Vec<Constraint>>,
) -> Result<HashMap<PortId, Value>, ConstraintSolverError>
where
    Value: Copy + Hash + Eq + HasCompositeSize + HasDimensionality + HasRanking,
{
    if i >= topologically_ordered_ports.len() {
        return Ok(assignments.clone());
    }
    let current_port = topologically_ordered_ports[i].clone();

    let maybe_assignment = assignments.get(&current_port);
    if maybe_assignment.is_some() {
        return assign_types(
            i + 1,
            topologically_ordered_ports,
            assignments,
            domains,
            constraints,
        );
    }

    let maybe_domain = domains.get(&current_port);
    if maybe_domain.is_none() {
        return Err(ContradictionFound);
    }
    let domain = maybe_domain.unwrap();
    for t in domain {
        let mut next_assignments = assignments.clone();
        let mut next_domains = domains.clone();
        next_assignments.insert(current_port.clone(), *t);
        next_domains.insert(current_port.clone(), vec![*t]);
        let propagation_result = propagate_constraints(
            current_port.clone(),
            &mut next_assignments,
            &mut next_domains,
            constraints,
        );
        if let Err(ContradictionFound) = propagation_result {
            continue;
        }
        let recursive_result = assign_types(
            i + 1,
            topologically_ordered_ports,
            &mut next_assignments,
            &mut next_domains,
            constraints,
        );
        if let Ok(ass) = recursive_result {
            return Ok(ass);
        }
    }
    Err(ContradictionFound)
}

pub fn assign_concrete_types(
    topologically_ordered_ports: &Vec<PortId>,
    port_types: &HashMap<PortId, AbstractDataType>,
    constraints: &HashMap<PortId, Vec<Constraint>>,
) -> Result<HashMap<PortId, ConcreteDataType>, ConstraintSolverError> {
    let mut domains: HashMap<PortId, Vec<ConcreteDataType>> = port_types
        .iter()
        .map(|t| (t.0.clone(), t.1.get_concrete_domain()))
        .collect();
    let mut assignments: HashMap<PortId, ConcreteDataType> = HashMap::new();
    assign_types(
        0,
        topologically_ordered_ports,
        &mut assignments,
        &mut domains,
        constraints,
    )
}

pub fn narrow_abstract_types(
    topologically_ordered_ports: &Vec<PortId>,
    port_types: &HashMap<PortId, AbstractDataType>,
    constraints: &HashMap<PortId, Vec<Constraint>>,
) -> Result<HashMap<PortId, AbstractDataType>, ConstraintSolverError> {
    let mut domains: HashMap<PortId, Vec<AbstractDataType>> = port_types
        .iter()
        .map(|t| (t.0.clone(), t.1.get_abstract_domain()))
        .collect();
    let mut assignments: HashMap<PortId, AbstractDataType> = HashMap::new();
    assign_types(
        0,
        topologically_ordered_ports,
        &mut assignments,
        &mut domains,
        constraints,
    )
}
