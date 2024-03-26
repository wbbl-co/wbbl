use crate::{
    constraint_solver_constraints::{Constraint, SameTypesConstraint},
    data_types::{AbstractDataType, CompositeSize, ComputationDomain, ConcreteDataType},
};

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
    pub fn port_ids(&self) -> Vec<PortId> {
        self.input_ports_ids()
            .into_iter()
            .map(|p| PortId::Input(p))
            .chain(
                self.output_ports_ids()
                    .into_iter()
                    .map(|p| PortId::Output(p)),
            )
            .collect()
    }

    pub fn input_ports_ids(&self) -> Vec<InputPortId> {
        (0..self.input_port_count)
            .map(|i| InputPortId {
                node_id: self.id,
                port_index: i,
            })
            .collect()
    }

    pub fn output_ports_ids(&self) -> Vec<OutputPortId> {
        (0..self.output_port_count)
            .map(|i| OutputPortId {
                node_id: self.id,
                port_index: i,
            })
            .collect()
    }

    fn make_input_ports(
        &self,
        edges: &Vec<&Edge>,
        ports: &[(AbstractDataType, Option<u128>, Option<u128>)],
    ) -> Vec<InputPort> {
        ports
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let port_id = InputPortId {
                    node_id: self.id,
                    port_index: i as u8,
                };
                InputPort {
                    incoming_edge: edges.iter().find(|e| e.input_port == port_id).map(|e| e.id),
                    id: port_id,
                    abstract_data_type: p.0,
                    new_branch_id: p.1,
                    new_subgraph_id: p.2,
                }
            })
            .collect()
    }

    fn make_output_ports(&self, edges: &Vec<&Edge>, ports: &[AbstractDataType]) -> Vec<OutputPort> {
        ports
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let port_id = OutputPortId {
                    node_id: self.id,
                    port_index: i as u8,
                };
                OutputPort {
                    outgoing_edges: edges
                        .iter()
                        .filter(|e| e.output_port == port_id)
                        .map(|e| e.id)
                        .collect(),
                    id: port_id,
                    abstract_data_type: p.clone(),
                }
            })
            .collect()
    }

    pub fn constraints(&self) -> Vec<Constraint> {
        match &self.node_type {
            NodeType::Output => vec![],
            NodeType::Slab => vec![],
            NodeType::Preview => vec![],
            NodeType::BuiltIn(_) => vec![],
            NodeType::BinaryOperation(op) => op.constraints(self),
        }
    }

    pub fn input_ports(&self, incoming_edges: &Vec<&Edge>) -> Vec<InputPort> {
        match &self.node_type {
            NodeType::Output => self.make_input_ports(
                incoming_edges,
                &[(AbstractDataType::AnyMaterial, None, None)],
            ),
            // TODO
            NodeType::Slab => vec![],
            NodeType::Preview => {
                self.make_input_ports(incoming_edges, &[(AbstractDataType::Any, None, None)])
            }
            NodeType::BinaryOperation(op) => self.make_input_ports(
                incoming_edges,
                &op.input_port_types(self)
                    .iter()
                    .map(|t| (t.clone(), None, None))
                    .collect::<Vec<(AbstractDataType, Option<u128>, Option<u128>)>>(),
            ),
            NodeType::BuiltIn(_) => vec![],
        }
    }

    pub fn output_ports(&self, outgoing_edges: &Vec<&Edge>) -> Vec<OutputPort> {
        match &self.node_type {
            NodeType::Output => vec![],
            NodeType::Slab => vec![],
            NodeType::Preview => vec![],
            NodeType::BinaryOperation(op) => {
                self.make_output_ports(outgoing_edges, &[op.output_port_type()])
            }
            NodeType::BuiltIn(b) => self.make_output_ports(outgoing_edges, &[b.output_port_type()]),
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, PartialOrd, Ord, Eq, Serialize, Deserialize)]
pub struct InputPortId {
    pub node_id: u128,
    pub port_index: u8,
}

#[derive(Debug, Clone, Hash, PartialEq, PartialOrd, Ord, Eq, Serialize, Deserialize)]
pub struct OutputPortId {
    pub node_id: u128,
    pub port_index: u8,
}

