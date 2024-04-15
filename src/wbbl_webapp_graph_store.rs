use std::{
    cell::RefCell,
    collections::{HashMap, HashSet, VecDeque},
    rc::Rc,
    str::FromStr,
    sync::Arc,
};

use glam::{Mat2, Vec2};
use wasm_bindgen::prelude::*;
use web_sys::{js_sys, MessageEvent, Worker};
use yrs::{types::ToJson, Map, MapPrelim, MapRef, ReadTxn, Transact, TransactionMut, Value};

use crate::{
    convex_hull::{get_convex_hull, get_line_line_intersection},
    data_types::AbstractDataType,
    graph_transfer_types::{
        from_type_name, get_type_name, Any, WbblWebappEdge, WbblWebappGraphSnapshot,
        WbblWebappNode, WbblWebappNodeGroup, WbblWebappNodeType, WbbleComputedNodeSize,
        WbblePosition,
    },
    graph_types::PortId,
    log,
    store_errors::WbblWebappStoreError,
    wbbl_graph_web_worker::{WbblGraphWebWorkerRequestMessage, WbblGraphWebWorkerResponseMessage},
    yrs_utils::*,
};

const GRAPH_YRS_NODE_GROUPS_MAP_KEY: &str = "node_groups";
const GRAPH_YRS_NODE_SELECTIONS_MAP_KEY: &str = "node_selections";
const GRAPH_YRS_EDGE_SELECTIONS_MAP_KEY: &str = "edge_selections";
const GRAPH_YRS_NODES_MAP_KEY: &str = "nodes";
const GRAPH_YRS_EDGES_MAP_KEY: &str = "edges";

#[wasm_bindgen]
pub struct WbblWebappGraphStore {
    id: u128,
    next_listener_handle: u32,
    listeners: Arc<RefCell<Vec<(u32, js_sys::Function)>>>,
    undo_manager: yrs::UndoManager,
    graph: Arc<yrs::Doc>,
    nodes: yrs::MapRef,
    node_selections: yrs::MapRef,
    edge_selections: yrs::MapRef,
    node_groups: yrs::MapRef,
    edges: yrs::MapRef,
    computed_node_sizes: HashMap<u128, WbbleComputedNodeSize>,
    graph_worker: Worker,
    worker_responder: Closure<dyn FnMut(MessageEvent) -> ()>,
    computed_types: Arc<RefCell<HashMap<PortId, AbstractDataType>>>,
    initialized: bool,
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct NewWbblWebappNode {
    id: u128,
    position: WbblePosition,
    node_type: WbblWebappNodeType,
    data: HashMap<String, Any>,
}

fn encode_data(
    data: &HashMap<String, Any>,
    transaction: &mut TransactionMut,
    node_ref: &mut yrs::MapRef,
) -> Result<HashMap<String, Any>, WbblWebappStoreError> {
    let mut map: HashMap<String, yrs::Any> = HashMap::new();
    for (key, value) in data.iter() {
        // Probably not that fast, but whatever
        let yrs_value = value.to_yrs();
        map.insert(key.to_owned(), yrs_value);
    }
    let prelim_map = MapPrelim::from(map.clone());
    node_ref.insert(transaction, "data", prelim_map);

    Ok(map.iter().map(|(k, v)| (k.clone(), v.into())).collect())
}

impl NewWbblWebappNode {
    pub(crate) fn encode(
        &self,
        transaction: &mut TransactionMut,
        nodes: &mut yrs::MapRef,
    ) -> Result<WbblWebappNode, WbblWebappStoreError> {
        let data = {
            let prelim_map: yrs::MapPrelim<yrs::Any> = yrs::MapPrelim::new();
            let mut node_ref = nodes.insert(
                transaction,
                uuid::Uuid::from_u128(self.id).to_string(),
                prelim_map,
            );
            node_ref.insert(transaction, "type", get_type_name(self.node_type));
            node_ref.insert(transaction, "x", self.position.x);
            node_ref.insert(transaction, "y", self.position.y);
            node_ref.insert(transaction, "dragging", false);
            node_ref.insert(transaction, "resizing", false);
            let prelim_in_edges: MapPrelim<yrs::Any> = yrs::MapPrelim::new();
            node_ref.insert(transaction, "in_edges", prelim_in_edges);
            let prelim_out_edges: MapPrelim<yrs::Any> = yrs::MapPrelim::new();
            node_ref.insert(transaction, "out_edges", prelim_out_edges);
            encode_data(&self.data, transaction, &mut node_ref)
        }?;

        Ok(WbblWebappNode {
            id: self.id,
            position: self.position,
            node_type: self.node_type,
            data,
            measured: None,
            dragging: false,
            resizing: false,
            selected: false,
            selectable: true,
            deletable: self.node_type != WbblWebappNodeType::Output,
            connectable: true,
            group_id: None,
        })
    }
}

#[wasm_bindgen]
impl NewWbblWebappNode {
    fn get_initial_data(node_type: WbblWebappNodeType) -> HashMap<String, Any> {
        match node_type {
            WbblWebappNodeType::Output => HashMap::new(),
            WbblWebappNodeType::Slab => HashMap::new(),
            WbblWebappNodeType::Preview => HashMap::new(),
            WbblWebappNodeType::Add => HashMap::new(),
            WbblWebappNodeType::Subtract => HashMap::new(),
            WbblWebappNodeType::Multiply => HashMap::new(),
            WbblWebappNodeType::Divide => HashMap::new(),
            WbblWebappNodeType::Modulo => HashMap::new(),
            WbblWebappNodeType::Equal => HashMap::new(),
            WbblWebappNodeType::NotEqual => HashMap::new(),
            WbblWebappNodeType::Less => HashMap::new(),
            WbblWebappNodeType::LessEqual => HashMap::new(),
            WbblWebappNodeType::Greater => HashMap::new(),
            WbblWebappNodeType::GreaterEqual => HashMap::new(),
            WbblWebappNodeType::And => HashMap::new(),
            WbblWebappNodeType::Or => HashMap::new(),
            WbblWebappNodeType::ShiftLeft => HashMap::new(),
            WbblWebappNodeType::ShiftRight => HashMap::new(),
            WbblWebappNodeType::WorldPosition => HashMap::new(),
            WbblWebappNodeType::ClipPosition => HashMap::new(),
            WbblWebappNodeType::WorldNormal => HashMap::new(),
            WbblWebappNodeType::WorldBitangent => HashMap::new(),
            WbblWebappNodeType::WorldTangent => HashMap::new(),
            WbblWebappNodeType::TexCoord => HashMap::new(),
            WbblWebappNodeType::TexCoord2 => HashMap::new(),
            WbblWebappNodeType::Junction => HashMap::new(),
        }
    }
    pub fn new(
        position_x: f64,
        position_y: f64,
        node_type: WbblWebappNodeType,
    ) -> Result<NewWbblWebappNode, WbblWebappStoreError> {
        Ok(NewWbblWebappNode {
            id: uuid::Uuid::new_v4().as_u128(),
            position: WbblePosition {
                x: position_x,
                y: position_y,
            },
            node_type,
            data: NewWbblWebappNode::get_initial_data(node_type),
        })
    }

