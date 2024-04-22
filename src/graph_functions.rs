use crate::constraint_solver::{self, assign_concrete_types, ConstraintSolverError};
use crate::constraint_solver_constraints::Constraint::SameTypes;
use crate::constraint_solver_constraints::{Constraint, SameTypesConstraint};
use crate::data_types::{AbstractDataType, ComputationDomain, ConcreteDataType};
use crate::graph_types::{
    BranchedMultiGraph, BranchedSubgraph, Graph, InputPort, InputPortId, MultiGraph, PortId,
    Subgraph,
};

use std::collections::{HashMap, HashSet, VecDeque};

fn has_no_dependencies(node: u128, graph: &Graph, visited_nodes: &HashSet<u128>) -> bool {
    let node = graph.nodes.get(&node).unwrap();
    node.input_ports_ids().iter().all(|p| {
        let input_port = graph.input_ports.get(&p).unwrap();
        if input_port.incoming_edge.is_none() {
            return true;
        }
        let incoming_edge_id = input_port.incoming_edge.unwrap();
        let incoming_edge = graph.edges.get(&incoming_edge_id).unwrap();
        let output_port = incoming_edge.output_port.clone();
        let other_node = graph.output_ports.get(&output_port).unwrap().id.node_id;
        visited_nodes.contains(&other_node)
    })
}

pub fn topologically_order_nodes(graph: &Graph) -> Vec<u128> {
    let mut visited_nodes: HashSet<u128> = HashSet::new();
    let mut results: Vec<u128> = graph
        .nodes
        .keys()
        .filter(|n| has_no_dependencies(**n, graph, &visited_nodes))
        .map(|n| *n)
        .collect();
    let mut i: usize = 0;
    while i < results.len() {
        let node_id = results[i];
        visited_nodes.insert(node_id);
        let node = graph.nodes.get(&node_id).unwrap();
        let successor_nodes: Vec<u128> = node
            .output_ports_ids()
            .iter()
            .map(|p| graph.output_ports.get(&p).unwrap())
            .flat_map(|p| p.outgoing_edges.clone())
            .map(|e| graph.edges.get(&e).unwrap().input_port.clone())
            .map(|p| graph.input_ports.get(&p).unwrap().id.node_id)
            .collect();
        for successor in successor_nodes {
            if has_no_dependencies(successor, graph, &visited_nodes) {
                results.push(successor);
            }
        }
        i += 1;
    }
    results
}

pub fn topologically_order_ports(graph: &Graph, ordered_nodes: &Vec<u128>) -> Vec<PortId> {
    ordered_nodes
        .iter()
        .flat_map(|n| {
            let node = graph.nodes.get(&n).unwrap();
            node.port_ids()
        })
        .collect()
}

fn label_nodes<Selector>(graph: &Graph, label_selector: &Selector) -> HashMap<u128, HashSet<u128>>
where
    Selector: Fn(&InputPort) -> Option<u128>,
{
    let start_node_id = graph.id;
    let start_node = graph.nodes.get(&start_node_id).unwrap();
    let mut queue: VecDeque<(u128, InputPortId)> = start_node
        .input_ports_ids()
        .iter()
        .filter_map(|p| graph.input_ports.get(&p))
        .filter(|p| p.incoming_edge.is_some())
        .map(|p| (label_selector(p).unwrap(), p.id.clone()))
        .collect();

    let mut result: HashMap<u128, HashSet<u128>> = HashMap::new();

    while !queue.is_empty() {
        let (label, port_id) = queue.pop_front().unwrap();
        let port = graph.input_ports.get(&port_id).unwrap();
        let output_port_id = graph
            .edges
            .get(&port.incoming_edge.unwrap())
            .unwrap()
            .output_port
            .clone();
        let node_id = graph.output_ports.get(&output_port_id).unwrap().id.node_id;
        let node = graph.nodes.get(&node_id).unwrap();
        let branch_tags = result.entry(node_id).or_insert(HashSet::new());
        branch_tags.insert(label);
        for port_id in node.input_ports_ids().iter() {
            let port = graph.input_ports.get(&port_id).unwrap();
            if let Some(_) = port.incoming_edge {
                let new_label = label_selector(port).unwrap_or(label);
                queue.push_back((new_label, port_id.clone()));
            }
        }
    }
    result
}

pub fn label_branches(graph: &Graph) -> HashMap<u128, HashSet<u128>> {
    label_nodes(graph, &|p: &InputPort| p.new_branch_id)
}

pub fn label_subgraphs(graph: &Graph) -> HashMap<u128, HashSet<u128>> {
    label_nodes(graph, &|p: &InputPort| p.new_subgraph_id)
}

