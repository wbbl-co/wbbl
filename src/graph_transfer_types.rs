use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};
use std::{collections::HashMap, str::FromStr, sync::Arc};
use wasm_bindgen::prelude::*;

use crate::graph_types::{Edge, Graph, InputPortId, OutputPortId};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct WbbleComputedNodeSize {
    pub width: Option<f64>,
    pub height: Option<f64>,
    #[serde(rename = "positionAbsolute")]
    pub position_absolute: Option<WbblePosition>,
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
}

pub fn get_type_name(node_type: WbblWebappNodeType) -> String {
    match node_type {
        WbblWebappNodeType::Output => "output".to_owned(),
        WbblWebappNodeType::Slab => "slab".to_owned(),
    }
}
#[wasm_bindgen]
pub fn from_type_name(type_name: &str) -> Option<WbblWebappNodeType> {
    match type_name {
        "output" => Some(WbblWebappNodeType::Output),
        "slab" => Some(WbblWebappNodeType::Slab),
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
    pub computed: Option<WbbleComputedNodeSize>,
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
    serializer.serialize_str(&format!("t-{}", number).to_owned())
}

fn source_handle_to_string<S>(number: &i64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&format!("s-{}", number).to_owned())
}

fn string_to_target_handle<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    let handle = String::deserialize(deserializer)?.replace("t-", "");

    handle
        .parse()
        .map_err(|_| Error::custom("Malformed target handle id"))
}

fn string_to_source_handle<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    let handle = String::deserialize(deserializer)?.replace("s-", "");

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
            updatable: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WbblWebappGraphSnapshot {
    #[serde(serialize_with = "uuid_to_string", deserialize_with = "string_to_uuid")]
    pub id: u128,
    pub nodes: Vec<WbblWebappNode>,
    pub edges: Vec<WbblWebappEdge>,
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

impl From<WbblWebappGraphSnapshot> for Graph {
    fn from(value: WbblWebappGraphSnapshot) -> Self {
        let nodes = value.nodes;

        Self {
            id: value.id,
            nodes: HashMap::new(),
            edges: value
                .edges
                .iter()
                .map(|e| (e.id, Edge::from(e.clone())))
                .collect(),
            input_ports: HashMap::new(),
            output_ports: HashMap::new(),
        }
    }
}
