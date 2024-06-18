use crate::constraint_solver::ConstraintSolverError::ContradictionFound;
use crate::constraint_solver_constraints::{
    Constraint, ConstraintApplicationResult, HasCompositeSize, HasDimensionality, HasRanking,
};
use crate::data_types::{AbstractDataType, ConcreteDataType};
use crate::graph_types::PortId;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use std::hash::Hash;
use std::rc::Rc;

#[derive(Debug, Serialize, Deserialize)]
pub enum ConstraintSolverError {
    ContradictionFound,
}

fn propagate_constraints<Value>(
    start_port: PortId,
    assignments: &mut im::HashMap<PortId, Value>,
    domains: &mut im::HashMap<PortId, Rc<Vec<Value>>>,
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
    topologically_ordered_ports: &[PortId],
    domains: im::HashMap<PortId, Rc<Vec<Value>>>,
    constraints: &HashMap<PortId, Vec<Constraint>>,
) -> Result<im::HashMap<PortId, Value>, ConstraintSolverError>
where
    Value: Debug + Copy + Hash + Eq + HasCompositeSize + HasDimensionality + HasRanking,
{
    let mut assignments_stack: VecDeque<im::HashMap<PortId, Value>> =
        VecDeque::from([im::HashMap::<PortId, Value>::new()]);
    let mut domains_stack: VecDeque<im::HashMap<PortId, Rc<Vec<Value>>>> =
        VecDeque::from([domains]);

    let mut i = 0;
    'outer: loop {
        if i >= topologically_ordered_ports.len() {
            return Ok(assignments_stack.pop_front().unwrap());
        }
        let assignments = assignments_stack.front().unwrap();
        let current_port = topologically_ordered_ports[i].clone();
        let maybe_assignment = assignments.get(&current_port);
        if maybe_assignment.is_some() {
            i += 1;
            continue 'outer;
        }
        let domains = domains_stack.front().unwrap();
        let maybe_domain = domains.get(&current_port);
        if maybe_domain.is_none() {
            return Err(ContradictionFound);
        }
        let domain = maybe_domain.unwrap();
        'inner: for t in domain.iter() {
            let mut next_assignments = assignments.clone();
            let mut next_domains = domains.clone();
            next_assignments.insert(current_port.clone(), *t);
            next_domains.insert(current_port.clone(), Rc::new(vec![*t]));
            let propagation_result = propagate_constraints(
                current_port.clone(),
                &mut next_assignments,
                &mut next_domains,
                constraints,
            );
            if let Err(ContradictionFound) = propagation_result {
                continue 'inner;
            }

            // We have a candidate type. Go to the next port
            i += 1;
            assignments_stack.push_front(next_assignments);
            domains_stack.push_front(next_domains);
            continue 'outer;
        }
        if i == 0 {
            return Err(ContradictionFound);
        } else {
            i -= 1;
            assignments_stack.pop_front();
            domains_stack.pop_front();
        }
    }
}

pub fn assign_concrete_types(
    topologically_ordered_ports: &[PortId],
    port_types: &HashMap<PortId, AbstractDataType>,
    constraints: &HashMap<PortId, Vec<Constraint>>,
) -> Result<HashMap<PortId, ConcreteDataType>, ConstraintSolverError> {
    let domains: im::HashMap<PortId, Rc<Vec<ConcreteDataType>>> = port_types
        .iter()
        .map(|t| (t.0.clone(), Rc::new(t.1.get_concrete_domain())))
        .collect();
    let result: HashMap<PortId, ConcreteDataType> =
        assign_types(topologically_ordered_ports, domains, constraints)?
            .into_iter()
            .collect();

    Ok(result)
}

pub fn narrow_abstract_types(
    topologically_ordered_ports: &[PortId],
    port_types: &HashMap<PortId, AbstractDataType>,
    constraints: &HashMap<PortId, Vec<Constraint>>,
) -> Result<HashMap<PortId, AbstractDataType>, ConstraintSolverError> {
    let domains: im::HashMap<PortId, Rc<Vec<AbstractDataType>>> = port_types
        .iter()
        .map(|t| (t.0.clone(), Rc::new(t.1.get_abstract_domain())))
        .collect();
    let result = assign_types(topologically_ordered_ports, domains, constraints)?
        .into_iter()
        .collect();

    Ok(result)
}
