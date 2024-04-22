use crate::{
    constraint_solver_constraints::{Constraint, SameTypesConstraint},
    data_types::{AbstractDataType, CompositeSize, ComputationDomain, ConcreteDataType},
    graph_transfer_types::{from_type_name, Any, WbblWebappNodeType},
    store_errors::WbblWebappStoreError,
    yrs_utils::{get_atomic_bigint, get_atomic_string, get_atomic_u128_from_string, get_map},
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use yrs::{Map, MapRef, ReadTxn};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edge {
    pub id: u128,
    pub input_port: InputPortId,
    pub output_port: OutputPortId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
            NodeType::Junction => vec![Constraint::SameTypes(SameTypesConstraint {
                ports: self.port_ids().iter().map(|p| p.clone()).collect(),
            })],
            NodeType::Frame => vec![],
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
            NodeType::Junction => {
                self.make_input_ports(incoming_edges, &[(AbstractDataType::Any, None, None)])
            }
            NodeType::Frame => vec![],
        }
    }

    pub fn output_ports(&self, outgoing_edges: &Vec<&Edge>) -> Vec<OutputPort> {
        match &self.node_type {
            NodeType::Output => vec![],
            NodeType::Slab => self.make_output_ports(
                outgoing_edges,
                &[AbstractDataType::ConcreteType(
                    ConcreteDataType::SlabMaterial,
                )],
            ),
            NodeType::Preview => vec![],
            NodeType::BinaryOperation(op) => {
                self.make_output_ports(outgoing_edges, &[op.output_port_type()])
            }
            NodeType::BuiltIn(b) => self.make_output_ports(outgoing_edges, &[b.output_port_type()]),
            NodeType::Junction => self.make_output_ports(outgoing_edges, &[AbstractDataType::Any]),
            NodeType::Frame => vec![],
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputPortId {
    pub node_id: u128,
    pub port_index: u8,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutputPortId {
    pub node_id: u128,
    pub port_index: u8,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[serde(into = "String", try_from = "String")]
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
            NodeType::Junction => None,
            NodeType::Frame => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputPort {
    pub id: InputPortId,
    pub incoming_edge: Option<u128>,
    pub abstract_data_type: AbstractDataType,
    pub new_branch_id: Option<u128>,
    pub new_subgraph_id: Option<u128>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct OutputPort {
    pub id: OutputPortId,
    pub outgoing_edges: Vec<u128>,
    pub abstract_data_type: AbstractDataType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
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
            BinaryOperation::ShiftLeft | BinaryOperation::ShiftRight => {
                let fst_input_port =
                    PortId::Input(node.input_ports_ids().first().unwrap().to_owned());
                let fst_output_port =
                    PortId::Output(node.output_ports_ids().first().unwrap().to_owned());
                vec![Constraint::SameTypes(SameTypesConstraint {
                    ports: HashSet::from([fst_input_port, fst_output_port]),
                })]
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeType {
    Output,
    Slab,
    Preview,
    BinaryOperation(BinaryOperation),
    BuiltIn(BuiltIn),
    Junction,
    Frame,
}

impl NodeType {
    pub fn input_port_count(
        &self,
        _incoming_edges: &Vec<&Edge>,
        _outgoing_edges: &Vec<&Edge>,
    ) -> u8 {
        match self {
            NodeType::Output => 1,
            NodeType::Slab => 0,
            NodeType::Preview => 1,
            NodeType::BinaryOperation(_) => 2,
            NodeType::BuiltIn(_) => 0,
            NodeType::Junction => 1,
            Self::Frame => 0,
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
            NodeType::Junction => 1,
            Self::Frame => 0,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct AbstractTypeAssignments {
    pub assignment: HashMap<u128, AbstractDataType>,
}

impl Node {
    fn decode_data<Txn: ReadTxn>(
        txn: &Txn,
        data: &MapRef,
    ) -> Result<HashMap<String, Any>, WbblWebappStoreError> {
        let mut results: HashMap<String, Any> = HashMap::new();
        for (key, value) in data.iter(txn) {
            let value: Any = match value {
                yrs::Value::Any(any) => Ok((&any).into()),
                _ => Err(WbblWebappStoreError::UnexpectedStructure),
            }?;
            results.insert(key.to_string(), value);
        }
        Ok(results)
    }

    fn node_type_from_webapp_node(
        node: WbblWebappNodeType,
        _data: &HashMap<String, Any>,
    ) -> NodeType {
        match node {
            WbblWebappNodeType::Output => NodeType::Output,
            WbblWebappNodeType::Slab => NodeType::Slab,
            WbblWebappNodeType::Preview => NodeType::Preview,
            WbblWebappNodeType::Add => {
                NodeType::BinaryOperation(crate::graph_types::BinaryOperation::Add)
            }
            WbblWebappNodeType::Subtract => {
                NodeType::BinaryOperation(crate::graph_types::BinaryOperation::Subtract)
            }
            WbblWebappNodeType::Multiply => {
                NodeType::BinaryOperation(crate::graph_types::BinaryOperation::Multiply)
            }
            WbblWebappNodeType::Divide => {
                NodeType::BinaryOperation(crate::graph_types::BinaryOperation::Divide)
            }
            WbblWebappNodeType::Modulo => {
                NodeType::BinaryOperation(crate::graph_types::BinaryOperation::Modulo)
            }
            WbblWebappNodeType::Equal => {
                NodeType::BinaryOperation(crate::graph_types::BinaryOperation::Equal)
            }
            WbblWebappNodeType::NotEqual => {
                NodeType::BinaryOperation(crate::graph_types::BinaryOperation::NotEqual)
            }
            WbblWebappNodeType::Less => {
                NodeType::BinaryOperation(crate::graph_types::BinaryOperation::Less)
            }
            WbblWebappNodeType::LessEqual => {
                NodeType::BinaryOperation(crate::graph_types::BinaryOperation::LessEqual)
            }
            WbblWebappNodeType::Greater => {
                NodeType::BinaryOperation(crate::graph_types::BinaryOperation::Greater)
            }
            WbblWebappNodeType::GreaterEqual => {
                NodeType::BinaryOperation(crate::graph_types::BinaryOperation::GreaterEqual)
            }
            WbblWebappNodeType::And => {
                NodeType::BinaryOperation(crate::graph_types::BinaryOperation::And)
            }
            WbblWebappNodeType::Or => {
                NodeType::BinaryOperation(crate::graph_types::BinaryOperation::Or)
            }
            WbblWebappNodeType::ShiftLeft => {
                NodeType::BinaryOperation(crate::graph_types::BinaryOperation::ShiftLeft)
            }
            WbblWebappNodeType::ShiftRight => {
                NodeType::BinaryOperation(crate::graph_types::BinaryOperation::ShiftRight)
            }
            WbblWebappNodeType::WorldPosition => {
                NodeType::BuiltIn(crate::graph_types::BuiltIn::WorldPosition)
            }
            WbblWebappNodeType::ClipPosition => {
                NodeType::BuiltIn(crate::graph_types::BuiltIn::ClipPosition)
            }
            WbblWebappNodeType::WorldNormal => {
                NodeType::BuiltIn(crate::graph_types::BuiltIn::WorldNormal)
            }
            WbblWebappNodeType::WorldBitangent => {
                NodeType::BuiltIn(crate::graph_types::BuiltIn::WorldBitangent)
            }
            WbblWebappNodeType::WorldTangent => {
                NodeType::BuiltIn(crate::graph_types::BuiltIn::WorldTangent)
            }
            WbblWebappNodeType::TexCoord => {
                NodeType::BuiltIn(crate::graph_types::BuiltIn::TextureCoordinate)
            }
            WbblWebappNodeType::TexCoord2 => {
                NodeType::BuiltIn(crate::graph_types::BuiltIn::TextureCoordinate2)
            }
            WbblWebappNodeType::Junction => NodeType::Junction,
        }
    }

    pub fn insert_new<Txn: ReadTxn>(
        txn: &Txn,
        node: &MapRef,
        graph: &mut Graph,
    ) -> Result<(), WbblWebappStoreError> {
        let id = get_atomic_u128_from_string("id", txn, node)?;
        let data = get_map("data", txn, node)?;
        let data = Node::decode_data(txn, &data)?;
        let type_name: String = get_atomic_string("type", txn, node)?;
        let node_transfer_type = from_type_name(&type_name);
        if node_transfer_type.is_none() {
            return Err(WbblWebappStoreError::UnknownNodeType);
        }
        let node_transfer_type = node_transfer_type.unwrap();
        let node_type = Self::node_type_from_webapp_node(node_transfer_type, &data);
        let input_port_count = node_type.input_port_count(&vec![], &vec![]);
        let output_port_count = node_type.output_port_count(&vec![], &vec![]);
        let node = Node {
            id,
            input_port_count,
            output_port_count,
            node_type,
        };

        for port in node.input_ports(&vec![]) {
            graph.input_ports.insert(port.id.clone(), port);
        }

        for port in node.output_ports(&vec![]) {
            graph.output_ports.insert(port.id.clone(), port);
        }
        graph.nodes.insert(node.id, node);

        Ok(())
    }

    pub fn update_existing<Txn: ReadTxn>(
        txn: &Txn,
        node: &MapRef,
        graph: &mut Graph,
    ) -> Result<(), WbblWebappStoreError> {
        let id = get_atomic_u128_from_string("id", txn, node)?;
        let data = get_map("data", txn, node)?;
        let data = Node::decode_data(txn, &data)?;
        let type_name: String = get_atomic_string("type", txn, node)?;
        let node_transfer_type = from_type_name(&type_name);
        if node_transfer_type.is_none() {
            return Err(WbblWebappStoreError::UnknownNodeType);
        }
        let node_transfer_type = node_transfer_type.unwrap();
        if let Some(prev_node) = graph.nodes.get(&id) {
            let prev_input_port_count = prev_node.input_port_count;
            let prev_output_port_count = prev_node.input_port_count;
            let mut incoming_edges: Vec<&Edge> = Vec::new();
            let mut outgoing_edges: Vec<&Edge> = Vec::new();

            for i in 0..prev_input_port_count {
                if let Some(input_port) = graph.input_ports.get(&InputPortId {
                    node_id: prev_node.id,
                    port_index: i,
                }) {
                    if let Some(Some(incoming_edge)) =
                        input_port.incoming_edge.map(|x| graph.edges.get(&x))
                    {
                        incoming_edges.push(incoming_edge);
                    }
                }
            }
            for i in 0..prev_output_port_count {
                if let Some(output_port) = graph.output_ports.get(&OutputPortId {
                    node_id: prev_node.id,
                    port_index: i,
                }) {
                    for edge_id in output_port.outgoing_edges.iter() {
                        if let Some(outgoing_edge) = graph.edges.get(&edge_id) {
                            outgoing_edges.push(outgoing_edge);
                        }
                    }
                }
            }

            let node_type = Self::node_type_from_webapp_node(node_transfer_type, &data);

            let new_input_port_count = node_type.input_port_count(&incoming_edges, &outgoing_edges);
            let new_output_port_count =
                node_type.output_port_count(&incoming_edges, &outgoing_edges);

            if new_input_port_count < prev_input_port_count {
                for i in new_input_port_count..prev_input_port_count {
                    let port_id = InputPortId {
                        node_id: id,
                        port_index: i,
                    };
                    if let Some(port) = graph.input_ports.remove(&port_id) {
                        if let Some(edge_id) = port.incoming_edge {
                            graph.edges.remove(&edge_id);
                        }
                    }
                }
            }

            if new_output_port_count < prev_output_port_count {
                for i in new_output_port_count..prev_output_port_count {
                    let port_id = OutputPortId {
                        node_id: id,
                        port_index: i,
                    };
                    if let Some(port) = graph.output_ports.remove(&port_id) {
                        for edge_id in port.outgoing_edges.iter() {
                            graph.edges.remove(&edge_id);
                        }
                    }
                }
            }

            let node = Node {
                id,
                input_port_count: new_input_port_count,
                output_port_count: new_output_port_count,
                node_type,
            };
            for mut port in node.input_ports(&vec![]) {
                if let Some(prev_port) = graph.input_ports.remove(&port.id) {
                    if prev_port.abstract_data_type == port.abstract_data_type {
                        // If abstract data type is same, keep edge
                        port.incoming_edge = prev_port.incoming_edge;
                    }
                    graph.input_ports.insert(port.id.clone(), port);
                } else {
                    graph.input_ports.insert(port.id.clone(), port);
                }
            }
            for mut port in node.output_ports(&vec![]) {
                if let Some(prev_port) = graph.output_ports.remove(&port.id) {
                    if prev_port.abstract_data_type == port.abstract_data_type {
                        // If abstract data type is same, keep edge
                        port.outgoing_edges = prev_port.outgoing_edges;
                    }
                    graph.output_ports.insert(port.id.clone(), port);
                } else {
                    graph.output_ports.insert(port.id.clone(), port);
                }
            }
            graph.nodes.insert(node.id.clone(), node);
            Ok(())
        } else {
            Err(WbblWebappStoreError::NotFound)
        }
    }
}

impl Edge {
    pub fn insert_new<Txn: ReadTxn>(
        txn: &Txn,
        edge: &MapRef,
        graph: &mut Graph,
    ) -> Result<(), WbblWebappStoreError> {
        let id = get_atomic_u128_from_string("id", txn, edge)?;
        let source = get_atomic_u128_from_string("source", txn, edge)?;
        let target = get_atomic_u128_from_string("target", txn, edge)?;
        let source_handle = get_atomic_bigint("source_handle", txn, edge)? as u8;
        let target_handle = get_atomic_bigint("target_handle", txn, edge)? as u8;
        let edge = Edge {
            id,
            input_port: InputPortId {
                node_id: target.clone(),
                port_index: target_handle,
            },
            output_port: OutputPortId {
                node_id: source.clone(),
                port_index: source_handle,
            },
        };
        graph.edges.insert(edge.id.clone(), edge.clone());
        if let Some(input_port) = graph.input_ports.get_mut(&edge.input_port) {
            input_port.incoming_edge = Some(edge.id);

            if let Some(input_node) = graph.nodes.get_mut(&target) {
                let prev_input_port_count = input_node.input_port_count;
                let prev_output_port_count = input_node.output_port_count;
                let mut incoming_edges: Vec<&Edge> = Vec::new();
                let mut outgoing_edges: Vec<&Edge> = Vec::new();

                for i in 0..prev_input_port_count {
                    if let Some(input_port) = graph.input_ports.get(&InputPortId {
                        node_id: input_node.id,
                        port_index: i,
                    }) {
                        if let Some(Some(incoming_edge)) =
                            input_port.incoming_edge.map(|x| graph.edges.get(&x))
                        {
                            incoming_edges.push(incoming_edge);
                        }
                    }
                }
                for i in 0..prev_output_port_count {
                    if let Some(output_port) = graph.output_ports.get(&OutputPortId {
                        node_id: input_node.id,
                        port_index: i,
                    }) {
                        for edge_id in output_port.outgoing_edges.iter() {
                            if let Some(outgoing_edge) = graph.edges.get(&edge_id) {
                                outgoing_edges.push(outgoing_edge);
                            }
                        }
                    }
                }
                input_node.input_port_count = input_node
                    .node_type
                    .input_port_count(&incoming_edges, &outgoing_edges);
                input_node.output_port_count = input_node
                    .node_type
                    .output_port_count(&incoming_edges, &outgoing_edges);
            }
        }
        if let Some(output_port) = graph.output_ports.get_mut(&edge.output_port) {
            output_port.outgoing_edges.push(edge.id);
            if let Some(output_node) = graph.nodes.get_mut(&source) {
                let prev_input_port_count = output_node.input_port_count;
                let prev_output_port_count = output_node.output_port_count;
                let mut incoming_edges: Vec<&Edge> = Vec::new();
                let mut outgoing_edges: Vec<&Edge> = Vec::new();

                for i in 0..prev_input_port_count {
                    if let Some(input_port) = graph.input_ports.get(&InputPortId {
                        node_id: output_node.id,
                        port_index: i,
                    }) {
                        if let Some(Some(incoming_edge)) =
                            input_port.incoming_edge.map(|x| graph.edges.get(&x))
                        {
                            incoming_edges.push(incoming_edge);
                        }
                    }
                }
                for i in 0..prev_output_port_count {
                    if let Some(output_port) = graph.output_ports.get(&OutputPortId {
                        node_id: output_node.id,
                        port_index: i,
                    }) {
                        for edge_id in output_port.outgoing_edges.iter() {
                            if let Some(outgoing_edge) = graph.edges.get(&edge_id) {
                                outgoing_edges.push(outgoing_edge);
                            }
                        }
                    }
                }
                output_node.input_port_count = output_node
                    .node_type
                    .input_port_count(&incoming_edges, &outgoing_edges);
                output_node.output_port_count = output_node
                    .node_type
                    .output_port_count(&incoming_edges, &outgoing_edges);
            }
        }
        Ok(())
    }
}