    pub fn new_with_data(
        position_x: f64,
        position_y: f64,
        node_type: WbblWebappNodeType,
        data: JsValue,
    ) -> Result<NewWbblWebappNode, WbblWebappStoreError> {
        let data = serde_wasm_bindgen::from_value::<HashMap<String, Any>>(data).unwrap();
        Ok(NewWbblWebappNode {
            id: uuid::Uuid::new_v4().as_u128(),
            position: WbblePosition {
                x: position_x,
                y: position_y,
            },
            node_type,
            data,
        })
    }
}

impl WbblWebappGraphSnapshot {
    pub(crate) fn get_full_snapshot<Transaction: yrs::ReadTxn>(
        read_txn: &Transaction,
        graph_id: u128,
        nodes_map_ref: &yrs::MapRef,
        edges_map_ref: &yrs::MapRef,
        node_selections_map: &yrs::MapRef,
        edge_selections_map: &yrs::MapRef,
        node_groups_map: &yrs::MapRef,
        computed_node_sizes: &HashMap<u128, WbbleComputedNodeSize>,
        client_id: u64,
    ) -> Result<WbblWebappGraphSnapshot, WbblWebappStoreError> {
        let mut nodes: Vec<WbblWebappNode> = Vec::new();
        let mut edges: Vec<WbblWebappEdge> = Vec::new();
        let mut groups: Vec<WbblWebappNodeGroup> = Vec::new();
        for node in nodes_map_ref.iter(read_txn) {
            let key = node.0.to_owned();
            let node_values = node.1;
            let key = uuid::Uuid::from_str(&key).map_err(|_| WbblWebappStoreError::MalformedId)?;
            let node: WbblWebappNode = match node_values {
                yrs::Value::YMap(map) => WbblWebappNode::decode(
                    &key.as_u128(),
                    read_txn,
                    &map,
                    node_selections_map,
                    client_id,
                ),
                _ => Err(WbblWebappStoreError::UnexpectedStructure),
            }?;
            nodes.push(node);
        }

        for edge in edges_map_ref.iter(read_txn) {
            let key = edge.0.to_owned();
            let edge_values = edge.1;
            let key = uuid::Uuid::from_str(&key).map_err(|_| WbblWebappStoreError::MalformedId)?;

            let edge: WbblWebappEdge = match edge_values {
                yrs::Value::YMap(map) => WbblWebappEdge::decode(
                    key.as_u128(),
                    read_txn,
                    &map,
                    edge_selections_map,
                    client_id,
                ),
                _ => Err(WbblWebappStoreError::UnexpectedStructure),
            }?;
            edges.push(edge);
        }

        for group in node_groups_map.iter(read_txn) {
            let key = group.0;
            let key_uuid =
                uuid::Uuid::from_str(key).map_err(|_| WbblWebappStoreError::MalformedId)?;
            let nodes: Vec<String> = match group.1 {
                Value::YMap(group) => Ok(group.keys(read_txn).map(|k| k.to_owned()).collect()),
                _ => Err(WbblWebappStoreError::UnexpectedStructure),
            }?;
            let nodes = nodes.iter().try_fold(Vec::<u128>::new(), |mut prev, n| {
                let id = uuid::Uuid::from_str(n)
                    .map_err(|_| WbblWebappStoreError::MalformedId)?
                    .as_u128();
                prev.push(id);
                Ok(prev)
            })?;
            let path = get_group_path(
                read_txn,
                key,
                nodes_map_ref,
                &node_groups_map,
                computed_node_sizes,
            )?;
            groups.push(WbblWebappNodeGroup {
                id: key_uuid.as_u128(),
                path: Some(path),
                nodes,
            })
        }

        nodes.sort_by_key(|n| n.id.clone());
        edges.sort_by_key(|e| e.id.clone());
        Ok(WbblWebappGraphSnapshot {
            id: graph_id,
            nodes,
            edges,
            node_groups: Some(groups),
            computed_types: None,
        })
    }
}

impl WbblWebappNode {
    pub(crate) fn decode<Txn: yrs::ReadTxn>(
        key: &u128,
        txn: &Txn,
        map: &yrs::MapRef,
        selections: &yrs::MapRef,
        client_id: u64,
    ) -> Result<WbblWebappNode, WbblWebappStoreError> {
        let type_name: String = get_atomic_string("type", txn, map)?;
        let data = get_map("data", txn, map)?;
        let x = get_float_64("x", txn, map)?;
        let y = get_float_64("y", txn, map)?;
        let dragging = get_bool("dragging", txn, map)?;
        let resizing = get_bool("resizing", txn, map)?;
        let maybe_group_id = get_atomic_u128_from_string("group_id", txn, map);
        let group_id = match maybe_group_id {
            Ok(id) => Ok(Some(id)),
            Err(WbblWebappStoreError::NotFound) => Ok(None),
            Err(err) => Err(err),
        }?;

        let data = &data.to_json(txn);
        let data = match data {
            yrs::Any::Map(hash_map) => Ok(hash_map.clone()),
            _ => Err(WbblWebappStoreError::UnexpectedStructure),
        }?;
        let node_type = match from_type_name(&type_name) {
            Some(typ) => Ok(typ),
            None => Err(WbblWebappStoreError::UnknownNodeType),
        }?;
        let key_str = uuid::Uuid::from_u128(*key).to_string();
        let selected = match selections.get(txn, &client_id.to_string()) {
            Some(Value::YMap(selection)) => selection.contains_key(txn, &key_str),
            _ => false,
        };
        Ok(WbblWebappNode {
            id: *key,
            position: WbblePosition { x, y },
            measured: None,
            node_type,
            dragging,
            resizing,
            selected,
            data: data
                .iter()
                .map(|(k, v)| (k.to_owned(), v.into()))
                .collect::<HashMap<String, Any>>(),
            connectable: true,
            selectable: true,
            deletable: node_type != WbblWebappNodeType::Output,
            group_id,
        })
    }