pub fn label_computation_domains(
    graph: &Graph,
    node_ordering: &Vec<u128>,
) -> HashMap<u128, HashSet<ComputationDomain>> {
    let mut result: HashMap<u128, HashSet<ComputationDomain>> = HashMap::new();
    for node_id in node_ordering.iter() {
        let mut domain: HashSet<ComputationDomain> = HashSet::new();
        let node = graph.nodes.get(node_id).unwrap();
        if let Some(node_domain) = node.get_computation_domain() {
            domain = domain.union(&node_domain).map(|d| *d).collect();
        }
        for input_port_id in node.input_ports_ids().iter() {
            let input_port = graph.input_ports.get(input_port_id).unwrap();
            if let Some(edge_id) = input_port.incoming_edge {
                let edge = graph.edges.get(&edge_id).unwrap();
                let output_port_id = edge.output_port.clone();
                let output_port = graph.output_ports.get(&output_port_id).unwrap();
                let other_node_id = output_port.id.node_id;
                if let Some(other_domain) = result.get(&other_node_id) {
                    domain = domain.union(&other_domain.clone()).map(|d| *d).collect();
                }
            }
        }
        result.insert(*node_id, domain);
    }

    result
}

fn map_constraints_to_ports(graph: &Graph) -> HashMap<PortId, Vec<Constraint>> {
    let mut constraints_with_edges: Vec<Constraint> =
        graph.nodes.values().flat_map(|x| x.constraints()).collect();
    let mut edge_constraints = graph
        .edges
        .values()
        .map(|e| {
            SameTypes(SameTypesConstraint {
                ports: HashSet::from([
                    PortId::Input(e.input_port.clone()),
                    PortId::Output(e.output_port.clone()),
                ]),
            })
        })
        .collect();
    constraints_with_edges.append(&mut edge_constraints);
    let constraints_list: Vec<(PortId, Constraint)> = constraints_with_edges
        .iter()
        .flat_map(|c| -> Vec<(PortId, Constraint)> {
            c.get_affected_ports()
                .into_iter()
                .map(|p| (p, c.clone()))
                .collect()
        })
        .collect();
    let constraints: HashMap<PortId, Vec<Constraint>> = constraints_list.iter().fold(
        HashMap::new(),
        |mut map: HashMap<PortId, Vec<Constraint>>, kv: &(PortId, Constraint)| {
            let clone = kv.1.clone();
            let entry = map.entry(kv.0.clone()).or_insert(Vec::new());
            entry.push(clone);
            map
        },
    );
    constraints
}

pub fn concretise_types_in_graph(
    graph: &Graph,
    node_ordering: &Vec<u128>,
) -> Result<HashMap<PortId, ConcreteDataType>, ConstraintSolverError> {
    let ordered_ports = topologically_order_ports(graph, node_ordering);
    let constraints = map_constraints_to_ports(graph);
    let mut port_domains_vec: Vec<(PortId, AbstractDataType)> = graph
        .input_ports
        .iter()
        .map(|t| (PortId::Input(t.0.clone()), t.1.abstract_data_type))
        .collect();
    let mut output_port_domains_vec: Vec<(PortId, AbstractDataType)> = graph
        .output_ports
        .iter()
        .map(|t| (PortId::Output(t.0.clone()), t.1.abstract_data_type))
        .collect();
    port_domains_vec.append(&mut output_port_domains_vec);
    let port_types: HashMap<PortId, AbstractDataType> = port_domains_vec.into_iter().collect();
    assign_concrete_types(&ordered_ports, &port_types, &constraints)
}

pub fn prune_graph(graph: &mut Graph, subgraph_tags: &HashMap<u128, Vec<u128>>) {
    let node_ids: Vec<u128> = graph.nodes.keys().map(|n| *n).collect();
    for node_id in node_ids {
        let node = graph.nodes.get(&node_id).unwrap().clone();
        if !subgraph_tags.contains_key(&node_id) || node_id != graph.id {
            graph.nodes.remove(&node_id);
            let input_ports = node.input_ports_ids();
            let output_ports = node.output_ports_ids();
            for input_port_id in input_ports {
                let input_port = graph.input_ports.get(&input_port_id).unwrap();
                if let Some(edge_id) = input_port.incoming_edge.clone() {
                    let edge = graph.edges.get(&edge_id).unwrap();
                    let input_port_id = edge.input_port.clone();
                    let output_port_id = edge.output_port.clone();
                    let input_port = graph.input_ports.get_mut(&input_port_id).unwrap();
                    let output_port = graph.output_ports.get_mut(&output_port_id).unwrap();
                    input_port.incoming_edge = None;
                    output_port.outgoing_edges = output_port
                        .outgoing_edges
                        .iter()
                        .filter(|e| **e != edge_id)
                        .map(|e| *e)
                        .collect();
                }
                graph.input_ports.remove(&input_port_id);
            }
            for output_port_id in output_ports {
                let output_port = graph.output_ports.get(&output_port_id).unwrap();
                for edge_id in output_port.outgoing_edges.clone() {
                    let edge = graph.edges.get(&edge_id).unwrap();
                    let input_port_id = edge.input_port.clone();
                    let output_port_id = edge.output_port.clone();
                    let input_port = graph.input_ports.get_mut(&input_port_id).unwrap();
                    let output_port = graph.output_ports.get_mut(&output_port_id).unwrap();
                    input_port.incoming_edge = None;
                    output_port.outgoing_edges = output_port
                        .outgoing_edges
                        .iter()
                        .filter(|e| **e != edge_id)
                        .map(|e| *e)
                        .collect();
                }
                graph.output_ports.remove(&output_port_id);
            }
        }
    }
}

