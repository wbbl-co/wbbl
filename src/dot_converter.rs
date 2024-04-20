use glam::Vec2;
use std::collections::HashSet;
use std::vec;
use std::{collections::HashMap, str::FromStr};
use web_sys::js_sys::escape;
use web_sys::js_sys::unescape;

use crate::node_display_data::get_node_dimensions;
use crate::{
    graph_transfer_types::{
        from_type_name, get_type_name, Any, WbblWebappEdge, WbblWebappGraphSnapshot,
        WbblWebappNode, WbblWebappNodeType, WbblePosition,
    },
    log,
};

use graphviz_rust::{
    dot_generator::*,
    dot_structures::*,
    dot_structures::{Attribute, Edge, EdgeTy, Graph, Id, Node, NodeId, Port, Vertex},
    parse,
    printer::PrinterContext,
};

#[derive(Debug)]
pub enum DotFileError {
    NotDigraph,
    MustNotBeStrict,
    ParseError(String),
    MalformedId,
    MalformedData(String),
    MalformedAttribute,
    MalformedTypeName,
    MissingNodeType,
}

fn parse_escaped_id(id: &str) -> Result<u128, DotFileError> {
    uuid::Uuid::from_str(&id.replace("\"", ""))
        .map(|x| x.as_u128())
        .map_err(|_| DotFileError::MalformedId)
}

fn unquote_string(id: &str) -> String {
    id.replace("\"", "").to_owned()
}

pub fn from_dot(dotfile: &str) -> Result<WbblWebappGraphSnapshot, DotFileError> {
    let graph = parse(dotfile).map_err(|err| {
        log!("err {:?}", err);
        DotFileError::ParseError(err)
    })?;
    match graph {
        Graph::DiGraph {
            id: Id::Escaped(id),
            strict,
            stmts,
        } => {
            if strict {
                return Err(DotFileError::MustNotBeStrict);
            }
            let id = parse_escaped_id(&id)?;
            let mut nodes: Vec<WbblWebappNode> = vec![];
            let mut edges: Vec<WbblWebappEdge> = vec![];
            for statment in stmts.iter() {
                match statment {
                    graphviz_rust::dot_structures::Stmt::Node(Node {
                        id: NodeId(Id::Escaped(id), None),
                        attributes,
                    }) => {
                        let id = parse_escaped_id(id)?;
                        let mut node_type: Option<WbblWebappNodeType> = None;
                        let mut node_data: HashMap<String, Any> = HashMap::new();
                        let mut position_x: f64 = 0.0;
                        let mut position_y: f64 = 0.0;
                        let mut group_id: Option<u128> = None;

                        for attribute in attributes {
                            match attribute {
                                Attribute(Id::Plain(id), Id::Escaped(data))
                                    if id.starts_with("data_") =>
                                {
                                    let decoded: String = unescape(&unquote_string(&data)).into();
                                    let value: Any =
                                        serde_json::from_str(&decoded).map_err(|err| {
                                            log!("json_err {:?}", err);
                                            DotFileError::MalformedData(data.to_owned())
                                        })?;
                                    node_data.insert(id.replacen("data_", "", 1).to_owned(), value);
                                }
                                Attribute(Id::Plain(id), Id::Escaped(group))
                                    if id.starts_with("group") =>
                                {
                                    let unquoted: String = unquote_string(&group);
                                    let id = uuid::Uuid::parse_str(&unquoted)
                                        .map_err(|_| DotFileError::MalformedId)?;
                                    group_id = Some(id.as_u128());
                                }
                                Attribute(Id::Plain(id), Id::Plain(x)) if id == "x" => {
                                    position_x = str::parse(&x)
                                        .map_err(|_| DotFileError::MalformedAttribute)?;
                                }
                                Attribute(Id::Plain(id), Id::Plain(x)) if id == "y" => {
                                    position_y = str::parse(&x)
                                        .map_err(|_| DotFileError::MalformedAttribute)?;
                                }
                                Attribute(Id::Plain(id), Id::Escaped(node_type_str))
                                    if id == "label" =>
                                {
                                    match from_type_name(&unquote_string(&node_type_str)) {
                                        Some(t) => node_type = Some(t),
                                        None => return Err(DotFileError::MalformedTypeName),
                                    }
                                }
                                _ => {}
                            }
                        }
                        match node_type {
                            Some(node_type) => {
                                let (width, height) = get_node_dimensions(node_type, None, None);
                                nodes.push(WbblWebappNode {
                                    id,
                                    position: WbblePosition {
                                        x: position_x,
                                        y: position_y,
                                    },
                                    node_type,
                                    width,
                                    height,
                                    data: node_data,
                                    dragging: false,
                                    resizing: false,
                                    selected: false,
                                    selectable: true,
                                    connectable: true,
                                    deletable: true,
                                    in_edges: HashSet::new(),
                                    out_edges: HashSet::new(),
                                    selections: HashSet::new(),
                                    group_id,
                                })
                            }
                            _ => return Err(DotFileError::MissingNodeType),
                        }
                    }
                    graphviz_rust::dot_structures::Stmt::Edge(Edge {
                        ty:
                            EdgeTy::Pair(
                                Vertex::N(NodeId(
                                    Id::Escaped(from_id),
                                    Some(Port(Some(Id::Plain(from_port)), None)),
                                )),
                                Vertex::N(NodeId(
                                    Id::Escaped(to_id),
                                    Some(Port(Some(Id::Plain(to_port)), None)),
                                )),
                            ),
                        attributes: _,
                    }) => {
                        let source = parse_escaped_id(from_id)?;
                        let target = parse_escaped_id(to_id)?;
                        let source_handle: i64 = str::parse(&from_port)
                            .map_err(|_| DotFileError::ParseError(from_port.to_owned()))?;
                        let target_handle: i64 = str::parse(&to_port)
                            .map_err(|_| DotFileError::ParseError(to_port.to_owned()))?;
                        edges.push(WbblWebappEdge {
                            id: uuid::Uuid::new_v4().as_u128(),
                            source,
                            target,
                            source_handle,
                            target_handle,
                            deletable: true,
                            selections: HashSet::new(),
                            selectable: true,
                            selected: false,
                            updatable: false,
                            source_position: Vec2::ZERO,
                            group_id: None,
                            target_position: Vec2::ZERO,
                        })
                    }
                    _ => {}
                }
            }

            Ok(WbblWebappGraphSnapshot {
                id,
                nodes,
                edges,
                node_groups: vec![],
                computed_types: HashMap::new(),
            })
        }
        _ => Err(DotFileError::NotDigraph),
    }
}