    pub(crate) fn encode(
        &self,
        transaction: &mut TransactionMut,
        nodes: &mut yrs::MapRef,
        node_groups: &mut MapRef,
    ) -> Result<(), WbblWebappStoreError> {
        let prelim_map: yrs::MapPrelim<yrs::Any> = yrs::MapPrelim::new();
        let node_id = uuid::Uuid::from_u128(self.id).to_string();

        let mut node_ref = nodes.insert(transaction, node_id.clone(), prelim_map);
        node_ref.insert(transaction, "type", get_type_name(self.node_type));
        node_ref.insert(transaction, "x", self.position.x);
        node_ref.insert(transaction, "y", self.position.y);
        node_ref.insert(transaction, "dragging", false);
        node_ref.insert(transaction, "resizing", false);
        let prelim_in_edges: MapPrelim<yrs::Any> = yrs::MapPrelim::new();
        node_ref.insert(transaction, "in_edges", prelim_in_edges);
        let prelim_out_edges: MapPrelim<yrs::Any> = yrs::MapPrelim::new();
        node_ref.insert(transaction, "out_edges", prelim_out_edges);
        if let Some(group_id) = self.group_id {
            let group_id = uuid::Uuid::from_u128(group_id).to_string();
            node_ref.insert(transaction, "group_id", group_id.clone());
            node_groups.insert(
                transaction,
                group_id,
                yrs::MapPrelim::<bool>::from([(node_id, true)]),
            );
        }
        encode_data(&self.data, transaction, &mut node_ref)?;
        Ok(())
    }
}

fn delete_edge(
    transaction: &mut TransactionMut,
    edge_id: &str,
    edges: &mut MapRef,
    nodes: &mut MapRef,
    edge_selections: &mut MapRef,
) -> Result<(), WbblWebappStoreError> {
    let edge = match get_map(edge_id, transaction, edges) {
        Ok(edge) => Ok(Some(edge)),
        Err(WbblWebappStoreError::NotFound) => Ok(None),
        Err(err) => Err(err),
    }?;
    if edge.is_none() {
        return Ok(());
    }
    let edge = edge.unwrap();
    let source = get_atomic_string("source", transaction, &edge)?;
    let target = get_atomic_string("target", transaction, &edge)?;
    edges.remove(transaction, edge_id);
    match get_map(&source, transaction, nodes) {
        Ok(in_node) => {
            let in_edges = get_map("in_edges", transaction, &in_node)?;
            in_edges.remove(transaction, edge_id);
            Ok(())
        }
        Err(WbblWebappStoreError::NotFound) => Ok(()),
        Err(err) => Err(err),
    }?;
    match get_map(&target, transaction, nodes) {
        Ok(out_node) => {
            let out_edges = get_map("out_edges", transaction, &out_node)?;
            out_edges.remove(transaction, edge_id);
            Ok(())
        }
        Err(WbblWebappStoreError::NotFound) => Ok(()),
        Err(err) => Err(err),
    }?;
    {
        let keys: Vec<String> = edge_selections
            .keys(transaction)
            .map(|x| x.to_owned())
            .collect();
        for k in keys {
            match edge_selections.get(transaction, &k) {
                Some(yrs::Value::YMap(map)) => {
                    map.remove(transaction, edge_id);
                }
                _ => {}
            };
        }
    }

    Ok(())
}

fn delete_associated_edges(
    transaction: &mut TransactionMut,
    node: &MapRef,
    nodes: &mut MapRef,
    edges: &mut MapRef,
    edge_selections: &mut MapRef,
) -> Result<(), WbblWebappStoreError> {
    let in_edges = get_map("in_edges", transaction, &node)?;
    let out_edges = get_map("out_edges", transaction, &node)?;
    {
        let in_edges: Vec<String> = in_edges.iter(transaction).map(|x| x.0.to_owned()).collect();
        let out_edges: Vec<String> = out_edges
            .iter(transaction)
            .map(|x| x.0.to_owned())
            .collect();
        for edge in in_edges.iter() {
            delete_edge(transaction, edge, edges, nodes, edge_selections)?;
        }
        for edge in out_edges.iter() {
            delete_edge(transaction, edge, edges, nodes, edge_selections)?;
        }
        Ok(())
    }
}

fn remove_node_from_group(
    transaction: &mut TransactionMut,
    node_id: &str,
    node: &MapRef,
    node_groups: &mut MapRef,
) -> Result<(), WbblWebappStoreError> {
    if let Ok(group_id) = get_atomic_string("group_id", transaction, node) {
        if let Ok(group) = get_map(&group_id, transaction, node_groups) {
            group.remove(transaction, node_id);
            if group.len(transaction) == 0 {
                node_groups.remove(transaction, &group_id);
            }
        }
    }
    Ok(())
}

fn delete_node(
    transaction: &mut TransactionMut,
    node_id: &str,
    nodes: &mut MapRef,
    edges: &mut MapRef,
    node_selections: &mut MapRef,
    edge_selections: &mut MapRef,
    computed_node_sizes: &mut HashMap<u128, WbbleComputedNodeSize>,
    node_groups: &mut MapRef,
) -> Result<(), WbblWebappStoreError> {
    let node = match get_map(node_id, transaction, &nodes) {
        Ok(node) => Ok(Some(node)),
        Err(WbblWebappStoreError::NotFound) => Ok(None),
        Err(err) => Err(err),
    }?;
    if node.is_none() {
        return Ok(());
    }
    let node = node.unwrap();
    delete_associated_edges(transaction, &node, nodes, edges, edge_selections)?;
    remove_node_from_group(transaction, node_id, &node, node_groups)?;
    let node_id_u128 = uuid::Uuid::parse_str(node_id)
        .map_err(|_| WbblWebappStoreError::MalformedId)?
        .as_u128();
    computed_node_sizes.remove(&node_id_u128);
    nodes.remove(transaction, node_id);
    {
        let keys: Vec<String> = node_selections
            .keys(transaction)
            .map(|x| x.to_owned())
            .collect();
        for k in keys {
            match node_selections.get(transaction, &k) {
                Some(yrs::Value::YMap(map)) => {
                    map.remove(transaction, node_id);
                }
                _ => {}
            };
        }
    }
    Ok(())
}

pub fn get_group_bounds(
    txn: &mut TransactionMut,
    group_id: &str,
    nodes: &MapRef,
    node_groups: &mut MapRef,
    computed_node_sizes: &HashMap<u128, WbbleComputedNodeSize>,
) -> Result<Vec<f32>, WbblWebappStoreError> {
    let mut min_position = Vec2::new(f32::MAX, f32::MAX);
    let mut max_position = Vec2::new(f32::MIN, f32::MIN);

    let group = get_map(group_id, txn, &node_groups)?;
    let node_ids: Vec<String> = group.keys(txn).map(|k| k.to_owned()).collect();
    for node_id in node_ids {
        match get_map(&node_id, txn, nodes) {
            Ok(map) => {
                let node_id_u128 = uuid::Uuid::parse_str(&node_id)
                    .map_err(|_| WbblWebappStoreError::MalformedId)?
                    .as_u128();
                if let Some(WbbleComputedNodeSize {
                    width: Some(w),
                    height: Some(h),
                }) = computed_node_sizes.get(&node_id_u128)
                {
                    let min_pos_x = get_float_64("x", txn, &map)? as f32;
                    let max_pos_x = min_pos_x + (*w as f32);
                    if min_pos_x < min_position.x {
                        min_position.x = min_pos_x;
                    }
                    if max_pos_x > max_position.x {
                        max_position.x = max_pos_x;
                    }
                    let min_pos_y = get_float_64("y", txn, &map)? as f32;
                    let max_pos_y = min_pos_y + (*h as f32);
                    if min_pos_y < min_position.y {
                        min_position.y = min_pos_y;
                    }
                    if max_pos_y > max_position.y {
                        max_position.y = max_pos_y;
                    }
                }
            }
            Err(WbblWebappStoreError::NotFound) => {}
            Err(err) => return Err(err),
        };
    }
    Ok(Vec::from([
        min_position.x,
        min_position.y,
        max_position.x,
        min_position.y,
        max_position.x,
        max_position.y,
        min_position.x,
        max_position.y,
    ]))
}

pub fn get_group_convex_hull<Txn: ReadTxn>(
    txn: &Txn,
    group_id: &str,
    nodes: &MapRef,
    node_groups: &MapRef,
    computed_node_sizes: &HashMap<u128, WbbleComputedNodeSize>,
) -> Result<Vec<Vec2>, WbblWebappStoreError> {
    let mut positions: Vec<Vec2> = Vec::new();
    let group = get_map(group_id, txn, &node_groups)?;
    let node_ids: Vec<String> = group.keys(txn).map(|k| k.to_owned()).collect();
    for node_id in node_ids {
        match get_map(&node_id, txn, nodes) {
            Ok(map) => {
                let node_id_u128 = uuid::Uuid::parse_str(&node_id)
                    .map_err(|_| WbblWebappStoreError::MalformedId)?
                    .as_u128();
                if let Some(WbbleComputedNodeSize {
                    width: Some(w),
                    height: Some(h),
                }) = computed_node_sizes.get(&node_id_u128)
                {
                    let min_pos_x = get_float_64("x", txn, &map)? as f32;
                    let max_pos_x = min_pos_x + (*w as f32);

                    let min_pos_y = get_float_64("y", txn, &map)? as f32;
                    let max_pos_y = min_pos_y + (*h as f32);
                    positions.push(Vec2 {
                        x: min_pos_x,
                        y: min_pos_y,
                    });

                    positions.push(Vec2 {
                        x: max_pos_x,
                        y: max_pos_y,
                    });

                    positions.push(Vec2 {
                        x: min_pos_x,
                        y: max_pos_y,
                    });

                    positions.push(Vec2 {
                        x: max_pos_x,
                        y: min_pos_y,
                    });
                }
            }
            Err(WbblWebappStoreError::NotFound) => {}
            Err(err) => return Err(err),
        };
    }

    Ok(get_convex_hull(&mut positions))
}

const INFLATE_GROUP_PATH_BY: f32 = 25.0;

pub fn get_group_path<Txn: ReadTxn>(
    txn: &Txn,
    group_id: &str,
    nodes: &MapRef,
    node_groups: &MapRef,
    computed_node_sizes: &HashMap<u128, WbbleComputedNodeSize>,
) -> Result<String, WbblWebappStoreError> {
    let convex_hull =
        get_group_convex_hull(txn, group_id, nodes, node_groups, computed_node_sizes)?;

    if convex_hull.len() > 2 {
        let first = convex_hull[0];
        let second = convex_hull[1];
        let delta_first_second = second - first;
        let first_second_tangent = Vec2::new(-delta_first_second.y, delta_first_second.x)
            .normalize()
            * INFLATE_GROUP_PATH_BY;
        let first_1 = first_second_tangent + first;
        let mut result = format!("M {} {}", first_1.x, first_1.y);
        let mut i = 1;
        while i <= convex_hull.len() {
            let prev = convex_hull[i - 1];
            let current = convex_hull[i % convex_hull.len()];
            let wrapped_index = (i + 1) % convex_hull.len();
            let next = convex_hull[wrapped_index];
            let delta_current_prev = current.clone() - prev.clone();
            let delta_next_current = next - current.clone();
            let tangent_prev = INFLATE_GROUP_PATH_BY
                * Vec2::new(-delta_current_prev.y, delta_current_prev.x).normalize();
            let tangent_next = INFLATE_GROUP_PATH_BY
                * Vec2::new(-delta_next_current.y, delta_next_current.x).normalize();
            let prev = prev + tangent_prev;
            let current_1 = current + tangent_prev;
            let current_2 = current + tangent_next;
            let next = next + tangent_next;
            let intersection = get_line_line_intersection(&prev, &current_1, &current_2, &next);
            result.push_str(&format!("L {} {}", current_1.x, current_1.y));
            result.push_str(&format!(
                "Q {} {}, {} {}",
                intersection.x, intersection.y, current_2.x, current_2.y
            ));
            i += 1;
        }

        result.push('Z');
        return Ok(result);
    }
    return Ok("".to_owned());
}

impl WbblWebappEdge {
    pub(crate) fn decode<Txn: yrs::ReadTxn>(
        key: u128,
        txn: &Txn,
        map: &yrs::MapRef,
        selections: &yrs::MapRef,
        client_id: u64,
    ) -> Result<WbblWebappEdge, WbblWebappStoreError> {
        let source = get_atomic_u128_from_string("source", txn, map)?;
        let target = get_atomic_u128_from_string("target", txn, map)?;
        let source_handle = get_atomic_bigint("source_handle", txn, map)?;
        let target_handle = get_atomic_bigint("target_handle", txn, map)?;
        let key_str = uuid::Uuid::from_u128(key).to_string();
        let selected = match selections.get(txn, &client_id.to_string()) {
            Some(Value::YMap(selection)) => selection.contains_key(txn, &key_str),
            _ => false,
        };
        Ok(WbblWebappEdge {
            id: key,
            source,
            target,
            source_handle,
            target_handle,
            deletable: true,
            selectable: true,
            updatable: false,
            selected,
        })
    }