pub fn decompose_subgraphs(
    graph: Graph,
    subgraph_tags: &HashMap<u128, HashSet<u128>>,
    node_ordering: &Vec<u128>,
) -> MultiGraph {
    let mut result: MultiGraph = MultiGraph {
        graph,
        subgraphs: HashMap::new(),
        subgraph_ordering: vec![],
        dependencies: HashMap::new(),
    };
    let empty_set = HashSet::new();
    for node_id in node_ordering.iter() {
        let subgraph_ids = subgraph_tags.get(node_id).unwrap_or(&empty_set);
        for subgraph_id in subgraph_ids {
            if !result.subgraphs.contains_key(subgraph_id) {
                result.subgraph_ordering.push(*subgraph_id);
                result.subgraphs.insert(
                    *subgraph_id,
                    Subgraph {
                        id: *subgraph_id,
                        nodes: vec![],
                    },
                );
                result.dependencies.insert(*subgraph_id, HashSet::new());
            }
            let graph = result.subgraphs.get_mut(subgraph_id).unwrap();
            graph.nodes.push(*node_id);
        }
        if result.subgraphs.contains_key(node_id) {
            // This is a subgraph node. Lets get its dependencies
            let mut dependencies = HashSet::new();
            for dependant_node in result.subgraphs.get(node_id).unwrap().nodes.iter() {
                if result.subgraphs.contains_key(dependant_node) {
                    // This is a subgraph node and therefore a dependency of node_id
                    dependencies.insert(*dependant_node);
                }
            }
            result.dependencies.insert(*node_id, dependencies);
        }
    }

    result
}

pub fn decompose_branches(
    multi_graph: MultiGraph,
    branch_tags: &HashMap<u128, HashSet<u128>>,
) -> BranchedMultiGraph {
    let mut result = BranchedMultiGraph {
        graph: multi_graph.graph,
        subgraphs: HashMap::new(),
        subgraph_ordering: multi_graph.subgraph_ordering,
        dependencies: multi_graph.dependencies,
    };
    let empty_set = HashSet::new();
    for kv in multi_graph.subgraphs {
        let graph_id = kv.0;
        let multigraph_subgraph = kv.1;
        let mut branched_subgraph = BranchedSubgraph {
            id: graph_id,
            nodes: vec![],
            branches: HashMap::new(),
        };
        for node_id in multigraph_subgraph.nodes.iter() {
            let these_branch_tags = branch_tags.get(node_id).unwrap_or(&empty_set);
            if these_branch_tags.len() == 1 {
                let branch_tag = these_branch_tags.iter().next().unwrap();
                let branch = branched_subgraph
                    .branches
                    .entry(*branch_tag)
                    .or_insert(Vec::new());
                branch.push(*node_id);
            } else {
                branched_subgraph.nodes.push(*node_id);
            }
        }
        result.subgraphs.insert(graph_id, branched_subgraph);
    }

    result
}

pub fn narrow_abstract_types(
    graph: &Graph,
) -> Result<HashMap<PortId, AbstractDataType>, ConstraintSolverError> {
    // TODO: Make this a gradual algorithm
    let ordered_nodes = topologically_order_nodes(graph);
    let ordered_ports = topologically_order_ports(graph, &ordered_nodes);
    let mut port_types: Vec<(PortId, AbstractDataType)> = graph
        .input_ports
        .values()
        .map(|p| (PortId::Input(p.id.clone()), p.abstract_data_type.clone()))
        .collect();
    let mut output_port_types: Vec<(PortId, AbstractDataType)> = graph
        .output_ports
        .values()
        .map(|p| (PortId::Output(p.id.clone()), p.abstract_data_type.clone()))
        .collect();
    port_types.append(&mut output_port_types);
    let port_types: HashMap<PortId, AbstractDataType> = port_types.into_iter().collect();
    let constraints = map_constraints_to_ports(graph);
    let result =
        constraint_solver::narrow_abstract_types(&ordered_ports, &port_types, &constraints);
    result.into()
}
