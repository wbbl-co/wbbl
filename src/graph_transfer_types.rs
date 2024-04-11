use glam::Vec2;
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    str::FromStr,
    sync::Arc,
};
use wasm_bindgen::prelude::*;

use crate::{
    constraint_solver_constraints::Constraint,
    data_types::AbstractDataType,
    graph_types::{
        Edge, Graph, InputPort, InputPortId, Node, NodeType, OutputPort, OutputPortId, PortId,
    },
};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct WbbleComputedNodeSize {
    pub width: Option<f64>,
    pub height: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Any {
    Null,
    Undefined,
    Bool(bool),
    Number(f64),
    BigInt(i64),
    String(Arc<str>),
    Buffer(Arc<[u8]>),
    Array(Arc<[Any]>),
    Map(Arc<HashMap<String, Any>>),
}

impl Into<String> for PortId {
    fn into(self) -> String {
        match self {
            PortId::Output(OutputPortId {
                node_id,
                port_index,
            }) => format!(
                "{}#s#{}",
                uuid::Uuid::from_u128(node_id).to_string(),
                port_index
            ),
            PortId::Input(InputPortId {
                node_id,
                port_index,
            }) => format!(
                "{}#t#{}",
                uuid::Uuid::from_u128(node_id).to_string(),
                port_index
            ),
        }
    }
}

#[derive(Debug)]
pub enum TryFromStringToPortIdError {
    MalformedId,
}

impl Display for TryFromStringToPortIdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Malformed Id")
    }
}

impl TryFrom<String> for PortId {
    type Error = TryFromStringToPortIdError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let res: &Vec<&str> = &value.split("#").collect();
        if res.len() != 3 {
            return Err(TryFromStringToPortIdError::MalformedId);
        }
        let node_id =
            uuid::Uuid::from_str(res[0]).map_err(|_| TryFromStringToPortIdError::MalformedId)?;
        let index: u8 = str::parse(res[2]).map_err(|_| TryFromStringToPortIdError::MalformedId)?;
        match res[1] {
            "s" => Ok(PortId::Output(OutputPortId {
                node_id: node_id.as_u128(),
                port_index: index,
            })),
            "t" => Ok(PortId::Input(InputPortId {
                node_id: node_id.as_u128(),
                port_index: index,
            })),
            _ => Err(TryFromStringToPortIdError::MalformedId),
        }
    }
}

impl Any {
    pub fn to_yrs(&self) -> yrs::Any {
        match self {
            Any::Null => yrs::Any::Null,
            Any::Undefined => yrs::Any::Undefined,
            Any::Bool(b) => yrs::Any::Bool(*b),
            Any::Number(n) => yrs::Any::Number(*n),
            Any::BigInt(b) => yrs::Any::BigInt(*b),
            Any::String(str) => yrs::Any::String(str.clone()),
            Any::Buffer(b) => yrs::Any::Buffer(b.clone()),
            Any::Array(arr) => yrs::Any::Array(arr.iter().map(|a| Self::to_yrs(a)).collect()),
            Any::Map(map) => yrs::Any::Map(
                map.iter()
                    .map(|(k, v)| (k.to_owned(), Self::to_yrs(v)))
                    .collect::<HashMap<String, yrs::Any>>()
                    .into(),
            ),
        }
    }
}