    fn set_node_edge_ids(
        &self,
        txn: &mut TransactionMut,
        nodes: &yrs::MapRef,
    ) -> Result<(), WbblWebappStoreError> {
        let source_node = nodes.get(txn, &uuid::Uuid::from_u128(self.source).to_string());
        let target_node = nodes.get(txn, &uuid::Uuid::from_u128(self.target).to_string());
        match (source_node, target_node) {
            (Some(Value::YMap(source_node)), Some(Value::YMap(target_node))) => {
                let edge_id_str = uuid::Uuid::from_u128(self.id).to_string();
                {
                    let out_edges = get_map("out_edges", txn, &source_node)?;
                    out_edges.insert(txn, edge_id_str.clone(), yrs::Any::Bool(true));
                }
                {
                    let in_edges = get_map("in_edges", txn, &target_node)?;
                    in_edges.insert(txn, edge_id_str, yrs::Any::Bool(true));
                }
                Ok(())
            }
            (_, _) => Err(WbblWebappStoreError::UnexpectedStructure),
        }?;
        Ok(())
    }

    pub(crate) fn encode(
        &self,
        txn: &mut TransactionMut,
        edges: &mut yrs::MapRef,
        nodes: &yrs::MapRef,
    ) -> Result<(), WbblWebappStoreError> {
        let prelim_map: HashMap<String, yrs::Any> = HashMap::new();
        let edge_ref = edges.insert(
            txn,
            uuid::Uuid::from_u128(self.id).to_string(),
            MapPrelim::from(prelim_map),
        );

        edge_ref.insert(
            txn,
            "source".to_owned(),
            yrs::Any::String(uuid::Uuid::from_u128(self.source).to_string().into()),
        );
        edge_ref.insert(
            txn,
            "target".to_owned(),
            yrs::Any::String(uuid::Uuid::from_u128(self.target).to_string().into()),
        );
        edge_ref.insert(
            txn,
            "source_handle".to_owned(),
            yrs::Any::BigInt(self.source_handle),
        );
        edge_ref.insert(
            txn,
            "target_handle".to_owned(),
            yrs::Any::BigInt(self.target_handle),
        );
        self.set_node_edge_ids(txn, nodes)?;
        Ok(())
    }
}

#[wasm_bindgen]
impl WbblWebappGraphStore {
    pub fn empty(graph_worker: Worker) -> Self {
        let graph = yrs::Doc::new();
        let node_selections = graph.get_or_insert_map(GRAPH_YRS_NODE_SELECTIONS_MAP_KEY.to_owned());
        let node_groups = graph.get_or_insert_map(GRAPH_YRS_NODE_GROUPS_MAP_KEY.to_owned());
        let edge_selections = graph.get_or_insert_map(GRAPH_YRS_EDGE_SELECTIONS_MAP_KEY.to_owned());
        let nodes = graph.get_or_insert_map(GRAPH_YRS_NODES_MAP_KEY.to_owned());
        let edges = graph.get_or_insert_map(GRAPH_YRS_EDGES_MAP_KEY.to_owned());
        let undo_manager = yrs::UndoManager::with_options(
            &graph,
            &nodes,
            yrs::undo::Options {
                capture_timeout_millis: 1_000,
                tracked_origins: HashSet::new(),
                capture_transaction: Rc::new(|_txn| true),
                timestamp: Rc::new(|| js_sys::Date::now() as u64),
            },
        );

        let computed_types = Arc::new(RefCell::new(HashMap::new()));

        let listeners = Arc::new(RefCell::new(Vec::<(u32, js_sys::Function)>::new()));
        let worker_responder = Closure::<dyn FnMut(MessageEvent) -> ()>::new({
            let computed_types = computed_types.clone();
            let listeners = listeners.clone();
            move |msg: MessageEvent| {
                match serde_wasm_bindgen::from_value::<WbblGraphWebWorkerResponseMessage>(
                    msg.data(),
                ) {
                    Ok(WbblGraphWebWorkerResponseMessage::TypesUpdated(types)) => {
                        computed_types.replace(types);
                        for (_, listener) in listeners.borrow().iter() {
                            listener
                                .call0(&JsValue::UNDEFINED)
                                .map_err(|_| WbblWebappStoreError::FailedToEmit)
                                .unwrap();
                        }
                    }
                    Ok(WbblGraphWebWorkerResponseMessage::TypeUnificationFailure) => {
                        log!("Type unification failed");
                    }
                    Ok(WbblGraphWebWorkerResponseMessage::Ready) => {}
                    Err(_) => {
                        log!("Malformed message");
                    }
                };
                ()
            }
        });
        graph_worker
            .add_event_listener_with_callback("message", worker_responder.as_ref().unchecked_ref())
            .unwrap();

        let mut store = WbblWebappGraphStore {
            id: uuid::Uuid::new_v4().as_u128(),
            next_listener_handle: 0,
            listeners: listeners.clone(),
            undo_manager,
            graph: Arc::new(graph),
            nodes,
            edges,
            node_groups,
            node_selections,
            edge_selections,
            computed_node_sizes: HashMap::new(),
            computed_types: computed_types.clone(),
            graph_worker: graph_worker.clone(),
            worker_responder,
            initialized: false,
        };

        let output_node = NewWbblWebappNode::new(600.0, 500.0, WbblWebappNodeType::Output).unwrap();
        store.add_node(output_node.clone()).unwrap();

        let slab_node = NewWbblWebappNode::new(200.0, 500.0, WbblWebappNodeType::Slab).unwrap();
        store.add_node(slab_node.clone()).unwrap();

        store
            .add_edge(
                &uuid::Uuid::from_u128(slab_node.id).to_string(),
                &uuid::Uuid::from_u128(output_node.id).to_string(),
                0,
                0,
            )
            .unwrap();

        {
            let mut txn_mut = store.graph.transact_mut();

            let local_node_selections = store.node_selections.insert(
                &mut txn_mut,
                store.graph.client_id().to_string(),
                MapPrelim::<String>::new(),
            );

            let local_edge_selections = store.edge_selections.insert(
                &mut txn_mut,
                store.graph.client_id().to_string(),
                MapPrelim::<String>::new(),
            );
            store.undo_manager.expand_scope(&local_node_selections);
            store.undo_manager.expand_scope(&local_edge_selections);
        }
        store.undo_manager.include_origin(store.graph.client_id()); // only track changes originating from local peer
        store.undo_manager.expand_scope(&store.edges);
        store.undo_manager.expand_scope(&store.node_groups);

        store.initialized = true;
        store.emit(true).unwrap();
        store
    }

