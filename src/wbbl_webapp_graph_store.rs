use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
    str::FromStr,
    sync::Arc,
    vec,
};

use glam::Vec2;
use graphviz_rust::{attributes::group, dot_generator::edge};
use rstar::RTree;
use wasm_bindgen::prelude::*;
use web_sys::{js_sys, MessageEvent, Worker};
use yrs::{
    types::ToJson, DeepObservable, Map, MapPrelim, MapRef, ReadTxn, Transact, TransactionMut, Value,
};

use crate::{
    convex_hull::{get_convex_hull, get_ray_ray_intersection},
    data_types::AbstractDataType,
    graph_transfer_types::{
        from_type_name, get_type_name, Any, WbblWebappEdge, WbblWebappGraphEntity,
        WbblWebappGraphEntityId, WbblWebappGraphSnapshot, WbblWebappNode, WbblWebappNodeGroup,
        WbblWebappNodeType, WbbleComputedNodeSize, WbblePosition,
    },
    graph_types::PortId,
    log,
    store_errors::WbblWebappStoreError,
    wbbl_graph_web_worker::WbblGraphWebWorkerResponseMessage,
    yrs_utils::*,
};

const GRAPH_YRS_NODE_GROUPS_MAP_KEY: &str = "node_groups";
const GRAPH_YRS_NODE_GROUP_SELECTIONS_MAP_KEY: &str = "node_group_selections";

const GRAPH_YRS_NODES_MAP_KEY: &str = "nodes";
const GRAPH_YRS_EDGES_MAP_KEY: &str = "edges";