#[derive(Debug, Clone, Hash, PartialEq, PartialOrd, Ord, Eq, Serialize, Deserialize)]
pub enum PortId {
    Output(OutputPortId),
    Input(InputPortId),
}

impl Node {
    pub fn get_computation_domain(&self) -> Option<HashSet<ComputationDomain>> {
        match self.node_type {
            NodeType::Output => Some(HashSet::from([
                ComputationDomain::TimeVarying,
                ComputationDomain::ModelDependant,
                ComputationDomain::TransformDependant,
            ])),
            NodeType::Slab => Some(HashSet::from([
                ComputationDomain::TimeVarying,
                ComputationDomain::ModelDependant,
                ComputationDomain::TransformDependant,
            ])),
            NodeType::Preview => Some(HashSet::from([
                ComputationDomain::TimeVarying,
                ComputationDomain::ModelDependant,
                ComputationDomain::TransformDependant,
            ])),
            NodeType::BinaryOperation(_) => None,
            NodeType::BuiltIn(_) => Some(HashSet::from([
                ComputationDomain::ModelDependant,
                ComputationDomain::TransformDependant,
            ])),
        }
    }
}

#[derive(Clone)]
pub struct InputPort {
    pub id: InputPortId,
    pub incoming_edge: Option<u128>,
    pub abstract_data_type: AbstractDataType,
    pub new_branch_id: Option<u128>,
    pub new_subgraph_id: Option<u128>,
}

#[derive(Clone)]
pub struct OutputPort {
    pub id: OutputPortId,
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
    pub constraints: Vec<Constraint>,
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
pub enum BinaryOperation {
    Add,
    Subtract,
    Multiply,
    Divide,
    /// Equivalent of the WGSL's `%` operator or SPIR-V's `OpFRem`
    Modulo,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    And,
    Or,
    ShiftLeft,
    /// Right shift carries the sign of signed integers only.
    ShiftRight,
}

impl BinaryOperation {
    fn uniform_input_types(node: &Node, t: AbstractDataType) -> Vec<AbstractDataType> {
        node.input_ports_ids().iter().map(|_| t).collect()
    }
    pub fn input_port_types(&self, node: &Node) -> Vec<AbstractDataType> {
        match self {
            BinaryOperation::Add
            | BinaryOperation::Subtract
            | BinaryOperation::Multiply
            | BinaryOperation::Divide
            | BinaryOperation::Modulo
            | BinaryOperation::Equal
            | BinaryOperation::NotEqual
            | BinaryOperation::Less
            | BinaryOperation::LessEqual
            | BinaryOperation::Greater
            | BinaryOperation::GreaterEqual => {
                Self::uniform_input_types(node, AbstractDataType::AnyNumber)
            }
            BinaryOperation::And | BinaryOperation::Or => {
                Self::uniform_input_types(node, AbstractDataType::AnyVectorOrScalar)
            }
            BinaryOperation::ShiftLeft | BinaryOperation::ShiftRight => {
                vec![
                    AbstractDataType::AnyNumber,
                    AbstractDataType::ConcreteType(ConcreteDataType::Int),
                ]
            }
        }
    }

    pub fn output_port_type(&self) -> AbstractDataType {
        match self {
            BinaryOperation::Add => AbstractDataType::AnyNumber,
            BinaryOperation::Subtract => AbstractDataType::AnyNumber,
            BinaryOperation::Multiply => AbstractDataType::AnyNumber,
            BinaryOperation::Divide => AbstractDataType::AnyNumber,
            BinaryOperation::Modulo => AbstractDataType::AnyNumber,
            BinaryOperation::Equal => AbstractDataType::ConcreteType(ConcreteDataType::Bool),
            BinaryOperation::NotEqual => AbstractDataType::ConcreteType(ConcreteDataType::Bool),
            BinaryOperation::Less => AbstractDataType::ConcreteType(ConcreteDataType::Bool),
            BinaryOperation::LessEqual => AbstractDataType::ConcreteType(ConcreteDataType::Bool),
            BinaryOperation::Greater => AbstractDataType::ConcreteType(ConcreteDataType::Bool),
            BinaryOperation::GreaterEqual => AbstractDataType::ConcreteType(ConcreteDataType::Bool),
            BinaryOperation::And => AbstractDataType::AnyVectorOrScalar,
            BinaryOperation::Or => AbstractDataType::AnyVectorOrScalar,
            BinaryOperation::ShiftLeft => AbstractDataType::AnyNumber,
            BinaryOperation::ShiftRight => AbstractDataType::AnyNumber,
        }
    }