    pub fn emit(&self, should_publish_to_worker: bool) -> Result<(), WbblWebappStoreError> {
        for (_, listener) in self.listeners.borrow().iter() {
            listener
                .call0(&JsValue::UNDEFINED)
                .map_err(|_| WbblWebappStoreError::FailedToEmit)?;
        }
        if should_publish_to_worker && self.initialized {
            let snapshot = self.get_snapshot_raw()?;
            let snapshot_js_value = serde_wasm_bindgen::to_value(
                &WbblGraphWebWorkerRequestMessage::SetSnapshot(snapshot),
            )
            .map_err(|_| WbblWebappStoreError::UnexpectedStructure)?;

            self.graph_worker.post_message(&snapshot_js_value).unwrap();
        }
        Ok(())
    }

    pub fn subscribe(&mut self, subscriber: js_sys::Function) -> u32 {
        let handle = self.next_listener_handle;
        self.listeners.borrow_mut().push((handle, subscriber));
        self.next_listener_handle = self.next_listener_handle + 1;
        handle
    }

    pub fn unsubscribe(&mut self, handle: u32) {
        let mut listeners = self.listeners.borrow_mut();
        if let Some((idx, _)) = listeners
            .iter()
            .enumerate()
            .find(|(_, (k, _))| *k == handle)
        {
            let _ = listeners.remove(idx);
        }
    }

    pub fn undo(&mut self) -> Result<bool, WbblWebappStoreError> {
        let result = self
            .undo_manager
            .undo()
            .map_err(|_| WbblWebappStoreError::FailedToUndo)?;
        if result {
            self.emit(true)?;
        }
        Ok(result)
    }

    pub fn redo(&mut self) -> Result<bool, WbblWebappStoreError> {
        let result = self
            .undo_manager
            .redo()
            .map_err(|_| WbblWebappStoreError::FailedToRedo)?;
        if result {
            self.emit(true)?;
        }
        Ok(result)
    }

    pub fn can_undo(&self) -> bool {
        self.undo_manager.can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.undo_manager.can_redo()
    }

    pub fn get_snapshot(&mut self) -> Result<JsValue, WbblWebappStoreError> {
        let mut snapshot = self.get_snapshot_raw()?;
        snapshot.computed_types = Some(self.computed_types.borrow().clone());
        serde_wasm_bindgen::to_value(&snapshot)
            .map_err(|_| WbblWebappStoreError::UnexpectedStructure)
    }

    pub fn remove_node(&mut self, node_id: &str) -> Result<(), WbblWebappStoreError> {
        {
            let mut mut_transaction = self.graph.transact_mut_with(self.graph.client_id());
            let node = get_map(&node_id, &mut_transaction, &self.nodes)?;
            let type_name = get_atomic_string(&"type", &mut_transaction, &node)?;
            if let Some(WbblWebappNodeType::Output) = from_type_name(&type_name) {
                return Err(WbblWebappStoreError::CannotDeleteOutputNode);
            }
            delete_node(
                &mut mut_transaction,
                node_id,
                &mut self.nodes,
                &mut self.edges,
                &mut self.node_selections,
                &mut self.edge_selections,
                &mut self.computed_node_sizes,
                &mut self.node_groups,
            )?;
        }

        // Important that emit is called here. Rather than before the drop
        // as this could trigger a panic as the store value may be read
        self.emit(true)?;

        Ok(())
    }

    pub fn remove_selected_nodes_and_edges(&mut self) -> Result<(), WbblWebappStoreError> {
        {
            let selected_nodes = self.get_locally_selected_nodes()?;
            let selected_edges = self.get_locally_selected_edges()?;
            let mut mut_transaction = self.graph.transact_mut_with(self.graph.client_id());
            for node_id in selected_nodes {
                match get_map(&node_id, &mut_transaction, &self.nodes) {
                    Ok(node) => {
                        let type_name: String = get_atomic_string("type", &mut_transaction, &node)?;
                        if let Some(WbblWebappNodeType::Output) = from_type_name(&type_name) {
                            Ok(())
                        } else {
                            delete_node(
                                &mut mut_transaction,
                                &node_id,
                                &mut self.nodes,
                                &mut self.edges,
                                &mut self.node_selections,
                                &mut self.edge_selections,
                                &mut self.computed_node_sizes,
                                &mut self.node_groups,
                            )?;
                            Ok(())
                        }
                    }
                    Err(WbblWebappStoreError::NotFound) => Ok(()),
                    Err(err) => Err(err),
                }?;
            }
            for edge_id in selected_edges {
                delete_edge(
                    &mut mut_transaction,
                    &edge_id,
                    &mut self.edges,
                    &mut self.nodes,
                    &mut self.edge_selections,
                )?;
            }
        }
        self.emit(true)?;
        Ok(())
    }

    pub fn remove_edge(&mut self, edge_id: &str) -> Result<(), WbblWebappStoreError> {
        {
            let mut mut_transaction = self.graph.transact_mut_with(self.graph.client_id());
            delete_edge(
                &mut mut_transaction,
                &edge_id,
                &mut self.edges,
                &mut self.nodes,
                &mut self.edge_selections,
            )?;
        }

        // Important that emit is called here. Rather than before the drop
        // as this could trigger a panic as the store value may be read
        self.emit(true)?;

        Ok(())
    }