#[wasm_bindgen]
#[allow(unused)]
pub struct WbblWebappGraphStore {
    id: u128,
    next_listener_handle: u32,
    listeners: Arc<RefCell<Vec<(u32, js_sys::Function)>>>,
    undo_manager: yrs::UndoManager,
    graph: Arc<yrs::Doc>,
    nodes: Arc<yrs::MapRef>,
    node_group_selections: Arc<yrs::MapRef>,
    edges: Arc<yrs::MapRef>,
    graph_worker: Worker,
    worker_responder: Closure<dyn FnMut(MessageEvent) -> ()>,
    computed_types: Arc<RefCell<HashMap<PortId, AbstractDataType>>>,
    spatial_index: Arc<RefCell<RTree<WbblWebappGraphEntity>>>,
    entities: Arc<RefCell<HashMap<WbblWebappGraphEntityId, WbblWebappGraphEntity>>>,
    subscriptions: Vec<yrs::Subscription>,
    locally_selected_entities: Arc<RefCell<HashSet<WbblWebappGraphEntityId>>>,
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct NewWbblWebappNode {
    id: u128,
    position: WbblePosition,
    node_type: WbblWebappNodeType,
    data: HashMap<String, Any>,
}

fn try_into_u128(value: &str) -> Result<u128, WbblWebappStoreError> {
    return uuid::Uuid::from_str(value)
        .map_err(|_| WbblWebappStoreError::MalformedId)
        .map(|x| x.as_u128());
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

fn encode_selection(
    txn: &mut TransactionMut,
    id: u128,
    root_map: &MapRef,
    client_id: u64,
    selected: bool,
) -> Result<(), WbblWebappStoreError> {
    let entity = get_map(&uuid::Uuid::from_u128(id).to_string(), txn, root_map)?;
    let selections = get_map("selections", txn, &entity)?;
    if selected {
        selections.remove(txn, &client_id.to_string());
    } else {
        selections.insert(txn, client_id.to_string(), true);
    }
    Ok(())
}

impl NewWbblWebappNode {
    pub(crate) fn encode(
        &self,
        transaction: &mut TransactionMut,
        nodes: &mut yrs::MapRef,
        client_id: u64,
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
            node_ref.insert(
                transaction,
                "selections",
                MapPrelim::from(HashMap::from([(client_id.to_string(), false)])),
            );
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
            selections: vec![],
            selectable: true,
            deletable: self.node_type != WbblWebappNodeType::Output,
            connectable: true,
            group_id: None,
            in_edges: vec![],
            out_edges: vec![],
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

fn get_mutual_edges<Txn: ReadTxn>(
    txn: &Txn,
    node_ids: &Vec<u128>,
    nodes_map: &yrs::MapRef,
) -> Result<Vec<u128>, WbblWebappStoreError> {
    let mut mutual_edges: Vec<u128> = vec![];
    let mut edges_map: HashSet<u128> = HashSet::new();
    for node_id in node_ids.iter() {
        let node = get_map(&uuid::Uuid::from_u128(*node_id).to_string(), txn, nodes_map)?;
        let in_edge_ids = get_map("in_edges", txn, &node)?.keys(txn).try_fold(
            HashSet::new(),
            |mut prev, key| {
                let id = uuid::Uuid::parse_str(key)
                    .map_err(|_| WbblWebappStoreError::MalformedId)?
                    .as_u128();
                prev.insert(id);
                Ok(prev)
            },
        )?;
        let out_edge_ids = get_map("out_edges", txn, &node)?.keys(txn).try_fold(
            HashSet::new(),
            |mut prev, key| {
                let id = uuid::Uuid::parse_str(key)
                    .map_err(|_| WbblWebappStoreError::MalformedId)?
                    .as_u128();
                prev.insert(id);
                Ok(prev)
            },
        )?;
        let mut in_intersection: Vec<u128> =
            edges_map.intersection(&in_edge_ids).cloned().collect();
        let mut out_intersection: Vec<u128> =
            edges_map.intersection(&out_edge_ids).cloned().collect();
        mutual_edges.append(&mut in_intersection);
        mutual_edges.append(&mut out_intersection);
        edges_map = edges_map.union(&in_edge_ids).cloned().collect();
        edges_map = edges_map.union(&out_edge_ids).cloned().collect();
    }
    Ok(mutual_edges)
}

impl WbblWebappNode {
    pub(crate) fn decode<Txn: yrs::ReadTxn>(
        key: &u128,
        txn: &Txn,
        map: &yrs::MapRef,
        client_id: u64,
    ) -> Result<WbblWebappNode, WbblWebappStoreError> {
        let type_name: String = get_atomic_string("type", txn, map)?;
        let data = get_map("data", txn, map)?;
        let x = get_float_64("x", txn, map)?;
        let y = get_float_64("y", txn, map)?;
        let dragging = get_bool("dragging", txn, map)?;
        let resizing = get_bool("resizing", txn, map)?;
        let selections = get_map("selections", txn, map)?;
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
        let selected = selections.contains_key(txn, &client_id.to_string());
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
            selections: selections.keys(txn).map(|x| x.to_string()).collect(),
            deletable: node_type != WbblWebappNodeType::Output,
            group_id,
            in_edges: vec![],
            out_edges: vec![],
        })
    }

    pub(crate) fn encode(
        &self,
        transaction: &mut TransactionMut,
        nodes: &mut yrs::MapRef,
        client_id: u64,
    ) -> Result<(), WbblWebappStoreError> {
        let prelim_map: yrs::MapPrelim<yrs::Any> = yrs::MapPrelim::new();
        let node_id = uuid::Uuid::from_u128(self.id).to_string();
        let mut node_ref = nodes.insert(transaction, node_id.clone(), prelim_map);
        node_ref.insert(transaction, "type", get_type_name(self.node_type));
        node_ref.insert(transaction, "x", self.position.x);
        let prelim_selections: MapPrelim<bool> = yrs::MapPrelim::new();
        node_ref.insert(transaction, "selections", prelim_selections);
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
        }
        encode_data(&self.data, transaction, &mut node_ref)?;
        Ok(())
    }
}

fn delete_edge(
    transaction: &mut TransactionMut,
    edge_id: u128,
    edges: &MapRef,
) -> Result<(), WbblWebappStoreError> {
    let id_str = uuid::Uuid::from_u128(edge_id.clone()).to_string();
    edges.remove(transaction, &id_str);
    Ok(())
}

fn delete_node_group_and_associated_nodes_and_edges(
    txn: &mut TransactionMut,
    group_id: u128,
    nodes: &MapRef,
    edges: &MapRef,
    entities: &HashMap<WbblWebappGraphEntityId, WbblWebappGraphEntity>,
) -> Result<(), WbblWebappStoreError> {
    if let Some(WbblWebappGraphEntity::Group(group)) =
        entities.get(&WbblWebappGraphEntityId::GroupId(group_id))
    {
        for node_id in group.nodes {
            let node_id_str = &uuid::Uuid::from_u128(node_id).to_string();
            match get_map(&node_id_str, &mut_transaction, &self.nodes) {
                Ok(node) => {
                    let type_name: String = get_atomic_string("type", &mut_transaction, &node)?;
                    if let Some(WbblWebappNodeType::Output) = from_type_name(&type_name) {
                        Ok(())
                    } else {
                        delete_node_and_associated_edges(txn, node_id, nodes, edges, entities)?;
                        Ok(())
                    }
                }
                Err(WbblWebappStoreError::NotFound) => Ok(()),
                Err(err) => Err(err),
            }?;
        }
        Ok(())
    } else {
        Err(WbblWebappStoreError::NotFound)
    }
}

fn clear_local_selections(
    txn: &mut TransactionMut,
    client_id: u64,
    nodes: &MapRef,
    edges: &MapRef,
    node_groups_selections: &MapRef,
    locally_selected_entities: &HashSet<WbblWebappGraphEntityId>,
) -> Result<(), WbblWebappStoreError> {
    {
        let local_group_seletions = get_map(&client_id.to_string(), txn, node_groups_selections)?;
        local_group_seletions.clear(txn);
        for entity_id in locally_selected_entities.iter() {
            match entity_id {
                WbblWebappGraphEntityId::NodeId(node_id) => {
                    encode_selection(txn, node_id.clone(), nodes, client_id.clone(), false);
                }
                WbblWebappGraphEntityId::EdgeId(edge_id) => {
                    encode_selection(txn, edge_id.clone(), edges, client_id.clone(), false);
                }
                WbblWebappGraphEntityId::GroupId(group_id) => {}
            }
        }
        Ok(())
    }
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
) -> Result<(String, Vec<Vec2>), WbblWebappStoreError> {
    let convex_hull =
        get_group_convex_hull(txn, group_id, nodes, node_groups, computed_node_sizes)?;
    let mut inflated_convex_hull = vec![];
    if convex_hull.len() > 2 {
        let first = convex_hull[0];
        let second = convex_hull[1];
        let delta_first_second = second - first;
        let first_second_tangent = Vec2::new(-delta_first_second.y, delta_first_second.x)
            .normalize()
            * INFLATE_GROUP_PATH_BY;
        let first_1 = first_second_tangent + first;
        inflated_convex_hull.push(first_1);
        let mut result = format!("M {} {}", first_1.x, first_1.y);
        let mut i = 1;
        while i <= convex_hull.len() {
            let prev = convex_hull[(i - 1) % convex_hull.len()];
            let current = convex_hull[i % convex_hull.len()];
            let next = convex_hull[(i + 1) % convex_hull.len()];
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
            let intersection = get_ray_ray_intersection(&prev, &current_1, &current_2, &next);
            result.push_str(&format!("L {} {}", current_1.x, current_1.y));
            if let Some(intersection) = intersection {
                inflated_convex_hull.push(intersection);
                result.push_str(&format!(
                    "Q {} {}, {} {}",
                    intersection.x, intersection.y, current_2.x, current_2.y
                ));
            }
            i += 1;
        }
        result.push('Z');
        return Ok((result, inflated_convex_hull));
    }
    return Ok(("".to_owned(), vec![]));
}

impl WbblWebappEdge {
    pub(crate) fn decode<Txn: yrs::ReadTxn>(
        key: u128,
        txn: &Txn,
        map: &yrs::MapRef,
        client_id: u64,
    ) -> Result<WbblWebappEdge, WbblWebappStoreError> {
        let source = get_atomic_u128_from_string("source", txn, map)?;
        let target = get_atomic_u128_from_string("target", txn, map)?;
        let source_handle = get_atomic_bigint("source_handle", txn, map)?;
        let target_handle = get_atomic_bigint("target_handle", txn, map)?;
        let key_str = uuid::Uuid::from_u128(key).to_string();
        let selections: MapRef = get_map("selections", txn, map)?;
        let selected = selections.contains_key(txn, &client_id.to_string());
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

    pub(crate) fn encode(
        &self,
        txn: &mut TransactionMut,
        edges: &mut yrs::MapRef,
        client_id: u64,
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
        edge_ref.insert(txn, "selections".to_owned(), yrs::MapPrelim::<bool>::new());
        Ok(())
    }
}

fn delete_node_and_associated_edges(
    txn: &mut TransactionMut,
    node_id: u128,
    nodes: &MapRef,
    edges: &MapRef,
    entities: &HashMap<WbblWebappGraphEntityId, WbblWebappGraphEntity>,
) -> Result<(), WbblWebappStoreError> {
    let node: WbblWebappNode = match entities.get(&WbblWebappGraphEntityId::NodeId(node_id)) {
        Some(entity) => entity.clone().try_into(),
        None => Err(WbblWebappStoreError::NotFound),
    }?;
    for edge in node.in_edges {
        delete_edge(txn, edge, edges);
    }
    for edge in node.out_edges {
        delete_edge(txn, edge, edges);
    }
    nodes.remove(txn, &uuid::Uuid::from_u128(node_id).to_string());
    Ok(())
}

#[wasm_bindgen]
impl WbblWebappGraphStore {
    pub fn empty(graph_worker: Worker) -> Self {
        let graph = yrs::Doc::new();
        let node_groups: Arc<MapRef> =
            Arc::new(graph.get_or_insert_map(GRAPH_YRS_NODE_GROUPS_MAP_KEY.to_owned()));
        let nodes = Arc::new(graph.get_or_insert_map(GRAPH_YRS_NODES_MAP_KEY.to_owned()));
        let edges = Arc::new(graph.get_or_insert_map(GRAPH_YRS_EDGES_MAP_KEY.to_owned()));
        let node_group_selections =
            Arc::new(graph.get_or_insert_map(GRAPH_YRS_NODE_GROUP_SELECTIONS_MAP_KEY));
        let undo_manager = yrs::UndoManager::with_options(
            &graph,
            nodes.as_ref(),
            yrs::undo::Options {
                capture_timeout_millis: 1_000,
                tracked_origins: HashSet::new(),
                capture_transaction: Rc::new(|_txn| true),
                timestamp: Rc::new(|| js_sys::Date::now() as u64),
            },
        );

        let graph = Arc::new(graph);

        let computed_types = Arc::new(RefCell::new(HashMap::new()));
        let locally_selected_entities: Arc<RefCell<HashSet<WbblWebappGraphEntityId>>> =
            Arc::new(RefCell::new(HashSet::new()));
        let listeners = Arc::new(RefCell::new(Vec::<(u32, js_sys::Function)>::new()));
        let worker_responder = Closure::<dyn FnMut(MessageEvent) -> ()>::new({
            let computed_types = computed_types.clone();
            let listeners: Arc<RefCell<Vec<(u32, js_sys::Function)>>> = listeners.clone();
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

        let computed_node_sizes: Arc<RefCell<HashMap<u128, WbbleComputedNodeSize>>> =
            Arc::new(RefCell::new(HashMap::new()));
        let spatial_index = Arc::new(RefCell::new(RTree::new()));
        let entities: Arc<RefCell<HashMap<WbblWebappGraphEntityId, WbblWebappGraphEntity>>> =
            Arc::new(RefCell::new(HashMap::new()));

        let nodes_subscription = nodes.observe_deep({
            let spatial_index = spatial_index.clone();
            let entities = entities.clone();
            let computed_node_sizes = computed_node_sizes.clone();
            let listeners = listeners.clone();
            let graph: Arc<yrs::Doc> = graph.clone();
            move |mut txn, evts| {
                for evt in evts.iter() {
                    let path = evt.path();
                    if path.len() == 0 {
                        // Signals an addition/removal from the graph store
                        if let yrs::types::Event::Map(mapEvt) = evt {
                            for (key, change) in mapEvt.keys(&mut txn).iter() {
                                if let Ok(node_id) =
                                    uuid::Uuid::parse_str(&key).map(|id| id.as_u128())
                                {
                                    match change {
                                        yrs::types::EntryChange::Inserted(yrs::Value::YMap(
                                            node,
                                        )) => {
                                            let client_id = graph.client_id();
                                            if let Ok(node) = WbblWebappNode::decode(
                                                &node_id, txn, node, client_id,
                                            ) {
                                                entities.borrow_mut().insert(
                                                    WbblWebappGraphEntityId::NodeId(node_id),
                                                    WbblWebappGraphEntity::Node(node),
                                                );
                                            }
                                        }
                                        yrs::types::EntryChange::Removed(_) => todo!(),
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }
                }
                // TODO: Update modified nodes
                // TODO: Update edges for each modified Node if moved/resized
                // TODO: Update node groups for each modified node if moved/resized

                // Emitting change to listeners. Do this at end as we need to first update our
                // domain representation
                for (_, listener) in listeners.borrow().iter() {
                    let _ = listener
                        .call0(&JsValue::UNDEFINED)
                        .inspect_err(|err| log!("Publish error: {:?}", err));
                }
            }
        });
        let edges_subscription = edges.observe_deep({
            let spatial_index = spatial_index.clone();
            let listeners: Arc<RefCell<Vec<(u32, js_sys::Function)>>> = listeners.clone();
            move |txn: &TransactionMut<'_>, evts| {
                // TODO: Update modified edges
                // TODO: Update nodes if edges changed

                // Emitting change to listeners. Do this at end as we need to first update our
                // domain representation
                for (_, listener) in listeners.borrow().iter() {
                    let _ = listener
                        .call0(&JsValue::UNDEFINED)
                        .inspect_err(|err| log!("Publish error: {:?}", err));
                }
            }
        });

        let node_group_selections_subscription = node_group_selections.observe_deep({
            let listeners: Arc<RefCell<Vec<(u32, js_sys::Function)>>> = listeners.clone();
            move |txn, evts| {
                // TODO: Update node group selection state if modified
                // Emitting change to listeners. Do this at end as we need to first update our
                // domain representation
                for (_, listener) in listeners.borrow().iter() {
                    let _ = listener
                        .call0(&JsValue::UNDEFINED)
                        .inspect_err(|err| log!("Publish error: {:?}", err));
                }
            }
        });

        let subscriptions = vec![
            nodes_subscription,
            edges_subscription,
            node_group_selections_subscription,
        ];

        let mut store = WbblWebappGraphStore {
            id: uuid::Uuid::new_v4().as_u128(),
            next_listener_handle: 0,
            listeners: listeners.clone(),
            undo_manager,
            graph: graph.clone(),
            nodes,
            edges,
            node_group_selections,
            computed_types: computed_types.clone(),
            locally_selected_entities,
            graph_worker: graph_worker.clone(),
            worker_responder,
            spatial_index,
            entities,
            subscriptions,
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

            let local_node_group_selections = store.node_group_selections.insert(
                &mut txn_mut,
                store.graph.client_id().to_string(),
                MapPrelim::<String>::new(),
            );
            store
                .undo_manager
                .expand_scope(&local_node_group_selections);
        }
        store.undo_manager.include_origin(store.graph.client_id()); // only track changes originating from local peer
        store.undo_manager.expand_scope(&store.edges.as_ref());

        store
    }

    // pub fn emit(&self, should_publish_to_worker: bool) -> Result<(), WbblWebappStoreError> {
    //     for (_, listener) in self.listeners.borrow().iter() {
    //         listener
    //             .call0(&JsValue::UNDEFINED)
    //             .map_err(|_| WbblWebappStoreError::FailedToEmit)?;
    //     }
    //     if should_publish_to_worker && self.initialized {
    //         let snapshot = self.get_snapshot_raw()?;
    //         let snapshot_js_value = serde_wasm_bindgen::to_value(
    //             &WbblGraphWebWorkerRequestMessage::SetSnapshot(snapshot),
    //         )
    //         .map_err(|_| WbblWebappStoreError::UnexpectedStructure)?;

    //         self.graph_worker.post_message(&snapshot_js_value).unwrap();
    //     }

    //     Ok(())
    // }

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

        Ok(result)
    }

    pub fn redo(&mut self) -> Result<bool, WbblWebappStoreError> {
        let result = self
            .undo_manager
            .redo()
            .map_err(|_| WbblWebappStoreError::FailedToRedo)?;
        Ok(result)
    }

    pub fn can_undo(&self) -> bool {
        self.undo_manager.can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.undo_manager.can_redo()
    }

    pub fn get_snapshot(&mut self) -> Result<JsValue, WbblWebappStoreError> {
        let entities = self.entities.borrow();
        let nodes: Vec<WbblWebappNode> = entities
            .values()
            .filter_map(|n| match n {
                WbblWebappGraphEntity::Node(n) => Some(n),
                _ => None,
            })
            .cloned()
            .collect();
        let edges: Vec<WbblWebappEdge> = entities
            .values()
            .filter_map(|e| match e {
                WbblWebappGraphEntity::Edge(e) => Some(e),
                _ => None,
            })
            .cloned()
            .collect();

        let node_groups: Vec<WbblWebappNodeGroup> = entities
            .values()
            .filter_map(|g| match g {
                WbblWebappGraphEntity::Group(g) => Some(g),
                _ => None,
            })
            .cloned()
            .collect();

        let snapshot = WbblWebappGraphSnapshot {
            id: self.id,
            nodes,
            edges,
            node_groups,
            computed_types: self.computed_types.borrow().clone(),
        };
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
            delete_node(&mut mut_transaction, node_id, &mut self.nodes)?;
        }
        Ok(())
    }

    pub fn remove_selected_entities(&mut self) -> Result<(), WbblWebappStoreError> {
        {
            let mut mut_transaction = self.graph.transact_mut_with(self.graph.client_id());

            for entity in self.locally_selected_entities.borrow().iter() {
                match entity {
                    WbblWebappGraphEntityId::NodeId(node_id) => delete_node_and_associated_edges(
                        &mut mut_transaction,
                        *node_id,
                        &self.nodes,
                        &self.edges,
                        &self.entities.borrow(),
                    ),
                    WbblWebappGraphEntityId::EdgeId(edge_id) => {
                        delete_edge(&mut mut_transaction, *edge_id, &self.edges)
                    }
                    WbblWebappGraphEntityId::GroupId(_) => Ok(()),
                }?;
            }
        }
        Ok(())
    }

    pub fn remove_node_group_and_contents(
        &mut self,
        group_id: &str,
    ) -> Result<(), WbblWebappStoreError> {
        {
            let mut mut_transaction = self.graph.transact_mut_with(self.graph.client_id());
            let group_id_uuid = uuid::Uuid::parse_str(group_id)
                .map_err(|_| WbblWebappStoreError::MalformedId)?
                .as_u128();
        }
        Ok(())
    }

    pub fn remove_edge(&mut self, edge_id: &str) -> Result<(), WbblWebappStoreError> {
        {
            let edge_id = try_into_u128(edge_id)?;
            let mut mut_transaction = self.graph.transact_mut_with(self.graph.client_id());
            delete_edge(&mut mut_transaction, edge_id, &mut self.edges)?;
        }
        Ok(())
    }

    pub fn add_node(&mut self, node: NewWbblWebappNode) -> Result<(), WbblWebappStoreError> {
        {
            let mut mut_transaction = self.graph.transact_mut_with(self.graph.client_id());
            node.encode(
                &mut mut_transaction,
                &mut self.nodes,
                self.graph.client_id(),
            )
        }?;
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
            let node_ref = get_map(node_id, &mut_transaction, &self.nodes)?;
            node_ref.insert(&mut mut_transaction, "x", yrs::Any::Number(x));
            node_ref.insert(&mut mut_transaction, "y", yrs::Any::Number(y));
            node_ref.insert(
                &mut mut_transaction,
                "dragging",
                yrs::Any::Bool(dragging.unwrap_or(false)),
            );
        }
        Ok(())
    }

    pub fn set_node_selection(
        &mut self,
        node_id: &str,
        selected: bool,
    ) -> Result<(), WbblWebappStoreError> {
        {
            let mut mut_transaction: TransactionMut<'_> =
                self.graph.transact_mut_with(self.graph.client_id());
            let id = self.graph.client_id();
            let node_id = try_into_u128(node_id)?;
            encode_selection(&mut mut_transaction, node_id, &self.nodes, id, selected);
        }
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
            edge.encode(
                &mut mut_transaction,
                &mut self.edges,
                self.graph.client_id(),
            )?;
        }
        Ok(())
    }

    fn get_selection_snapshot(&self) -> Result<WbblWebappGraphSnapshot, WbblWebappStoreError> {
        let mut nodes: Vec<WbblWebappNode> = Vec::new();
        let mut edges: Vec<WbblWebappEdge> = Vec::new();
        for entity_id in self.locally_selected_entities.borrow().iter() {
            let entity = match self.entities.borrow().get(entity_id) {
                Some(entity) => Ok(entity),
                None => Err(WbblWebappStoreError::NotFound),
            }?;

            match entity {
                WbblWebappGraphEntity::Node(node) => {
                    nodes.push(node.clone());
                }
                WbblWebappGraphEntity::Edge(edge) => {
                    edges.push(edge.clone());
                }
                WbblWebappGraphEntity::Group(_) => {}
            };
        }
        nodes.sort_by_key(|n| n.id.clone());
        edges.sort_by_key(|e| e.id.clone());
        Ok(WbblWebappGraphSnapshot {
            id: self.id,
            nodes,
            edges,
            node_groups: vec![],
            computed_types: HashMap::new(),
        })
    }

    fn get_group_snapshot(
        &self,
        group_id: &str,
    ) -> Result<WbblWebappGraphSnapshot, WbblWebappStoreError> {
        let mut nodes: Vec<WbblWebappNode> = Vec::new();
        let mut edges: Vec<WbblWebappEdge> = Vec::new();
        let id = uuid::Uuid::from_str(group_id)
            .map_err(|_| WbblWebappStoreError::MalformedId)?
            .as_u128();
        let group = self
            .entities
            .borrow()
            .get(&WbblWebappGraphEntityId::GroupId(id));
        if group.is_none() {
            return Err(WbblWebappStoreError::NotFound);
        }
        if let WbblWebappGraphEntity::Group(group) = group.unwrap() {
            {
                let txn = self.graph.transact();
                let node_ids = group.nodes;

                let edge_ids = get_mutual_edges(&txn, &node_ids, &self.nodes)?;
                for node_id in group.nodes.iter() {
                    if let Some(WbblWebappGraphEntity::Node(node)) = self
                        .entities
                        .borrow()
                        .get(&WbblWebappGraphEntityId::NodeId(*node_id))
                    {
                        nodes.push(node.clone());
                    }
                }

                for edge_id in edge_ids.iter() {
                    if let Some(WbblWebappGraphEntity::Edge(edge)) = self
                        .entities
                        .borrow()
                        .get(&WbblWebappGraphEntityId::EdgeId(*edge_id))
                    {
                        edges.push(edge.clone());
                    }
                }
            }
            nodes.sort_by_key(|n| n.id.clone());
            edges.sort_by_key(|e| e.id.clone());
            Ok(WbblWebappGraphSnapshot {
                id: self.id,
                nodes,
                edges,
                node_groups: vec![],
                computed_types: HashMap::new(),
            })
        } else {
            return Err(WbblWebappStoreError::UnexpectedStructure);
        }
    }

    fn get_node_snapshot(
        &self,
        node_id: &str,
    ) -> Result<WbblWebappGraphSnapshot, WbblWebappStoreError> {
        let node_id_uuid = uuid::Uuid::parse_str(node_id)
            .map_err(|_| WbblWebappStoreError::MalformedId)?
            .as_u128();
        let nodes = if let Some(WbblWebappGraphEntity::Node(node)) = self
            .entities
            .borrow()
            .get(&WbblWebappGraphEntityId::NodeId(node_id_uuid))
        {
            vec![node.clone()]
        } else {
            vec![]
        };

        let snapshot = WbblWebappGraphSnapshot {
            id: self.id,
            nodes: nodes,
            edges: vec![],
            computed_types: HashMap::new(),
            node_groups: vec![],
        };
        Ok(snapshot)
    }

    pub fn duplicate(&mut self) -> Result<(), WbblWebappStoreError> {
        let mut snapshot = self.get_selection_snapshot()?;
        snapshot.offset(&Vec2::new(200.0, 200.0));
        self.integrate_snapshot(None, &mut snapshot)?;
        Ok(())
    }

    pub fn duplicate_group(&mut self, group_id: &str) -> Result<(), WbblWebappStoreError> {
        let mut snapshot = self.get_group_snapshot(group_id)?;
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
    pub fn copy_group(&mut self, group_id: &str) -> Result<js_sys::Promise, WbblWebappStoreError> {
        use crate::dot_converter::to_dot;
        let clipboard_contents: String = {
            let snapshot = self.get_group_snapshot(group_id)?;
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
            let client_id = self.graph.client_id();
            if let Some(position) = position {
                snapshot.recenter(&position);
            }
            for node in snapshot.nodes.iter() {
                node.encode(&mut mut_transaction, &mut self.nodes, client_id.clone())?;
            }
            for edge in snapshot.edges.iter() {
                edge.encode(&mut mut_transaction, &mut self.edges, client_id.clone())?;
            }
        }
        Ok(())
    }

    pub fn group_selected_nodes(&mut self) -> Result<(), WbblWebappStoreError> {
        let selected_nodes = self.get_locally_selected_nodes();
        {
            let mut mut_transaction = self.graph.transact_mut_with(self.graph.client_id());
            let group_id = uuid::Uuid::new_v4().to_string();
            for node_id in selected_nodes.iter() {
                match get_map(&node_id, &mut_transaction, &self.nodes) {
                    Ok(node) => {
                        node.insert(&mut mut_transaction, "group_id", group_id.clone());
                    }
                    Err(WbblWebappStoreError::NotFound) => {}
                    Err(err) => {
                        return Err(err);
                    }
                };
            }
        }
        Ok(())
    }

    pub fn ungroup_selected_nodes(&mut self) -> Result<(), WbblWebappStoreError> {
        let selected_nodes = self.get_locally_selected_nodes();
        {
            let mut mut_transaction = self.graph.transact_mut_with(self.graph.client_id());
            for node_id in selected_nodes.iter() {
                match get_map(&node_id, &mut_transaction, &self.nodes) {
                    Ok(node) => {
                        node.remove(&mut mut_transaction, "group_id");
                    }
                    Err(WbblWebappStoreError::NotFound) => {}
                    Err(err) => {
                        return Err(err);
                    }
                };
            }
        }
        Ok(())
    }

    pub fn ungroup(&mut self, group_id: &str) -> Result<(), WbblWebappStoreError> {
        let group_uuid = try_into_u128(group_id)?;
        let group: &WbblWebappNodeGroup = match self
            .entities
            .borrow()
            .get(&WbblWebappGraphEntityId::GroupId(group_uuid))
        {
            Some(WbblWebappGraphEntity::Group(group)) => Ok(group),
            None => Err(WbblWebappStoreError::NotFound),
            _ => Err(WbblWebappStoreError::MalformedId),
        }?;
        let nodes_in_group: Vec<u128> = group.nodes.clone();
        {
            let mut mut_transaction = self.graph.transact_mut_with(self.graph.client_id());
            for node_id in nodes_in_group {
                match get_map(&node_id.to_string(), &mut_transaction, &self.nodes) {
                    Ok(node) => {
                        node.remove(&mut mut_transaction, "group_id");
                    }
                    Err(WbblWebappStoreError::NotFound) => {}
                    Err(err) => {
                        return Err(err);
                    }
                };
            }
        }
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
            let edge = get_map(edge_id, &mut_transaction, &self.edges)?;
            let selections = get_map("selections", &mut_transaction, &edge)?;
            if selected {
                selections.insert(&mut mut_transaction, id, true);
            } else {
                selections.remove(&mut mut_transaction, &id);
            }
        }
        Ok(())
    }

    pub fn select_all(&mut self) -> Result<(), WbblWebappStoreError> {
        let id = self.graph.client_id();
        {
            let mut txn: TransactionMut<'_> = self.graph.transact_mut_with(self.graph.client_id());
            let node_group_selections =
                get_map(&id.to_string(), &txn, &self.node_group_selections)?;

            for entity_id in self.entities.borrow().keys() {
                match entity_id {
                    WbblWebappGraphEntityId::NodeId(node_id) => {
                        encode_selection(&mut txn, *node_id, &self.nodes, id.clone(), true)?;
                    }
                    WbblWebappGraphEntityId::EdgeId(edge_id) => {
                        encode_selection(&mut txn, *edge_id, &self.edges, id.clone(), true)?;
                    }
                    WbblWebappGraphEntityId::GroupId(group_id) => {
                        node_group_selections.insert(
                            &mut txn,
                            uuid::Uuid::from_u128(*group_id).to_string(),
                            true,
                        );
                    }
                }
            }
        }
        Ok(())
    }

    pub fn select_group(
        &mut self,
        group_id: &str,
        additive: bool,
    ) -> Result<(), WbblWebappStoreError> {
        let id = self.graph.client_id();
        let group_uuid: u128 = try_into_u128(group_id)?;
        let group = self
            .entities
            .borrow()
            .get(&WbblWebappGraphEntityId::GroupId(group_uuid));
        if group.is_none() {
            return Err(WbblWebappStoreError::NotFound);
        }
        let group: WbblWebappNodeGroup = group.unwrap().clone().try_into()?;

        {
            let mut txn = self.graph.transact_mut_with(self.graph.client_id());
            if !additive {
                clear_local_selections(
                    &mut txn,
                    id,
                    &self.nodes,
                    &self.edges,
                    &self.node_group_selections,
                    &self.locally_selected_entities.borrow(),
                )?;
            }
            let node_group_selections: MapRef =
                get_map(&id.to_string(), &txn, &self.node_group_selections)?;
            node_group_selections.insert(&mut txn, group_id, true);
            for node_id in group.nodes.iter() {
                encode_selection(&mut txn, node_id.clone(), &self.nodes, id.clone(), true)?;
            }
            let edge_ids = get_mutual_edges(&txn, &group.nodes, &self.nodes)?;
            for e in edge_ids {
                encode_selection(&mut txn, e, &self.nodes, id.clone(), true)?;
            }
        }
        Ok(())
    }

    pub fn deselect_group(&mut self, group_id: &str) -> Result<(), WbblWebappStoreError> {
        let id = self.graph.client_id();
        let group_uuid: u128 = try_into_u128(group_id)?;
        let group = self
            .entities
            .borrow()
            .get(&WbblWebappGraphEntityId::GroupId(group_uuid));
        if group.is_none() {
            return Err(WbblWebappStoreError::NotFound);
        }
        let group: WbblWebappNodeGroup = group.unwrap().clone().try_into()?;
        {
            let mut txn = self.graph.transact_mut_with(self.graph.client_id());
            let node_group_selections =
                get_map(&id.to_string(), &txn, &self.node_group_selections)?;
            node_group_selections.remove(&mut txn, group_id);

            for n in group.nodes.iter() {
                encode_selection(&mut txn, *n, &self.nodes, id, false)?;
            }
            let edge_ids = get_mutual_edges(&txn, &group.nodes, &self.nodes)?;
            for e in edge_ids {
                encode_selection(&mut txn, e, &self.edges, id, false)?;
            }
        }
        Ok(())
    }

    pub fn select_none(&mut self) -> Result<(), WbblWebappStoreError> {
        let id = self.graph.client_id();
        {
            let mut txn = self.graph.transact_mut_with(self.graph.client_id());
            clear_local_selections(
                &mut txn,
                id,
                &self.nodes,
                &self.edges,
                &self.node_group_selections,
                &self.locally_selected_entities.borrow(),
            )?;
        }
        Ok(())
    }

    pub fn get_locally_selected_nodes(&self) -> Vec<String> {
        return self
            .locally_selected_entities
            .borrow()
            .iter()
            .filter_map(|x| match x {
                WbblWebappGraphEntityId::NodeId(id) => {
                    uuid::Uuid::from_u128(*id).to_string().into()
                }
                _ => None,
            })
            .collect();
    }

    pub fn get_locally_selected_edges(&self) -> Vec<String> {
        return self
            .locally_selected_entities
            .borrow()
            .iter()
            .filter_map(|x| match x {
                WbblWebappGraphEntityId::EdgeId(id) => {
                    uuid::Uuid::from_u128(*id).to_string().into()
                }
                _ => None,
            })
            .collect();
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
            preview_node.encode(mut_transaction, &mut self.nodes, self.graph.client_id())?;
            let source =
                uuid::Uuid::from_str(node_id).map_err(|_| WbblWebappStoreError::MalformedId)?;
            let edge = WbblWebappEdge::new(&(source.as_u128()), &preview_node.id, 0, 0);
            edge.encode(mut_transaction, &mut self.edges, self.graph.client_id())?;
        }
        Ok(())
    }