fn to_node(node: &WbblWebappNode) -> Node {
    let type_name = get_type_name(node.node_type);
    let mut data_attributes: Vec<Attribute> = node
        .data
        .iter()
        .map(|(k, v)| {
            let value = serde_json::to_string(v).unwrap();
            let key = format!("data_{}", k);
            let escaped: String = escape(&value).into();
            attr!(key, esc escaped)
        })
        .collect();
    let mut dot_node = node!(esc uuid::Uuid::from_u128(node.id).to_string();
        attr!("label", esc type_name),
        attr!("x", node.position.x.to_string()),
        attr!("y", node.position.y.to_string())
    );
    if let Some(group_id) = node.group_id {
        dot_node
            .attributes
            .push(attr!("group", esc uuid::Uuid::from_u128(group_id).to_string()));
    }
    dot_node.attributes.append(&mut data_attributes);
    dot_node
}

pub fn to_dot(graph: &WbblWebappGraphSnapshot) -> String {
    let mut stmts = graph
        .nodes
        .iter()
        .map(|n| stmt!(to_node(n)))
        .collect::<Vec<Stmt>>();

    let mut edges = graph
        .edges
        .iter()
        .map(|e| {
            stmt!(edge!(
            node_id!(esc uuid::Uuid::from_u128(e.source).to_string(), port!(id!(e.source_handle.to_string()))) =>
            node_id!(esc uuid::Uuid::from_u128(e.target).to_string(), port!(id!(e.target_handle.to_string()))))
            )
        })
        .collect::<Vec<Stmt>>();

    stmts.append(&mut edges);

    graphviz_rust::print(
        graph!(
            di id!(esc uuid::Uuid::from_u128(graph.id).to_string()),
            stmts
        ),
        &mut PrinterContext::default(),
    )
}