    pub fn add_node(&mut self, node: NewWbblWebappNode) -> Result<(), WbblWebappStoreError> {
        {
            let mut mut_transaction = self.graph.transact_mut_with(self.graph.client_id());
            node.encode(&mut mut_transaction, &mut self.nodes)
        }?;

        // Important that emit is called here. Rather than before the drop
        // as this could trigger a panic as the store value may be read
        self.emit(true)?;

        Ok(())
    }

    fn get_snapshot_raw(&self) -> Result<WbblWebappGraphSnapshot, WbblWebappStoreError> {
        let read_txn = self.graph.transact();
        let mut snapshot = WbblWebappGraphSnapshot::get_full_snapshot(
            &read_txn,
            self.id,
            &self.nodes,
            &self.edges,
            &self.node_selections,
            &self.edge_selections,
            &self.node_groups,
            &self.computed_node_sizes,
            self.graph.client_id(),
        )?;
        for node in snapshot.nodes.iter_mut() {
            node.measured = self.computed_node_sizes.get(&node.id).map(|s| s.clone());
        }
        Ok(snapshot)
    }

    pub fn set_computed_node_dimension(
        &mut self,
        node_id: &str,
        width: Option<f64>,
        height: Option<f64>,
        resizing: Option<bool>,
    ) -> Result<(), WbblWebappStoreError> {
        {
            let mut mut_transaction = self.graph.transact_mut_with(self.graph.client_id());
            match self.nodes.get(&mut_transaction, node_id) {
                Some(yrs::Value::YMap(node_ref)) => {
                    node_ref.insert(
                        &mut mut_transaction,
                        "resizing",
                        yrs::Any::Bool(resizing.unwrap_or(false)),
                    );
                    Ok(())
                }
                _ => Err(WbblWebappStoreError::UnexpectedStructure),
            }?;
        }

        let node_id = uuid::Uuid::from_str(node_id)
            .map(|id| id.as_u128())
            .map_err(|_| WbblWebappStoreError::MalformedId)?;

        if let Some(maybe_computed) = self.computed_node_sizes.get_mut(&node_id) {
            maybe_computed.width = width;
            maybe_computed.height = height;
        } else {
            self.computed_node_sizes
                .insert(node_id.to_owned(), WbbleComputedNodeSize { width, height });
        }

        // Important that emit is called here. Rather than before the drop
        // as this could trigger a panic as the store value may be read
        self.emit(false)?;

        Ok(())
    }

    pub fn set_node_position(
        &mut self,
        node_id: &str,
        x: f64,
        y: f64,
        dragging: Option<bool>,
    ) -> Result<(), WbblWebappStoreError> {
        {
            let mut mut_transaction = self.graph.transact_mut_with(self.graph.client_id());

            match self.nodes.get(&mut_transaction, node_id) {
                Some(yrs::Value::YMap(node_ref)) => {
                    node_ref.insert(&mut mut_transaction, "x", yrs::Any::Number(x));
                    node_ref.insert(&mut mut_transaction, "y", yrs::Any::Number(y));
                    node_ref.insert(
                        &mut mut_transaction,
                        "dragging",
                        yrs::Any::Bool(dragging.unwrap_or(false)),
                    );
                    Ok(())
                }
                _ => Err(WbblWebappStoreError::UnexpectedStructure),
            }?;
        }
        // Important that emit is called here. Rather than before the drop
        // as this could trigger a panic as the store value may be read
        self.emit(false)?;

        Ok(())
    }

    pub fn set_node_selection(
        &mut self,
        node_id: &str,
        selected: bool,
    ) -> Result<(), WbblWebappStoreError> {
        {
            let mut mut_transaction = self.graph.transact_mut_with(self.graph.client_id());
            let id = self.graph.client_id().to_string();
            let map = get_or_insert_map(
                &id,
                &mut mut_transaction,
                &mut self.node_selections,
                MapPrelim::<String>::new(),
            )?;
            if selected {
                map.insert(&mut mut_transaction, node_id, true);
            } else {
                map.remove(&mut mut_transaction, node_id);
            }
        }
        // Important that emit is called here. Rather than before the drop
        // as this could trigger a panic as the store value may be read
        self.emit(false)?;

        Ok(())
    }

    pub fn replace_node(&mut self, node: &NewWbblWebappNode) -> Result<(), WbblWebappStoreError> {
        {
            let mut mut_transaction = self.graph.transact_mut_with(self.graph.client_id());
            node.encode(&mut mut_transaction, &mut self.nodes)
        }?;

        // Important that emit is called here. Rather than before the drop
        // as this could trigger a panic as the store value may be read
        self.emit(true)?;

        Ok(())
    }

    pub fn add_edge(
        &mut self,
        source: &str,
        target: &str,
        source_handle: i64,
        target_handle: i64,
    ) -> Result<(), WbblWebappStoreError> {
        {
            let source_uuid =
                uuid::Uuid::from_str(source).map_err(|_| WbblWebappStoreError::MalformedId)?;
            let target_uuid =
                uuid::Uuid::from_str(target).map_err(|_| WbblWebappStoreError::MalformedId)?;

            let edge = WbblWebappEdge::new(
                &(source_uuid.as_u128()),
                &(target_uuid.as_u128()),
                source_handle,
                target_handle,
            );

            let mut mut_transaction = self.graph.transact_mut_with(self.graph.client_id());
            edge.encode(&mut mut_transaction, &mut self.edges, &self.nodes)?;
        }

        // Important that emit is called here. Rather than before the drop
        // as this could trigger a panic as the store value may be read
        self.emit(true)?;

        Ok(())
    }

    fn get_selection_snapshot(&self) -> Result<WbblWebappGraphSnapshot, WbblWebappStoreError> {
        let selected_nodes = self.get_locally_selected_nodes()?;
        let selected_edges = self.get_locally_selected_edges()?;

        let mut nodes: Vec<WbblWebappNode> = Vec::new();
        let mut edges: Vec<WbblWebappEdge> = Vec::new();
        {
            let read_txn = self.graph.transact();
            for node_id in selected_nodes.iter() {
                let key = uuid::Uuid::from_str(&node_id)
                    .map_err(|_| WbblWebappStoreError::MalformedId)?;
                let node_values = get_map(node_id, &read_txn, &self.nodes)?;
                let node = WbblWebappNode::decode(
                    &key.as_u128(),
                    &read_txn,
                    &node_values,
                    &self.node_selections,
                    self.graph.client_id(),
                )?;
                nodes.push(node);
            }

            for edge_id in selected_edges.iter() {
                let key = uuid::Uuid::from_str(&edge_id)
                    .map_err(|_| WbblWebappStoreError::MalformedId)?;
                let edge_values = get_map(edge_id, &read_txn, &self.edges)?;
                let edge: WbblWebappEdge = WbblWebappEdge::decode(
                    key.as_u128(),
                    &read_txn,
                    &edge_values,
                    &self.edge_selections,
                    self.graph.client_id(),
                )?;
                edges.push(edge);
            }
        }
        nodes.sort_by_key(|n| n.id.clone());
        edges.sort_by_key(|e| e.id.clone());
        Ok(WbblWebappGraphSnapshot {
            id: self.id,
            nodes,
            edges,
            node_groups: None,
            computed_types: None,
        })
    }