impl From<&yrs::Any> for Any {
    fn from(value: &yrs::Any) -> Self {
        match value {
            yrs::Any::Null => Any::Null,
            yrs::Any::Undefined => Any::Undefined,
            yrs::Any::Bool(b) => Any::Bool(*b),
            yrs::Any::Number(n) => Any::Number(*n),
            yrs::Any::BigInt(b) => Any::BigInt(*b),
            yrs::Any::String(str) => Any::String(str.clone()),
            yrs::Any::Buffer(b) => Any::Buffer(b.clone()),
            yrs::Any::Array(arr) => Any::Array(arr.iter().map(|a| Self::from(a)).collect()),
            yrs::Any::Map(map) => Any::Map(
                map.iter()
                    .map(|(k, v)| (k.to_owned(), Self::from(v)))
                    .collect::<HashMap<String, Any>>()
                    .into(),
            ),
        }
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum WbblWebappNodeType {
    Output,
    Slab,
    Preview,

    Add,
    Subtract,
    Multiply,
    Divide,

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
    ShiftRight,

    WorldPosition,
    ClipPosition,
    WorldNormal,
    WorldBitangent,
    WorldTangent,
    TexCoord,
    TexCoord2,
}

pub fn get_type_name(node_type: WbblWebappNodeType) -> String {
    match node_type {
        WbblWebappNodeType::Output => "output".to_owned(),
        WbblWebappNodeType::Slab => "slab".to_owned(),
        WbblWebappNodeType::Preview => "preview".to_owned(),
        WbblWebappNodeType::Add => "add".to_owned(),
        WbblWebappNodeType::Subtract => "subtract".to_owned(),
        WbblWebappNodeType::Multiply => "multiply".to_owned(),
        WbblWebappNodeType::Divide => "divide".to_owned(),
        WbblWebappNodeType::Modulo => "modulo".to_owned(),
        WbblWebappNodeType::Equal => "==".to_owned(),
        WbblWebappNodeType::NotEqual => "!=".to_owned(),
        WbblWebappNodeType::Less => "<".to_owned(),
        WbblWebappNodeType::LessEqual => "<=".to_owned(),
        WbblWebappNodeType::Greater => ">".to_owned(),
        WbblWebappNodeType::GreaterEqual => ">=".to_owned(),
        WbblWebappNodeType::And => "and".to_owned(),
        WbblWebappNodeType::ShiftLeft => "<<".to_owned(),
        WbblWebappNodeType::ShiftRight => ">>".to_owned(),
        WbblWebappNodeType::Or => "or".to_owned(),
        WbblWebappNodeType::WorldPosition => "position".to_owned(),
        WbblWebappNodeType::ClipPosition => "clip_pos".to_owned(),
        WbblWebappNodeType::WorldNormal => "normal".to_owned(),
        WbblWebappNodeType::WorldBitangent => "bitangent".to_owned(),
        WbblWebappNodeType::WorldTangent => "tangent".to_owned(),
        WbblWebappNodeType::TexCoord => "tex_coord".to_owned(),
        WbblWebappNodeType::TexCoord2 => "tex_coord_2".to_owned(),
    }
}

#[wasm_bindgen]
pub fn from_type_name(type_name: &str) -> Option<WbblWebappNodeType> {
    match type_name {
        "output" => Some(WbblWebappNodeType::Output),
        "slab" => Some(WbblWebappNodeType::Slab),
        "preview" => Some(WbblWebappNodeType::Preview),
        "add" => Some(WbblWebappNodeType::Add),
        "subtract" => Some(WbblWebappNodeType::Subtract),
        "multiply" => Some(WbblWebappNodeType::Multiply),
        "divide" => Some(WbblWebappNodeType::Divide),
        "modulo" => Some(WbblWebappNodeType::Modulo),
        "==" => Some(WbblWebappNodeType::Equal),
        "!=" => Some(WbblWebappNodeType::NotEqual),
        "<" => Some(WbblWebappNodeType::Less),
        "<=" => Some(WbblWebappNodeType::LessEqual),
        ">" => Some(WbblWebappNodeType::Greater),
        ">=" => Some(WbblWebappNodeType::GreaterEqual),
        "and" => Some(WbblWebappNodeType::And),
        "<<" => Some(WbblWebappNodeType::ShiftLeft),
        ">>" => Some(WbblWebappNodeType::ShiftRight),
        "or" => Some(WbblWebappNodeType::Or),
        "position" => Some(WbblWebappNodeType::WorldPosition),
        "clip_pos" => Some(WbblWebappNodeType::ClipPosition),
        "normal" => Some(WbblWebappNodeType::WorldNormal),
        "bitangent" => Some(WbblWebappNodeType::WorldBitangent),
        "tangent" => Some(WbblWebappNodeType::WorldTangent),
        "tex_coord" => Some(WbblWebappNodeType::TexCoord),
        "tex_coord_2" => Some(WbblWebappNodeType::TexCoord2),
        _ => None,
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct WbblePosition {
    pub x: f64,
    pub y: f64,
}

fn type_to_string<S>(node_type: &WbblWebappNodeType, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&get_type_name(*node_type))
}

fn string_to_type<'de, D>(deserializer: D) -> Result<WbblWebappNodeType, D::Error>
where
    D: Deserializer<'de>,
{
    let buf = String::deserialize(deserializer)?;
    match from_type_name(&buf) {
        Some(t) => Ok(t),
        None => Err(Error::custom("Unknown type name")),
    }
}

fn uuid_to_string<S>(id: &u128, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&uuid::Uuid::from_u128(*id).to_string())
}

fn string_to_uuid<'de, D>(deserializer: D) -> Result<u128, D::Error>
where
    D: Deserializer<'de>,
{
    let buf = String::deserialize(deserializer)?;
    match uuid::Uuid::from_str(&buf) {
        Ok(t) => Ok(t.as_u128()),
        Err(_) => Err(Error::custom("Malformed Id")),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WbblWebappNode {
    #[serde(serialize_with = "uuid_to_string", deserialize_with = "string_to_uuid")]
    pub id: u128,
    pub position: WbblePosition,
    #[serde(
        rename = "type",
        serialize_with = "type_to_string",
        deserialize_with = "string_to_type"
    )]
    pub node_type: WbblWebappNodeType,
    pub data: HashMap<String, Any>,
    pub measured: Option<WbbleComputedNodeSize>,
    pub dragging: bool,
    pub resizing: bool,
    pub selected: bool,
    pub selectable: bool,
    pub connectable: bool,
    pub deletable: bool,
}

fn target_handle_to_string<S>(number: &i64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&format!("t#{}", number).to_owned())
}

fn source_handle_to_string<S>(number: &i64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&format!("s#{}", number).to_owned())
}