    pub fn constraints(&self, node: &Node) -> Vec<Constraint> {
        match self {
            BinaryOperation::Add
            | BinaryOperation::Subtract
            | BinaryOperation::Multiply
            | BinaryOperation::Divide
            | BinaryOperation::Modulo => {
                let ports: HashSet<PortId> = node.port_ids().iter().map(|p| p.clone()).collect();
                vec![Constraint::SameTypes(SameTypesConstraint { ports })]
            }
            BinaryOperation::Equal
            | BinaryOperation::NotEqual
            | BinaryOperation::Less
            | BinaryOperation::LessEqual
            | BinaryOperation::Greater
            | BinaryOperation::GreaterEqual => {
                let ports: HashSet<PortId> = node
                    .input_ports_ids()
                    .iter()
                    .map(|p| PortId::Input(p.clone()))
                    .collect();
                vec![Constraint::SameTypes(SameTypesConstraint { ports })]
            }
            BinaryOperation::And | BinaryOperation::Or => {
                let ports: HashSet<PortId> = node.port_ids().iter().map(|p| p.clone()).collect();
                vec![Constraint::SameTypes(SameTypesConstraint { ports })]
            }
            BinaryOperation::ShiftLeft | BinaryOperation::ShiftRight => vec![],
        }
    }
}

#[derive(Clone)]
pub enum BuiltIn {
    WorldPosition,
    ClipPosition,
    WorldNormal,
    WorldBitangent,
    WorldTangent,
    TextureCoordinate,
    TextureCoordinate2,
}

impl BuiltIn {
    pub fn output_port_type(&self) -> AbstractDataType {
        match self {
            BuiltIn::WorldPosition => {
                AbstractDataType::ConcreteType(ConcreteDataType::Float(CompositeSize::S3))
            }
            BuiltIn::ClipPosition => {
                AbstractDataType::ConcreteType(ConcreteDataType::Float(CompositeSize::S4))
            }
            BuiltIn::WorldNormal => {
                AbstractDataType::ConcreteType(ConcreteDataType::Float(CompositeSize::S3))
            }
            BuiltIn::WorldBitangent => {
                AbstractDataType::ConcreteType(ConcreteDataType::Float(CompositeSize::S3))
            }
            BuiltIn::WorldTangent => {
                AbstractDataType::ConcreteType(ConcreteDataType::Float(CompositeSize::S3))
            }
            BuiltIn::TextureCoordinate => {
                AbstractDataType::ConcreteType(ConcreteDataType::Float(CompositeSize::S2))
            }
            BuiltIn::TextureCoordinate2 => {
                AbstractDataType::ConcreteType(ConcreteDataType::Float(CompositeSize::S2))
            }
        }
    }
}

#[derive(Clone)]
pub enum NodeType {
    Output,
    Slab,
    Preview,
    BinaryOperation(BinaryOperation),
    BuiltIn(BuiltIn),
}

impl NodeType {
    pub fn input_port_count(
        &self,
        _incoming_edges: &Vec<&Edge>,
        _outgoing_edges: &Vec<&Edge>,
    ) -> u8 {
        match self {
            NodeType::Output => 1,
            NodeType::Slab => 1,
            NodeType::Preview => 1,
            NodeType::BinaryOperation(_) => 2,
            NodeType::BuiltIn(_) => 0,
        }
    }

    pub fn output_port_count(
        &self,
        _incoming_edges: &Vec<&Edge>,
        _outgoing_edges: &Vec<&Edge>,
    ) -> u8 {
        match self {
            NodeType::Output => 0,
            NodeType::Slab => 1,
            NodeType::Preview => 0,
            NodeType::BinaryOperation(_) => 1,
            NodeType::BuiltIn(_) => 1,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct AbstractTypeAssignments {
    pub assignment: HashMap<u128, AbstractDataType>,
}