    fn get_node_snapshot(
        &self,
        node_id: &str,
    ) -> Result<WbblWebappGraphSnapshot, WbblWebappStoreError> {
        let node_id_uuid = uuid::Uuid::parse_str(node_id)
            .map_err(|_| WbblWebappStoreError::MalformedId)?
            .as_u128();
        let txn = self.graph.transact();
        let node_map = get_map(node_id, &txn, &self.nodes)?;
        let node = WbblWebappNode::decode(
            &node_id_uuid,
            &txn,
            &node_map,
            &self.node_selections,
            self.graph.client_id(),
        )?;

        let mut snapshot = WbblWebappGraphSnapshot {
            id: self.id,
            nodes: vec![node],
            edges: vec![],
            computed_types: None,
            node_groups: None,
        };
        snapshot.edges = vec![];

        Ok(snapshot)
    }

    pub fn duplicate(&mut self) -> Result<(), WbblWebappStoreError> {
        let mut snapshot = self.get_selection_snapshot()?;
        snapshot.offset(&Vec2::new(200.0, 200.0));
        self.integrate_snapshot(None, &mut snapshot)?;
        Ok(())
    }

    pub fn duplicate_node(&mut self, node_id: &str) -> Result<(), WbblWebappStoreError> {
        let mut snapshot = self.get_node_snapshot(node_id)?;
        snapshot.offset(&Vec2::new(200.0, 200.0));
        self.integrate_snapshot(None, &mut snapshot)?;
        Ok(())
    }

    #[cfg(web_sys_unstable_apis)]
    pub fn copy_node(&mut self, node_id: &str) -> Result<js_sys::Promise, WbblWebappStoreError> {
        use crate::dot_converter::to_dot;
        let clipboard_contents: String = {
            let snapshot = self.get_node_snapshot(node_id)?;
            to_dot(&snapshot)
        };
        let window = web_sys::window().expect("Missing Window");
        if let Some(clipboard) = window.navigator().clipboard() {
            Ok(clipboard.write_text(&clipboard_contents))
        } else {
            Err(WbblWebappStoreError::ClipboardFailure)
        }
    }

    #[cfg(web_sys_unstable_apis)]
    pub fn copy(&self) -> Result<js_sys::Promise, WbblWebappStoreError> {
        use crate::dot_converter::to_dot;
        let snapshot = self.get_selection_snapshot()?;
        let clipboard_contents = to_dot(&snapshot);
        let window = web_sys::window().expect("Missing Window");
        if let Some(clipboard) = window.navigator().clipboard() {
            Ok(clipboard.write_text(&clipboard_contents))
        } else {
            Err(WbblWebappStoreError::ClipboardFailure)
        }
    }

    fn integrate_snapshot(
        &mut self,
        position: Option<Vec2>,
        snapshot: &mut WbblWebappGraphSnapshot,
    ) -> Result<(), WbblWebappStoreError> {
        {
            let mut mut_transaction = self.graph.transact_mut_with(self.graph.client_id());
            snapshot.reassign_ids();
            snapshot.filter_out_output_ports();
            if let Some(position) = position {
                snapshot.recenter(&position);
            }
            for node in snapshot.nodes.iter() {
                node.encode(&mut mut_transaction, &mut self.nodes, &mut self.node_groups)?;
            }
            for edge in snapshot.edges.iter() {
                edge.encode(&mut mut_transaction, &mut self.edges, &mut self.nodes)?;
            }
            if let Some(node_groups) = &snapshot.node_groups {
                for group in node_groups.iter() {
                    self.node_groups.insert(
                        &mut mut_transaction,
                        uuid::Uuid::from_u128(group.id).to_string(),
                        MapPrelim::<bool>::from(
                            group
                                .nodes
                                .iter()
                                .map(|n| (uuid::Uuid::from_u128(n.clone()).to_string(), true))
                                .collect::<HashMap<String, bool>>(),
                        ),
                    );
                }
            }
        }
        self.emit(true)?;
        Ok(())
    }

    pub fn group_selected_nodes(&mut self) -> Result<(), WbblWebappStoreError> {
        let selected_nodes = self.get_locally_selected_nodes()?;
        {
            let mut mut_transaction = self.graph.transact_mut_with(self.graph.client_id());
            let group_id = uuid::Uuid::new_v4().to_string();
            for node_id in selected_nodes.iter() {
                match get_map(&node_id, &mut_transaction, &self.nodes) {
                    Ok(node) => {
                        remove_node_from_group(
                            &mut mut_transaction,
                            node_id,
                            &node,
                            &mut self.node_groups,
                        )?;
                        node.insert(&mut mut_transaction, "group_id", group_id.clone());
                    }
                    Err(WbblWebappStoreError::NotFound) => {}
                    Err(err) => {
                        return Err(err);
                    }
                };
            }
            self.node_groups.insert(
                &mut mut_transaction,
                group_id,
                MapPrelim::from(
                    selected_nodes
                        .iter()
                        .map(|n| (n.to_owned(), true))
                        .collect::<HashMap<String, bool>>(),
                ),
            );
        }
        self.emit(false)?;
        Ok(())
    }

    pub fn ungroup_selected_nodes(&mut self) -> Result<(), WbblWebappStoreError> {
        let selected_nodes = self.get_locally_selected_nodes()?;
        {
            let mut mut_transaction = self.graph.transact_mut_with(self.graph.client_id());
            for node_id in selected_nodes.iter() {
                match get_map(&node_id, &mut_transaction, &self.nodes) {
                    Ok(node) => {
                        remove_node_from_group(
                            &mut mut_transaction,
                            node_id,
                            &node,
                            &mut self.node_groups,
                        )?;
                        node.remove(&mut mut_transaction, "group_id");
                    }
                    Err(WbblWebappStoreError::NotFound) => {}
                    Err(err) => {
                        return Err(err);
                    }
                };
            }
        }
        self.emit(false)?;
        Ok(())
    }

    #[cfg(web_sys_unstable_apis)]
    pub async fn get_clipboard_snapshot() -> Result<JsValue, WbblWebappStoreError> {
        use web_sys::js_sys::JsString;

        use crate::dot_converter::from_dot;

        let window = web_sys::window().expect("Missing Window");
        if let Some(clipboard) = window.navigator().clipboard() {
            let value = wasm_bindgen_futures::JsFuture::from(clipboard.read_text())
                .await
                .map_err(|_| WbblWebappStoreError::ClipboardFailure)?;
            let str: String = value.dyn_into::<JsString>().unwrap().into();
            let snapshot =
                from_dot(&str).map_err(|_| WbblWebappStoreError::ClipboardContentsFailure)?;
            let serialized_snapshot = serde_wasm_bindgen::to_value(&snapshot)
                .map_err(|_| WbblWebappStoreError::SerializationFailure)?;
            return Ok(serialized_snapshot);
        };
        Err(WbblWebappStoreError::ClipboardNotFound)
    }

    pub fn integrate_clipboard_snapshot(
        &mut self,
        value: JsValue,
        cursor_position: &[f32],
    ) -> Result<(), WbblWebappStoreError> {
        let mut snapshot: WbblWebappGraphSnapshot = serde_wasm_bindgen::from_value(value)
            .map_err(|_| WbblWebappStoreError::SerializationFailure)?;
        let position = Vec2::from_slice(cursor_position);
        self.integrate_snapshot(Some(position), &mut snapshot)?;
        Ok(())
    }

    pub fn set_edge_selection(
        &mut self,
        edge_id: &str,
        selected: bool,
    ) -> Result<(), WbblWebappStoreError> {
        {
            let mut mut_transaction = self.graph.transact_mut_with(self.graph.client_id());
            let id = self.graph.client_id().to_string();
            let map = get_or_insert_map(
                &id,
                &mut mut_transaction,
                &self.edge_selections,
                MapPrelim::<String>::new(),
            )?;
            if selected {
                map.insert(&mut mut_transaction, edge_id, true);
            } else {
                map.remove(&mut mut_transaction, edge_id);
            }
        }
        // Important that emit is called here. Rather than before the drop
        // as this could trigger a panic as the store value may be read
        self.emit(false)?;

        Ok(())
    }