fn string_to_target_handle<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    let handle = String::deserialize(deserializer)?.replace("t#", "");

    handle
        .parse()
        .map_err(|_| Error::custom("Malformed target handle id"))
}

fn string_to_source_handle<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    let handle = String::deserialize(deserializer)?.replace("s#", "");

    handle
        .parse()
        .map_err(|_| Error::custom("Malformed target handle id"))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WbblWebappEdge {
    #[serde(serialize_with = "uuid_to_string", deserialize_with = "string_to_uuid")]
    pub id: u128,
    #[serde(serialize_with = "uuid_to_string", deserialize_with = "string_to_uuid")]
    pub source: u128,
    #[serde(serialize_with = "uuid_to_string", deserialize_with = "string_to_uuid")]
    pub target: u128,
    #[serde(
        rename = "sourceHandle",
        serialize_with = "source_handle_to_string",
        deserialize_with = "string_to_source_handle"
    )]
    pub source_handle: i64,
    #[serde(
        rename = "targetHandle",
        serialize_with = "target_handle_to_string",
        deserialize_with = "string_to_target_handle"
    )]
    pub target_handle: i64,
    pub deletable: bool,
    pub selectable: bool,
    pub selected: bool,
    pub updatable: bool,
}

impl WbblWebappEdge {
    pub fn new(
        source: &u128,
        target: &u128,
        source_handle: i64,
        target_handle: i64,
    ) -> WbblWebappEdge {
        WbblWebappEdge {
            id: uuid::Uuid::new_v4().as_u128(),
            source: *source,
            target: *target,
            source_handle,
            target_handle,
            deletable: true,
            selected: false,
            selectable: true,
            updatable: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WbblWebappGraphSnapshot {
    #[serde(serialize_with = "uuid_to_string", deserialize_with = "string_to_uuid")]
    pub id: u128,
    pub nodes: Vec<WbblWebappNode>,
    pub edges: Vec<WbblWebappEdge>,
    pub computed_types: Option<HashMap<PortId, AbstractDataType>>,
}

impl From<WbblWebappEdge> for Edge {
    fn from(value: WbblWebappEdge) -> Self {
        Edge {
            id: value.id,
            input_port: InputPortId {
                node_id: value.target,
                port_index: value.target_handle as u8,
            },
            output_port: OutputPortId {
                node_id: value.source,
                port_index: value.source_handle as u8,
            },
        }
    }
}

impl Node {
    fn node_type_from_webapp_node(
        node: &WbblWebappNode,
        _incoming_edges: &Vec<&Edge>,
        _outgoing_edges: &Vec<&Edge>,
    ) -> NodeType {
        match node.node_type {
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
        }
    }

    pub fn from(
        webapp_node: &WbblWebappNode,
        incoming_edges: &Vec<&Edge>,
        outgoing_edges: &Vec<&Edge>,
    ) -> Self {
        let node_type =
            Self::node_type_from_webapp_node(webapp_node, incoming_edges, outgoing_edges);
        Node {
            id: webapp_node.id,
            input_port_count: node_type.input_port_count(incoming_edges, outgoing_edges),
            output_port_count: node_type.output_port_count(incoming_edges, outgoing_edges),
            node_type,
        }
    }
}

impl From<WbblWebappGraphSnapshot> for Graph {
    fn from(value: WbblWebappGraphSnapshot) -> Self {
        let edges: HashMap<u128, Edge> = value
            .edges
            .iter()
            .map(|e| (e.id, Edge::from(e.clone())))
            .collect();

        let input_edges_by_node: HashMap<u128, Vec<&Edge>> =
            edges.iter().fold(HashMap::new(), |mut prev, (_, e)| {
                let entry = prev.entry(e.input_port.node_id).or_insert(vec![]);
                entry.push(&e);
                prev
            });

        let output_edges_by_node: HashMap<u128, Vec<&Edge>> =
            edges.iter().fold(HashMap::new(), |mut prev, (_, e)| {
                let entry = prev.entry(e.output_port.node_id).or_insert(vec![]);
                entry.push(&e);
                prev
            });

        let nodes: HashMap<u128, Node> = value
            .nodes
            .iter()
            .map(|n| {
                (
                    n.id,
                    Node::from(
                        n,
                        input_edges_by_node.get(&n.id).unwrap_or(&vec![]),
                        output_edges_by_node.get(&n.id).unwrap_or(&vec![]),
                    ),
                )
            })
            .collect();

        let input_ports: HashMap<InputPortId, InputPort> = nodes
            .values()
            .flat_map(|n| n.input_ports(input_edges_by_node.get(&n.id).unwrap_or(&Vec::new())))
            .map(|p| (p.id.clone(), p))
            .collect();

        let output_ports: HashMap<OutputPortId, OutputPort> = nodes
            .values()
            .flat_map(|n| n.output_ports(output_edges_by_node.get(&n.id).unwrap_or(&Vec::new())))
            .map(|p| (p.id.clone(), p))
            .collect();

        let constraints: Vec<Constraint> = nodes.values().flat_map(|n| n.constraints()).collect();

        Self {
            id: value.id,
            nodes,
            edges,
            input_ports,
            output_ports,
            constraints,
        }
    }
}

impl WbblWebappGraphSnapshot {
    pub(crate) fn reassign_ids(&mut self) {
        let mut new_node_ids: HashMap<u128, u128> = HashMap::new();
        for n in self.nodes.iter_mut() {
            let old_id = n.id;
            let new_id = uuid::Uuid::new_v4().as_u128();
            n.id = new_id;
            new_node_ids.insert(old_id, new_id);
        }
        self.edges = self
            .edges
            .iter()
            .filter(|x| {
                new_node_ids.contains_key(&x.source) && new_node_ids.contains_key(&x.target)
            })
            .map(|x| x.clone())
            .collect();
        for e in self.edges.iter_mut() {
            e.id = uuid::Uuid::new_v4().as_u128();
            e.source = new_node_ids.get(&e.source).map(|s| *s).unwrap();
            e.target = new_node_ids.get(&e.target).map(|t| *t).unwrap();
        }
        self.id = uuid::Uuid::new_v4().as_u128();
    }

    pub(crate) fn filter_out_output_ports(&mut self) {
        let mut output_node_ids: HashSet<u128> = HashSet::new();
        self.nodes.retain_mut(|n| {
            if n.node_type == WbblWebappNodeType::Output {
                output_node_ids.insert(n.id);
                return false;
            }
            return true;
        });
        self.edges
            .retain_mut(|e| !output_node_ids.contains(&e.target));
    }

    pub(crate) fn offset(&mut self, offset: &Vec2) {
        for node in self.nodes.iter_mut() {
            let new_position =
                Vec2::new(node.position.x as f32, node.position.y as f32) + offset.clone();
            node.position = WbblePosition {
                x: new_position.x as f64,
                y: new_position.y as f64,
            }
        }
    }

    pub(crate) fn recenter(&mut self, position: &Vec2) {
        let mut accumulated_position = Vec2::ZERO;
        for node in self.nodes.iter() {
            accumulated_position =
                accumulated_position + Vec2::new(node.position.x as f32, node.position.y as f32);
        }
        let average_position = accumulated_position / (self.nodes.len() as f32);
        let delta_position = position.clone() - average_position;
        for node in self.nodes.iter_mut() {
            let final_position =
                Vec2::new(node.position.x as f32, node.position.y as f32) + delta_position;
            node.position = WbblePosition {
                x: final_position.x as f64,
                y: final_position.y as f64,
            };
        }
    }
}
