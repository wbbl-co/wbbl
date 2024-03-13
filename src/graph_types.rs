use crate::data_types::{AbstractDataType, ComputationDomain};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Serialize, Deserialize, Clone)]
pub struct Edge {
    pub id: u128,
    pub input_port: u128,
    pub output_port: u128,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Node {
    pub id: u128,
    pub node_type: NodeType,
    pub input_ports: Vec<u128>,
    pub output_ports: Vec<u128>,
}

impl Node {
    pub fn get_computation_domain(&self) -> Option<HashSet<ComputationDomain>> {
        match self.node_type {
            NodeType::Output => None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct InputPort {
    pub id: u128,
    pub node: u128,
    pub incoming_edge: Option<u128>,
    pub abstract_data_type: AbstractDataType,
    pub new_branch_id: Option<u128>,
    pub new_subgraph_id: Option<u128>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct OutputPort {
    pub id: u128,
    pub node: u128,
    pub outgoing_edges: Vec<u128>,
    pub abstract_data_type: AbstractDataType,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Graph {
    pub id: u128,
    pub nodes: HashMap<u128, Node>,
    pub edges: HashMap<u128, Edge>,
    pub input_ports: HashMap<u128, InputPort>,
    pub output_ports: HashMap<u128, OutputPort>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Subgraph {
    pub id: u128,
    pub nodes: Vec<u128>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MultiGraph {
    pub graph: Graph,
    pub subgraphs: HashMap<u128, Subgraph>,
    pub subgraph_ordering: Vec<u128>,
    pub dependencies: HashMap<u128, HashSet<u128>>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BranchedSubgraph {
    pub id: u128,
    pub nodes: Vec<u128>,
    pub branches: HashMap<u128, Vec<u128>>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BranchedMultiGraph {
    pub graph: Graph,
    pub subgraphs: HashMap<u128, BranchedSubgraph>,
    pub subgraph_ordering: Vec<u128>,
    pub dependencies: HashMap<u128, HashSet<u128>>,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum NodeType {
    Output,
}

#[derive(Serialize, Deserialize)]
pub struct AbstractTypeAssignments {
    pub assignment: HashMap<u128, AbstractDataType>,
}
