use crate::data_types::{AbstractDataType, ComputationDomain};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Clone)]
pub struct Edge {
    pub id: u128,
    pub input_port: InputPortId,
    pub output_port: OutputPortId,
}

#[derive(Clone)]
pub struct Node {
    pub id: u128,
    pub node_type: NodeType,
    pub input_port_count: u8,
    pub output_port_count: u8,
}

impl Node {
    pub fn ports(&self) -> Vec<PortId> {
        self.input_ports()
            .into_iter()
            .map(|p| PortId::Input(p))
            .chain(self.output_ports().into_iter().map(|p| PortId::Output(p)))
            .collect()
    }

    pub fn input_ports(&self) -> Vec<InputPortId> {
        (0..self.input_port_count)
            .map(|i| InputPortId {
                node_id: self.id,
                port_index: i,
            })
            .collect()
    }

    pub fn output_ports(&self) -> Vec<OutputPortId> {
        (0..self.output_port_count)
            .map(|i| OutputPortId {
                node_id: self.id,
                port_index: i,
            })
            .collect()
    }
}

#[derive(Clone, Hash, PartialEq, PartialOrd, Ord, Eq)]
pub struct InputPortId {
    pub node_id: u128,
    pub port_index: u8,
}

#[derive(Clone, Hash, PartialEq, PartialOrd, Ord, Eq)]
pub struct OutputPortId {
    pub node_id: u128,
    pub port_index: u8,
}

#[derive(Clone, Hash, PartialEq, PartialOrd, Ord, Eq)]
pub enum PortId {
    Output(OutputPortId),
    Input(InputPortId),
}

impl Node {
    pub fn get_computation_domain(&self) -> Option<HashSet<ComputationDomain>> {
        match self.node_type {
            NodeType::Output => None,
        }
    }
}

#[derive(Clone)]
pub struct InputPort {
    pub id: InputPortId,
    pub node: u128,
    pub incoming_edge: Option<u128>,
    pub abstract_data_type: AbstractDataType,
    pub new_branch_id: Option<u128>,
    pub new_subgraph_id: Option<u128>,
}

#[derive(Clone)]
pub struct OutputPort {
    pub id: OutputPortId,
    pub node: u128,
    pub outgoing_edges: Vec<u128>,
    pub abstract_data_type: AbstractDataType,
}

#[derive(Clone)]
pub struct Graph {
    pub id: u128,
    pub nodes: HashMap<u128, Node>,
    pub edges: HashMap<u128, Edge>,
    pub input_ports: HashMap<InputPortId, InputPort>,
    pub output_ports: HashMap<OutputPortId, OutputPort>,
}

#[derive(Clone)]
pub struct Subgraph {
    pub id: u128,
    pub nodes: Vec<u128>,
}

#[derive(Clone)]
pub struct MultiGraph {
    pub graph: Graph,
    pub subgraphs: HashMap<u128, Subgraph>,
    pub subgraph_ordering: Vec<u128>,
    pub dependencies: HashMap<u128, HashSet<u128>>,
}

#[derive(Clone)]
pub struct BranchedSubgraph {
    pub id: u128,
    pub nodes: Vec<u128>,
    pub branches: HashMap<u128, Vec<u128>>,
}

#[derive(Clone)]
pub struct BranchedMultiGraph {
    pub graph: Graph,
    pub subgraphs: HashMap<u128, BranchedSubgraph>,
    pub subgraph_ordering: Vec<u128>,
    pub dependencies: HashMap<u128, HashSet<u128>>,
}

#[derive(Clone)]
pub enum NodeType {
    Output,
}

#[derive(Serialize, Deserialize)]
pub struct AbstractTypeAssignments {
    pub assignment: HashMap<u128, AbstractDataType>,
}