    pub fn select_all(&mut self) -> Result<(), WbblWebappStoreError> {
        let id = self.graph.client_id().to_string();
        {
            let mut txn = self.graph.transact_mut_with(self.graph.client_id());
            let node_ids: HashMap<String, bool> = self
                .nodes
                .iter(&txn)
                .map(|(k, _)| (k.to_owned(), true))
                .collect();
            let edge_ids: HashMap<String, bool> = self
                .edges
                .iter(&txn)
                .map(|(k, _)| (k.to_owned(), true))
                .collect();
            self.edge_selections
                .insert(&mut txn, id.clone(), MapPrelim::from(edge_ids));
            self.node_selections
                .insert(&mut txn, id.clone(), MapPrelim::from(node_ids));
        }
        self.emit(false)?;
        Ok(())
    }

    pub fn select_none(&mut self) -> Result<(), WbblWebappStoreError> {
        let id = self.graph.client_id().to_string();
        {
            let mut txn = self.graph.transact_mut_with(self.graph.client_id());
            match get_map(&id, &txn, &self.node_selections) {
                Ok(selection) => {
                    selection.clear(&mut txn);
                    Ok(())
                }
                Err(WbblWebappStoreError::NotFound) => Ok(()),
                Err(err) => Err(err),
            }?;
            match get_map(&id, &txn, &self.edge_selections) {
                Ok(selection) => {
                    selection.clear(&mut txn);
                    Ok(())
                }
                Err(WbblWebappStoreError::NotFound) => Ok(()),
                Err(err) => Err(err),
            }?;
        }
        self.emit(false)?;
        Ok(())
    }

    pub fn get_locally_selected_nodes(&self) -> Result<Vec<String>, WbblWebappStoreError> {
        let txn = self.graph.transact();
        let id = self.graph.client_id().to_string();
        let maybe_selection_for_current_user = match get_map(&id, &txn, &self.node_selections) {
            Ok(selection) => Ok(Some(selection)),
            Err(WbblWebappStoreError::NotFound) => Ok(None),
            Err(err) => Err(err),
        }?;
        if maybe_selection_for_current_user.is_none() {
            return Ok(vec![]);
        }

        return Ok(maybe_selection_for_current_user
            .unwrap()
            .keys(&txn)
            .map(|x| x.to_owned())
            .collect());
    }

    pub fn get_locally_selected_edges(&self) -> Result<Vec<String>, WbblWebappStoreError> {
        let txn = self.graph.transact();
        let id = self.graph.client_id().to_string();
        let maybe_selection_for_current_user = match get_map(&id, &txn, &self.edge_selections) {
            Ok(selection) => Ok(Some(selection)),
            Err(WbblWebappStoreError::NotFound) => Ok(None),
            Err(err) => Err(err),
        }?;
        if maybe_selection_for_current_user.is_none() {
            return Ok(vec![]);
        }

        return Ok(maybe_selection_for_current_user
            .unwrap()
            .keys(&txn)
            .map(|x| x.to_owned())
            .collect());
    }

    pub fn are_port_types_compatible(type_a: JsValue, type_b: JsValue) -> bool {
        match (
            serde_wasm_bindgen::from_value::<AbstractDataType>(type_a),
            serde_wasm_bindgen::from_value::<AbstractDataType>(type_b),
        ) {
            (Ok(a), Ok(b)) => AbstractDataType::are_types_compatible(a, b),
            (Ok(_), Err(_)) => false,
            (Err(_), Ok(_)) => false,
            (Err(_), Err(_)) => false,
        }
    }

    pub fn get_edge_type(
        type_a: JsValue,
        type_b: JsValue,
    ) -> Result<JsValue, WbblWebappStoreError> {
        match (
            serde_wasm_bindgen::from_value::<AbstractDataType>(type_a),
            serde_wasm_bindgen::from_value::<AbstractDataType>(type_b),
        ) {
            (Ok(a), Ok(b)) => Ok(serde_wasm_bindgen::to_value(
                &AbstractDataType::get_most_specific_type(a, b),
            )
            .unwrap()),
            (Ok(_), Err(_)) => Err(WbblWebappStoreError::UnexpectedStructure),
            (Err(_), Ok(_)) => Err(WbblWebappStoreError::UnexpectedStructure),
            (Err(_), Err(_)) => Err(WbblWebappStoreError::UnexpectedStructure),
        }
    }

    pub fn link_to_preview(&mut self, node_id: &str) -> Result<(), WbblWebappStoreError> {
        {
            // TODO Add validation for this
            let mut_transaction = &mut self.graph.transact_mut_with(self.graph.client_id());
            let position = match self.nodes.get(mut_transaction, node_id) {
                Some(yrs::Value::YMap(node_ref)) => {
                    match (
                        node_ref.get(mut_transaction, "x"),
                        node_ref.get(mut_transaction, "y"),
                    ) {
                        (
                            Some(yrs::Value::Any(yrs::Any::Number(x))),
                            Some(yrs::Value::Any(yrs::Any::Number(y))),
                        ) => Ok((x, y)),
                        (_, _) => Err(WbblWebappStoreError::UnexpectedStructure),
                    }
                }
                _ => Err(WbblWebappStoreError::UnexpectedStructure),
            }?;
            let preview_node = NewWbblWebappNode::new(
                position.0 + 350.0,
                position.1,
                WbblWebappNodeType::Preview,
            )?;
            preview_node.encode(mut_transaction, &mut self.nodes)?;
            let source =
                uuid::Uuid::from_str(node_id).map_err(|_| WbblWebappStoreError::MalformedId)?;
            let edge = WbblWebappEdge::new(&(source.as_u128()), &preview_node.id, 0, 0);
            edge.encode(mut_transaction, &mut self.edges, &self.nodes)?;
        }
        self.emit(true)?;
        Ok(())
    }

    pub fn make_junction(
        &mut self,
        edge_id: &str,
        position_x: f64,
        position_y: f64,
    ) -> Result<(), WbblWebappStoreError> {
        {
            let uuid =
                uuid::Uuid::parse_str(edge_id).map_err(|_| WbblWebappStoreError::MalformedId)?;
            let mut txn = self.graph.transact_mut_with(self.graph.client_id());
            let edge = get_map(edge_id, &txn, &self.edges)?;
            let edge = WbblWebappEdge::decode(
                uuid.as_u128(),
                &txn,
                &edge,
                &self.edge_selections,
                self.graph.client_id(),
            )?;
            let new_node =
                NewWbblWebappNode::new(position_x, position_y, WbblWebappNodeType::Junction)?;
            new_node.encode(&mut txn, &mut self.nodes)?;
            let edge_1 = WbblWebappEdge::new(&edge.source, &new_node.id, edge.source_handle, 0);
            let edge_2 = WbblWebappEdge::new(&new_node.id, &edge.target, 0, edge.target_handle);
            edge_1.encode(&mut txn, &mut self.edges, &self.nodes)?;
            edge_2.encode(&mut txn, &mut self.edges, &self.nodes)?;
            delete_edge(
                &mut txn,
                edge_id,
                &mut self.edges,
                &mut self.nodes,
                &mut self.edge_selections,
            )?;
        }
        self.emit(true)?;
        Ok(())
    }
}

impl Drop for WbblWebappGraphStore {
    fn drop(&mut self) {
        self.graph_worker
            .remove_event_listener_with_callback(
                "message",
                self.worker_responder.as_ref().unchecked_ref(),
            )
            .unwrap();
    }
}