    pub fn make_junction(
        &mut self,
        edge_id: &str,
        position_x: f64,
        position_y: f64,
    ) -> Result<(), WbblWebappStoreError> {
        {
            let client_id = self.graph.client_id();
            let uuid =
                uuid::Uuid::parse_str(edge_id).map_err(|_| WbblWebappStoreError::MalformedId)?;
            let mut txn = self.graph.transact_mut_with(self.graph.client_id());
            let edge = get_map(edge_id, &txn, &self.edges)?;
            let edge = WbblWebappEdge::decode(uuid.as_u128(), &txn, &edge, self.graph.client_id())?;
            let new_node =
                NewWbblWebappNode::new(position_x, position_y, WbblWebappNodeType::Junction)?;
            new_node.encode(&mut txn, &mut self.nodes, client_id.clone())?;
            let edge_1 = WbblWebappEdge::new(&edge.source, &new_node.id, edge.source_handle, 0);
            let edge_2 = WbblWebappEdge::new(&new_node.id, &edge.target, 0, edge.target_handle);
            edge_1.encode(&mut txn, &mut self.edges, client_id.clone())?;
            edge_2.encode(&mut txn, &mut self.edges, client_id.clone())?;
            let edge_id = try_into_u128(edge_id)?;
            delete_edge(&mut txn, edge_id, &mut self.edges)?;
        }
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
