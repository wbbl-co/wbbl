use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
    str::FromStr,
    sync::Arc,
    vec,
};

use glam::Vec2;
use rstar::RTree;
use wasm_bindgen::prelude::*;
use web_sys::{js_sys, MessageEvent, Worker};
use yrs::{
    types::{PathSegment, ToJson},
    DeepObservable, Map, MapPrelim, MapRef, Transact, TransactionMut,
};

use crate::{
    convex_hull::{get_convex_hull, get_ray_ray_intersection},
    data_types::AbstractDataType,
    graph_transfer_types::{
        from_type_name, get_type_name, Any, WbblWebappEdge, WbblWebappGraphEntity,
        WbblWebappGraphEntityId, WbblWebappGraphSnapshot, WbblWebappNode, WbblWebappNodeGroup,
        WbblWebappNodeType, WbblePosition, GRAPH_YRS_EDGES_MAP_KEY, GRAPH_YRS_NODES_MAP_KEY,
        GRAPH_YRS_NODE_GROUP_SELECTIONS_MAP_KEY,
    },
    log,
    node_display_data::{get_in_port_position, get_node_dimensions, get_out_port_position},
    store_errors::WbblWebappStoreError,
    utils::try_into_u128,
    wbbl_graph_web_worker::{WbblGraphWebWorkerRequestMessage, WbblGraphWebWorkerResponseMessage},
    yrs_utils::*,
};

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
    graph_worker: Arc<Worker>,
    worker_responder: Closure<dyn FnMut(MessageEvent) -> ()>,
    spatial_index: Arc<RefCell<RTree<WbblWebappGraphEntity>>>,
    computed_types: Arc<RefCell<JsValue>>,
    entities: Arc<RefCell<HashMap<WbblWebappGraphEntityId, WbblWebappGraphEntity>>>,
    js_entities: Arc<RefCell<HashMap<WbblWebappGraphEntityId, JsValue>>>,
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
        selections.insert(txn, client_id.to_string(), true);
    } else {
        selections.remove(txn, &client_id.to_string());
    }
    Ok(())
}

impl NewWbblWebappNode {
    pub(crate) fn encode(
        &self,
        transaction: &mut TransactionMut,
        nodes: &yrs::MapRef,
    ) -> Result<(), WbblWebappStoreError> {
        let prelim_map: yrs::MapPrelim<yrs::Any> = yrs::MapPrelim::new();
        let mut node_ref = nodes.insert(
            transaction,
            uuid::Uuid::from_u128(self.id).to_string(),
            prelim_map,
        );
        node_ref.insert(
            transaction,
            "id",
            uuid::Uuid::from_u128(self.id).to_string(),
        );
        node_ref.insert(transaction, "type", get_type_name(self.node_type));
        node_ref.insert(transaction, "x", self.position.x);
        node_ref.insert(transaction, "y", self.position.y);
        node_ref.insert(transaction, "dragging", false);
        node_ref.insert(transaction, "resizing", false);
        node_ref.insert(transaction, "selections", MapPrelim::<bool>::new());
        encode_data(&self.data, transaction, &mut node_ref)?;
        Ok(())
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

fn get_mutual_edges(
    node_ids: &HashSet<u128>,
    entities: &HashMap<WbblWebappGraphEntityId, WbblWebappGraphEntity>,
) -> Vec<u128> {
    let mut mutual_edges: Vec<u128> = vec![];
    let mut edges_map: HashSet<u128> = HashSet::new();
    for node_id in node_ids.iter() {
        if let Some(WbblWebappGraphEntity::Node(node)) =
            entities.get(&WbblWebappGraphEntityId::NodeId(*node_id))
        {
            let mut in_intersection: Vec<u128> =
                edges_map.intersection(&node.in_edges).cloned().collect();
            let mut out_intersection: Vec<u128> =
                edges_map.intersection(&node.out_edges).cloned().collect();
            mutual_edges.append(&mut in_intersection);
            mutual_edges.append(&mut out_intersection);
            for edge in node.in_edges.iter() {
                edges_map.insert(*edge);
            }
            for edge in node.out_edges.iter() {
                edges_map.insert(*edge);
            }
        }
    }
    mutual_edges
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
        let selected = selections.contains_key(txn, &client_id.to_string());
        let (width, height) = get_node_dimensions(node_type.clone(), None, None);
        Ok(WbblWebappNode {
            id: *key,
            position: WbblePosition { x, y },
            node_type,
            dragging,
            resizing,
            selected,
            width,
            height,
            data: data
                .iter()
                .map(|(k, v)| (k.to_owned(), v.into()))
                .collect::<HashMap<String, Any>>(),
            connectable: true,
            selectable: true,
            selections: selections
                .keys(txn)
                .flat_map(|x| uuid::Uuid::from_str(x))
                .map(|x| x.as_u128())
                .collect(),
            deletable: node_type != WbblWebappNodeType::Output,
            group_id,
            in_edges: HashSet::new(),
            out_edges: HashSet::new(),
        })
    }

    pub(crate) fn encode(
        &self,
        transaction: &mut TransactionMut,
        nodes: &yrs::MapRef,
    ) -> Result<(), WbblWebappStoreError> {
        let prelim_map: yrs::MapPrelim<yrs::Any> = yrs::MapPrelim::new();
        let node_id = uuid::Uuid::from_u128(self.id).to_string();
        let mut node_ref = nodes.insert(transaction, node_id.clone(), prelim_map);
        node_ref.insert(transaction, "id", node_id.clone());
        node_ref.insert(transaction, "type", get_type_name(self.node_type));
        node_ref.insert(transaction, "x", self.position.x);
        let prelim_selections: MapPrelim<bool> = yrs::MapPrelim::new();
        node_ref.insert(transaction, "selections", prelim_selections);
        node_ref.insert(transaction, "y", self.position.y);
        node_ref.insert(transaction, "dragging", false);
        node_ref.insert(transaction, "resizing", false);
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
    node_groups_selections: &MapRef,
    entities: &HashMap<WbblWebappGraphEntityId, WbblWebappGraphEntity>,
) -> Result<(), WbblWebappStoreError> {
    if let Some(WbblWebappGraphEntity::Group(group)) =
        entities.get(&WbblWebappGraphEntityId::GroupId(group_id))
    {
        for node_id in group.nodes.iter() {
            let node_id_str = &uuid::Uuid::from_u128(*node_id).to_string();
            match get_map(&node_id_str, txn, nodes) {
                Ok(node) => {
                    let type_name: String = get_atomic_string("type", txn, &node)?;
                    if let Some(WbblWebappNodeType::Output) = from_type_name(&type_name) {
                        Ok(())
                    } else {
                        delete_node_and_associated_edges(txn, *node_id, nodes, edges, entities)?;
                        Ok(())
                    }
                }
                Err(WbblWebappStoreError::NotFound) => Ok(()),
                Err(err) => Err(err),
            }?;
        }
        node_groups_selections.remove(txn, &uuid::Uuid::from_u128(group_id).to_string());

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
                    encode_selection(txn, node_id.clone(), nodes, client_id.clone(), false)?;
                }
                WbblWebappGraphEntityId::EdgeId(edge_id) => {
                    encode_selection(txn, edge_id.clone(), edges, client_id.clone(), false)?;
                }
                WbblWebappGraphEntityId::GroupId(_) => {}
            }
        }
        Ok(())
    }
}
const INFLATE_GROUP_PATH_BY: f32 = 25.0;

impl WbblWebappNodeGroup {
    pub fn update_group_convex_hull(
        &mut self,
        entities: &HashMap<WbblWebappGraphEntityId, WbblWebappGraphEntity>,
    ) {
        let mut positions: Vec<Vec2> = Vec::new();

        for node_id in self.nodes.iter() {
            let entity = entities.get(&WbblWebappGraphEntityId::NodeId(*node_id));
            if entity.is_none() {
                continue;
            }
            let entity = entity.unwrap();
            if let WbblWebappGraphEntity::Node(node) = entity {
                let top_left = Vec2::new(node.position.x as f32, node.position.y as f32);
                let size = Vec2::new(node.width as f32, node.height as f32);
                positions.push(top_left);

                positions.push(top_left + size);

                positions.push(Vec2 {
                    x: top_left.x + size.x,
                    y: top_left.y,
                });

                positions.push(Vec2 {
                    x: top_left.x,
                    y: top_left.y + size.y,
                });
            }
        }

        let convex_hull = get_convex_hull(&mut positions);

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
            self.path = Some(result);
            self.bounds = inflated_convex_hull;
        }
    }
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
        let selections: MapRef = get_map("selections", txn, map)?;
        let group_id: Option<u128> = match get_atomic_u128_from_string("group_id", txn, map) {
            Ok(id) => Ok(Some(id)),
            Err(WbblWebappStoreError::NotFound) => Ok(None),
            Err(err) => Err(err),
        }?;

        let selected = selections.contains_key(txn, &client_id.to_string());
        let selections = selections
            .keys(txn)
            .flat_map(|x| uuid::Uuid::from_str(x))
            .map(|x| x.as_u128())
            .collect();

        Ok(WbblWebappEdge {
            id: key,
            source,
            target,
            source_handle,
            target_handle,
            deletable: true,
            selectable: true,
            updatable: false,
            selections,
            selected,
            source_position: Vec2::ZERO,
            target_position: Vec2::ZERO,
            group_id,
        })
    }

    pub(crate) fn encode(
        &self,
        txn: &mut TransactionMut,
        edges: &yrs::MapRef,
    ) -> Result<(), WbblWebappStoreError> {
        let prelim_map: HashMap<String, yrs::Any> = HashMap::new();
        let edge_ref = edges.insert(
            txn,
            uuid::Uuid::from_u128(self.id).to_string(),
            MapPrelim::from(prelim_map),
        );
        edge_ref.insert(
            txn,
            "id".to_owned(),
            yrs::Any::String(uuid::Uuid::from_u128(self.id).to_string().into()),
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
    if node.node_type == WbblWebappNodeType::Output {
        return Ok(()); // DO NOTHING
    }
    for edge in node.in_edges {
        delete_edge(txn, edge, edges)?;
    }
    for edge in node.out_edges {
        delete_edge(txn, edge, edges)?;
    }
    nodes.remove(txn, &uuid::Uuid::from_u128(node_id).to_string());
    Ok(())
}

fn update_entity(
    id: &WbblWebappGraphEntityId,
    entity: &WbblWebappGraphEntity,
    entities: &mut HashMap<WbblWebappGraphEntityId, WbblWebappGraphEntity>,
    js_entities: &mut HashMap<WbblWebappGraphEntityId, JsValue>,
    spatial_index: &mut RTree<WbblWebappGraphEntity>,
) {
    let js_value = match entity {
        WbblWebappGraphEntity::Node(node) => serde_wasm_bindgen::to_value(&node).unwrap(),
        WbblWebappGraphEntity::Edge(edge) => serde_wasm_bindgen::to_value(&edge).unwrap(),
        WbblWebappGraphEntity::Group(group) => serde_wasm_bindgen::to_value(&group).unwrap(),
    };
    js_entities.insert(id.clone(), js_value);
    if let Some(prev_entity) = entities.get(id) {
        spatial_index.remove(prev_entity);
    }
    spatial_index.insert(entity.clone());
    match entity {
        WbblWebappGraphEntity::Node(entity) => {
            entities.insert(id.clone(), WbblWebappGraphEntity::Node(entity.clone()))
        }
        WbblWebappGraphEntity::Edge(entity) => {
            entities.insert(id.clone(), WbblWebappGraphEntity::Edge(entity.clone()))
        }
        WbblWebappGraphEntity::Group(entity) => {
            entities.insert(id.clone(), WbblWebappGraphEntity::Group(entity.clone()))
        }
    };
}

fn remove_entity(
    id: &WbblWebappGraphEntityId,
    entities: &mut HashMap<WbblWebappGraphEntityId, WbblWebappGraphEntity>,
    js_entities: &mut HashMap<WbblWebappGraphEntityId, JsValue>,
    spatial_index: &mut RTree<WbblWebappGraphEntity>,
) -> Option<WbblWebappGraphEntity> {
    js_entities.remove(id);
    if let Some(entity) = entities.remove(id) {
        spatial_index.remove(&entity);
        Some(entity)
    } else {
        None
    }
}

fn insert_or_update_node_group(
    node: &WbblWebappNode,
    entities: &mut HashMap<WbblWebappGraphEntityId, WbblWebappGraphEntity>,
    js_entities: &mut HashMap<WbblWebappGraphEntityId, JsValue>,
    spatial_index: &mut RTree<WbblWebappGraphEntity>,
    locally_selected_entities: &mut HashSet<WbblWebappGraphEntityId>,
    group_uuid: u128,
) {
    if node.group_id != Some(group_uuid) {
        let group_id = WbblWebappGraphEntityId::GroupId(group_uuid);
        let group = entities.get(&group_id).map(|x| x.clone());
        if group.is_none() {
            locally_selected_entities.remove(&group_id);
            return;
        }
        if let Some(WbblWebappGraphEntity::Group(mut group)) = group {
            group.nodes.remove(&node.id);
            if group.nodes.len() == 0 {
                locally_selected_entities.remove(&group_id);
                remove_entity(&group_id, entities, js_entities, spatial_index);
            } else {
                group.update_group_convex_hull(entities);
                for e in node.in_edges.iter() {
                    group.edges.remove(e);
                    if let Some(WbblWebappGraphEntity::Edge(mut edge)) = entities
                        .get(&WbblWebappGraphEntityId::EdgeId(*e))
                        .map(|x| x.clone())
                    {
                        edge.group_id = None;
                        update_entity(
                            &WbblWebappGraphEntityId::EdgeId(*e),
                            &WbblWebappGraphEntity::Edge(edge),
                            entities,
                            js_entities,
                            spatial_index,
                        );
                    }
                }
                for e in node.out_edges.iter() {
                    group.edges.remove(e);
                    let edge_id = WbblWebappGraphEntityId::EdgeId(*e);
                    if let Some(WbblWebappGraphEntity::Edge(mut edge)) =
                        entities.get(&edge_id).map(|x| x.clone())
                    {
                        edge.group_id = None;
                        update_entity(
                            &WbblWebappGraphEntityId::EdgeId(*e),
                            &WbblWebappGraphEntity::Edge(edge),
                            entities,
                            js_entities,
                            spatial_index,
                        );
                    }
                }
                update_entity(
                    &group_id,
                    &WbblWebappGraphEntity::Group(group),
                    entities,
                    js_entities,
                    spatial_index,
                );
            }
        }
    } else {
        let group_id = WbblWebappGraphEntityId::GroupId(group_uuid);

        let group = entities.get(&group_id).map(|x| x.clone());
        if group.is_none() {
            let mut group = WbblWebappNodeGroup {
                id: group_uuid,
                nodes: HashSet::from([node.id]),
                path: None,
                edges: HashSet::new(),
                bounds: vec![],
                selected: false,
                selections: HashSet::new(),
            };
            group.update_group_convex_hull(&entities);
            update_entity(
                &group_id,
                &WbblWebappGraphEntity::Group(group),
                entities,
                js_entities,
                spatial_index,
            );
        } else if let Some(WbblWebappGraphEntity::Group(mut group)) = group {
            group.nodes.insert(node.id);
            group.update_group_convex_hull(&entities);
            let edges: HashSet<u128> = get_mutual_edges(&group.nodes, entities)
                .iter()
                .cloned()
                .collect();
            for e in edges.iter() {
                if let Some(WbblWebappGraphEntity::Edge(mut edge)) = entities
                    .get(&WbblWebappGraphEntityId::EdgeId(*e))
                    .map(|x| x.clone())
                {
                    edge.group_id = Some(group_uuid);
                    update_entity(
                        &WbblWebappGraphEntityId::EdgeId(*e),
                        &WbblWebappGraphEntity::Edge(edge),
                        entities,
                        js_entities,
                        spatial_index,
                    );
                }
            }
            group.edges = edges;

            update_entity(
                &group_id,
                &WbblWebappGraphEntity::Group(group),
                entities,
                js_entities,
                spatial_index,
            );
        }
    }
}

#[wasm_bindgen]
impl WbblWebappGraphStore {
    pub fn empty(graph_worker: Worker) -> Self {
        let graph_worker = Arc::new(graph_worker);
        let graph = yrs::Doc::new();
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
        let spatial_index: Arc<RefCell<RTree<WbblWebappGraphEntity>>> =
            Arc::new(RefCell::new(RTree::new()));

        let computed_types = Arc::new(RefCell::new(JsValue::null()));
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
                        computed_types.replace(serde_wasm_bindgen::to_value(&types).unwrap());
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

        let entities: Arc<RefCell<HashMap<WbblWebappGraphEntityId, WbblWebappGraphEntity>>> =
            Arc::new(RefCell::new(HashMap::new()));

        let js_entities: Arc<RefCell<HashMap<WbblWebappGraphEntityId, JsValue>>> =
            Arc::new(RefCell::new(HashMap::new()));

        let nodes_subscription =
            nodes.observe_deep({
                let entities = entities.clone();
                let client_id = graph.client_id();
                let locally_selected_entities = locally_selected_entities.clone();
                let nodes = nodes.clone();
                let js_entities = js_entities.clone();
                let spatial_index = spatial_index.clone();
                move |mut txn, evts| {
                    for evt in evts.iter() {
                        let path = evt.path();
                        if path.len() == 0 {
                            // Signals an addition/removal from the graph store
                            if let yrs::types::Event::Map(map_evt) = evt {
                                for (key, change) in map_evt.keys(&mut txn).iter() {
                                    if let Ok(node_id) =
                                        uuid::Uuid::parse_str(&key).map(|id| id.as_u128())
                                    {
                                        match change {
                                            yrs::types::EntryChange::Inserted(
                                                yrs::Value::YMap(node),
                                            ) => {
                                                if let Ok(node) = WbblWebappNode::decode(
                                                    &node_id, txn, node, client_id,
                                                ) {
                                                    let mut entities = entities.borrow_mut();
                                                    let mut js_entities = js_entities.borrow_mut();
                                                    let mut spatial_index = spatial_index.borrow_mut();

                                                    let node_id =
                                                        WbblWebappGraphEntityId::NodeId(node_id);
                                                    update_entity(&node_id, &WbblWebappGraphEntity::Node(node.clone()), &mut entities, &mut js_entities, &mut spatial_index);
                                                    if let Some(group_id) = node.group_id {
                                                        insert_or_update_node_group(
                                                            &node,
                                                            &mut entities,
                                                            &mut js_entities,
                                                            &mut spatial_index,
                                                            &mut locally_selected_entities
                                                                .borrow_mut(),
                                                            group_id,
                                                        );
                                                    };
                                                }
                                            }
                                            yrs::types::EntryChange::Removed(_) => {
                                                let node_id =
                                                    WbblWebappGraphEntityId::NodeId(node_id);
                                                locally_selected_entities
                                                    .borrow_mut()
                                                    .remove(&node_id);
                                                let mut entities = entities.borrow_mut();
                                                let mut js_entities = js_entities.borrow_mut();
                                                let mut spatial_index = spatial_index.borrow_mut();

                                                if let Some(WbblWebappGraphEntity::Node(mut prev_node)) = remove_entity(&node_id, &mut entities, &mut js_entities, &mut spatial_index) {
                                                    if let Some(prev_group_id) = prev_node.group_id {
                                                        prev_node.group_id = None;
                                                        insert_or_update_node_group(
                                                            &prev_node,
                                                            &mut entities,
                                                            &mut js_entities,
                                                            &mut spatial_index,
                                                            &mut locally_selected_entities
                                                                .borrow_mut(),
                                                            prev_group_id,
                                                        );
                                                    }
                                                }
                                                // TODO: Update spatial index
                                                // TODO: Update node group
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                            }
                        } else if path.len() == 1 {
                            if let Some(PathSegment::Key(node_id)) = path.get(0) {
                                if let Ok(node_id) = try_into_u128(node_id) {
                                    let mut entities = entities.borrow_mut();
                                    let mut js_entities = js_entities.borrow_mut();
                                    let mut spatial_index = spatial_index.borrow_mut();

                                    if let yrs::Value::YMap(node_map) = evt.target() {
                                        if let Ok(mut node) = WbblWebappNode::decode(
                                            &node_id, txn, &node_map, client_id,
                                        ) {
                                            let key = WbblWebappGraphEntityId::NodeId(node_id);
                                            if let Some(WbblWebappGraphEntity::Node(prev_node)) =
                                                remove_entity(&key, &mut entities, &mut js_entities, &mut spatial_index)
                                            {
                                                node.width = prev_node.width;
                                                node.height = prev_node.height;
                                                node.in_edges = prev_node.in_edges;
                                                node.out_edges = prev_node.out_edges;
                                                update_entity(
                                                    &key,
                                                    &WbblWebappGraphEntity::Node(node.clone()),
                                                    &mut entities,
                                                    &mut js_entities,
                                                    &mut spatial_index
                                                );
                                                if let yrs::types::Event::Map(map_evt) = evt {
                                                    let keys = map_evt.keys(txn);
                                                    if keys.contains_key("group_id") {
                                                        let group_id_evt =
                                                            keys.get("group_id").unwrap();
                                                        // TODO: Update both groups and edges
                                                        // TODO: Update Spatial Index
                                                        match group_id_evt {
                                                            yrs::types::EntryChange::Inserted(
                                                                yrs::Value::Any(yrs::Any::String(
                                                                    group_id,
                                                                )),
                                                            ) => {
                                                                if let Ok(group_id) =
                                                                    try_into_u128(group_id)
                                                                {
                                                                    insert_or_update_node_group(
                                                                    &node,
                                                                    &mut entities,
                                                                    &mut js_entities,
                                                                    &mut spatial_index,
                                                                    &mut locally_selected_entities
                                                                        .borrow_mut(),
                                                                    group_id,
                                                                );
                                                                }
                                                            }
                                                            yrs::types::EntryChange::Updated(
                                                                yrs::Value::Any(yrs::Any::String(
                                                                    old_group_id,
                                                                )),
                                                                yrs::Value::Any(yrs::Any::String(
                                                                    new_group_id,
                                                                )),
                                                            ) => {
                                                                let mut locally_selected_entities =
                                                                    locally_selected_entities
                                                                        .borrow_mut();
                                                                if let (
                                                                    Ok(old_group_id),
                                                                    Ok(new_group_id),
                                                                ) = (
                                                                    try_into_u128(old_group_id),
                                                                    try_into_u128(new_group_id),
                                                                ) {
                                                                    insert_or_update_node_group(
                                                                        &node,
                                                                        &mut entities,
                                                                        &mut js_entities,
                                                                        &mut spatial_index,
                                                                        &mut locally_selected_entities,
                                                                        old_group_id,
                                                                    );
                                                                    insert_or_update_node_group(
                                                                    &node,
                                                                    &mut entities,
                                                                    &mut js_entities,
                                                                    &mut spatial_index,
                                                                    &mut locally_selected_entities,
                                                                    new_group_id,
                                                                );
                                                                }
                                                            }
                                                            yrs::types::EntryChange::Removed(
                                                                yrs::Value::Any(yrs::Any::String(
                                                                    group_id,
                                                                )),
                                                            ) => {
                                                                if let Ok(group_id) =
                                                                    try_into_u128(group_id)
                                                                {
                                                                    insert_or_update_node_group(
                                                                    &node,
                                                                    &mut entities,
                                                                    &mut js_entities,
                                                                    &mut spatial_index,
                                                                    &mut locally_selected_entities
                                                                        .borrow_mut(),
                                                                    group_id,
                                                                );
                                                                }
                                                            }
                                                            _ => {}
                                                        };
                                                    }
                                                    if keys.contains_key("x")
                                                        || keys.contains_key("y")
                                                    {
                                                        if let Some(group_id) = node.group_id {
                                                            insert_or_update_node_group(
                                                                &node,
                                                                &mut entities,
                                                                &mut js_entities,
                                                                &mut spatial_index,
                                                                &mut locally_selected_entities
                                                                    .borrow_mut(),
                                                                group_id,
                                                            );
                                                        }
                                                        let top_left = Vec2::new(
                                                            node.position.x as f32,
                                                            node.position.y as f32,
                                                        );
                                                        for edge_id in node.in_edges.iter() {
                                                            let edge_id = WbblWebappGraphEntityId::EdgeId(
                                                                *edge_id,
                                                            );
                                                            let edge = entities.get_mut(
                                                                &edge_id,
                                                            );

                                                            match edge {
                                                                Some(
                                                                    WbblWebappGraphEntity::Edge(
                                                                        edge,
                                                                    ),
                                                                ) => {
                                                                    let (x, y) =
                                                                        get_in_port_position(
                                                                            node.node_type,
                                                                            edge.target_handle
                                                                                as u8,
                                                                        );
                                                                    let target_position = top_left
                                                                        + Vec2::new(
                                                                            x as f32, y as f32,
                                                                        );
                                                                    edge.target_position =
                                                                        target_position;
                                                                    update_entity(&edge_id, &WbblWebappGraphEntity::Edge(edge.clone()), &mut entities, &mut js_entities, &mut spatial_index);
                                                                }
                                                                _ => {}
                                                            };
                                                        }
                                                        for edge_id in node.out_edges.iter() {
                                                            let edge_id = WbblWebappGraphEntityId::EdgeId(
                                                                *edge_id,
                                                            );
                                                            let edge = entities.get_mut(
                                                                &edge_id,
                                                            );

                                                            match edge {
                                                                Some(
                                                                    WbblWebappGraphEntity::Edge(
                                                                        edge,
                                                                    ),
                                                                ) => {
                                                                    let (x, y) =
                                                                        get_out_port_position(
                                                                            node.node_type,
                                                                            edge.source_handle
                                                                                as u8,
                                                                            Some(
                                                                                node.in_edges.len()
                                                                                    as u8,
                                                                            ),
                                                                            Some(
                                                                                node.out_edges.len()
                                                                                    as u8,
                                                                            ),
                                                                        );
                                                                    let source_position = top_left
                                                                        + Vec2::new(
                                                                            x as f32, y as f32,
                                                                        );
                                                                    edge.source_position =
                                                                        source_position;
                                                                    update_entity(&edge_id, &WbblWebappGraphEntity::Edge(edge.clone()), &mut entities, &mut js_entities, &mut spatial_index);
                                                                }
                                                                _ => {}
                                                            };
                                                        }
                                                        // TODO: Update Spatial Index
                                                    }
                                                }

                                            }
                                        }
                                    }
                                }
                            }
                        } else {
                            if let Some(PathSegment::Key(node_id)) = path.get(0) {
                                let mut entities = entities.borrow_mut();
                                let mut js_entities = js_entities.borrow_mut();
                                let mut spatial_index = spatial_index.borrow_mut();

                                if let Ok(node_id_uuid) = try_into_u128(node_id) {
                                    let key = WbblWebappGraphEntityId::NodeId(node_id_uuid);
                                    if let Some(WbblWebappGraphEntity::Node(prev_node)) =
                                        remove_entity(&key, &mut entities, &mut js_entities, &mut spatial_index)
                                    {
                                        if let Ok(node) = get_map(node_id, txn, &nodes) {
                                            if let Ok(mut node) = WbblWebappNode::decode(
                                                &node_id_uuid,
                                                txn,
                                                &node,
                                                client_id,
                                            ) {
                                                match (path.len() == 2, path.get(1).unwrap()) {
                                                    (true, PathSegment::Key(segment))
                                                        if segment.to_string() == "selections" =>
                                                    {
                                                        if node.selected {
                                                            locally_selected_entities
                                                                .borrow_mut()
                                                                .insert(key);
                                                        } else {
                                                            locally_selected_entities
                                                                .borrow_mut()
                                                                .remove(&key);
                                                        }
                                                    }
                                                    _ => {}
                                                };
                                                node.width = prev_node.width;
                                                node.height = prev_node.height;
                                                node.in_edges = prev_node.in_edges;
                                                node.out_edges = prev_node.out_edges;
                                                update_entity(&WbblWebappGraphEntityId::NodeId(node.id), &WbblWebappGraphEntity::Node(node.clone()), &mut entities, &mut js_entities, &mut spatial_index);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            });
        let edges_subscription = edges.observe_deep({
            let entities = entities.clone();
            let js_entities = js_entities.clone();
            let client_id = graph.client_id();
            let edges = edges.clone();
            let locally_selected_entities = locally_selected_entities.clone();
            let spatial_index = spatial_index.clone();

            move |mut txn, evts| {
                for evt in evts.iter() {
                    let path = evt.path();
                    if path.len() == 0 {
                        // Signals an addition/removal from the graph store
                        if let yrs::types::Event::Map(map_evt) = evt {
                            for (key, change) in map_evt.keys(&mut txn).iter() {
                                if let Ok(edge_uuid) =
                                    uuid::Uuid::parse_str(&key).map(|id| id.as_u128())
                                {
                                    match change {
                                        yrs::types::EntryChange::Inserted(yrs::Value::YMap(
                                            node,
                                        )) => {
                                            if let Ok(mut edge) = WbblWebappEdge::decode(
                                                edge_uuid, txn, node, client_id,
                                            ) {
                                                let edge_id =
                                                    WbblWebappGraphEntityId::EdgeId(edge_uuid);

                                                let mut source_position = Vec2::ZERO;
                                                let mut target_position = Vec2::ZERO;
                                                // TODO: Update Spatial Index
                                                let mut entities = entities.borrow_mut();
                                                let mut js_entities = js_entities.borrow_mut();
                                                let mut spatial_index = spatial_index.borrow_mut();

                                                let source_node = entities.get_mut(
                                                    &WbblWebappGraphEntityId::NodeId(edge.source),
                                                );
                                                let mut source_group_id: Option<u128> = None;

                                                let mut target_group_id: Option<u128> = None;
                                                if source_node.is_some() {
                                                    let source_node = source_node.unwrap();

                                                    match source_node {
                                                        WbblWebappGraphEntity::Node(n) => {
                                                            source_group_id = n.group_id.clone();
                                                            n.out_edges.insert(edge_uuid);
                                                            let (mut x, mut y) =
                                                                get_out_port_position(
                                                                    n.node_type,
                                                                    edge.source_handle as u8,
                                                                    Some(n.in_edges.len() as u8),
                                                                    Some(n.out_edges.len() as u8),
                                                                );
                                                            x = n.position.x + x;
                                                            y = n.position.y + y;
                                                            source_position.x = x as f32;
                                                            source_position.y = y as f32;
                                                            update_entity(
                                                                &WbblWebappGraphEntityId::NodeId(
                                                                    n.id,
                                                                ),
                                                                &WbblWebappGraphEntity::Node(
                                                                    n.clone(),
                                                                ),
                                                                &mut entities,
                                                                &mut js_entities,
                                                                &mut spatial_index,
                                                            );
                                                        }
                                                        _ => {}
                                                    };
                                                }

                                                let target_node = entities.get_mut(
                                                    &WbblWebappGraphEntityId::NodeId(edge.target),
                                                );
                                                if target_node.is_some() {
                                                    let target_node = target_node.unwrap();
                                                    match target_node {
                                                        WbblWebappGraphEntity::Node(n) => {
                                                            target_group_id = n.group_id.clone();
                                                            n.out_edges.insert(edge_uuid);
                                                            let (mut x, mut y) =
                                                                get_in_port_position(
                                                                    n.node_type,
                                                                    edge.source_handle as u8,
                                                                );
                                                            x = n.position.x + x;
                                                            y = n.position.y + y;
                                                            target_position.x = x as f32;
                                                            target_position.y = y as f32;
                                                            update_entity(
                                                                &WbblWebappGraphEntityId::NodeId(
                                                                    n.id,
                                                                ),
                                                                &WbblWebappGraphEntity::Node(
                                                                    n.clone(),
                                                                ),
                                                                &mut entities,
                                                                &mut js_entities,
                                                                &mut spatial_index,
                                                            );
                                                        }
                                                        _ => {}
                                                    };
                                                }
                                                edge.source_position = source_position;
                                                edge.target_position = target_position;
                                                if source_group_id.is_some()
                                                    && source_group_id == target_group_id
                                                {
                                                    let group_id = source_group_id.unwrap();
                                                    if let Some(WbblWebappGraphEntity::Group(
                                                        group,
                                                    )) = entities.get_mut(
                                                        &WbblWebappGraphEntityId::GroupId(group_id),
                                                    ) {
                                                        group.edges.insert(edge.id);
                                                        edge.group_id = Some(group_id);
                                                        update_entity(
                                                            &WbblWebappGraphEntityId::GroupId(
                                                                group.id,
                                                            ),
                                                            &WbblWebappGraphEntity::Group(
                                                                group.clone(),
                                                            ),
                                                            &mut entities,
                                                            &mut js_entities,
                                                            &mut spatial_index,
                                                        );
                                                    }
                                                }

                                                // TODO: Update Spatial Index
                                                update_entity(
                                                    &edge_id,
                                                    &WbblWebappGraphEntity::Edge(edge.clone()),
                                                    &mut entities,
                                                    &mut js_entities,
                                                    &mut spatial_index,
                                                );
                                            }
                                        }
                                        yrs::types::EntryChange::Removed(_) => {
                                            let edge_id =
                                                WbblWebappGraphEntityId::EdgeId(edge_uuid);
                                            let mut entities = entities.borrow_mut();
                                            let mut js_entities = js_entities.borrow_mut();
                                            let mut spatial_index = spatial_index.borrow_mut();
                                            locally_selected_entities.borrow_mut().remove(&edge_id);
                                            if let Some(WbblWebappGraphEntity::Edge(prev_edge)) =
                                                remove_entity(
                                                    &edge_id,
                                                    &mut entities,
                                                    &mut js_entities,
                                                    &mut spatial_index,
                                                )
                                            {
                                                if let Some(group_id) = prev_edge.group_id {
                                                    if let Some(WbblWebappGraphEntity::Group(
                                                        group,
                                                    )) = entities.get_mut(
                                                        &WbblWebappGraphEntityId::GroupId(group_id),
                                                    ) {
                                                        group.edges.remove(&prev_edge.id);
                                                        update_entity(
                                                            &WbblWebappGraphEntityId::GroupId(
                                                                group_id,
                                                            ),
                                                            &WbblWebappGraphEntity::Group(
                                                                group.clone(),
                                                            ),
                                                            &mut entities,
                                                            &mut js_entities,
                                                            &mut spatial_index,
                                                        );
                                                    }
                                                }
                                                // TODO: Update Spatial Index
                                                let source_node = entities.get_mut(
                                                    &WbblWebappGraphEntityId::NodeId(
                                                        prev_edge.source,
                                                    ),
                                                );
                                                if source_node.is_some() {
                                                    let source_node = source_node.unwrap();
                                                    match source_node {
                                                        WbblWebappGraphEntity::Node(n) => {
                                                            n.out_edges.remove(&edge_uuid);
                                                            update_entity(
                                                                &WbblWebappGraphEntityId::NodeId(
                                                                    n.id,
                                                                ),
                                                                &WbblWebappGraphEntity::Node(
                                                                    n.clone(),
                                                                ),
                                                                &mut entities,
                                                                &mut js_entities,
                                                                &mut spatial_index,
                                                            );
                                                        }
                                                        _ => {}
                                                    };
                                                }

                                                let target_node = entities.get_mut(
                                                    &WbblWebappGraphEntityId::NodeId(
                                                        prev_edge.target,
                                                    ),
                                                );
                                                if target_node.is_some() {
                                                    let target_node = target_node.unwrap();
                                                    match target_node {
                                                        WbblWebappGraphEntity::Node(n) => {
                                                            n.out_edges.remove(&edge_uuid);
                                                            update_entity(
                                                                &WbblWebappGraphEntityId::NodeId(
                                                                    n.id,
                                                                ),
                                                                &WbblWebappGraphEntity::Node(
                                                                    n.clone(),
                                                                ),
                                                                &mut entities,
                                                                &mut js_entities,
                                                                &mut spatial_index,
                                                            );
                                                        }
                                                        _ => {}
                                                    };
                                                }
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                    } else if path.len() == 2 {
                        if let Some(PathSegment::Key(edge_id)) = path.get(0) {
                            let mut entities = entities.borrow_mut();
                            let mut js_entities = js_entities.borrow_mut();
                            let mut spatial_index = spatial_index.borrow_mut();
                            if let Ok(edge_id_uuid) = try_into_u128(edge_id) {
                                let key = WbblWebappGraphEntityId::EdgeId(edge_id_uuid);
                                if let Some(WbblWebappGraphEntity::Edge(prev_edge)) = remove_entity(
                                    &key,
                                    &mut entities,
                                    &mut js_entities,
                                    &mut spatial_index,
                                ) {
                                    if let Ok(node) = get_map(edge_id, txn, &edges) {
                                        if let Ok(mut edge) = WbblWebappEdge::decode(
                                            edge_id_uuid,
                                            txn,
                                            &node,
                                            client_id,
                                        ) {
                                            match path.get(1).unwrap() {
                                                PathSegment::Key(segment)
                                                    if segment.to_string() == "selections" =>
                                                {
                                                    if edge.selected {
                                                        locally_selected_entities
                                                            .borrow_mut()
                                                            .insert(key);
                                                    } else {
                                                        locally_selected_entities
                                                            .borrow_mut()
                                                            .remove(&key);
                                                    }
                                                }
                                                _ => {}
                                            };
                                            edge.source_position = prev_edge.source_position;
                                            edge.target_position = prev_edge.target_position;
                                            update_entity(
                                                &key,
                                                &WbblWebappGraphEntity::Edge(edge.clone()),
                                                &mut entities,
                                                &mut js_entities,
                                                &mut spatial_index,
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });

        let node_group_selections_subscription = node_group_selections.observe_deep({
            let locally_selected_entities = locally_selected_entities.clone();
            let client_id = graph.client_id().to_string();
            let entities = entities.clone();
            let spatial_index = spatial_index.clone();
            let js_entities = js_entities.clone();
            move |txn, evts| {
                for evt in evts.iter() {
                    if let yrs::types::Event::Map(map_evt) = evt {
                        let path = map_evt.path();
                        if path.len() == 1 {
                            match path.get(0) {
                                Some(PathSegment::Key(segment)) => {
                                    let keys = map_evt.keys(txn);
                                    for (key, change) in keys {
                                        if let Ok(key) = try_into_u128(key) {
                                            let mut entities = entities.borrow_mut();
                                            let mut js_entities = js_entities.borrow_mut();
                                            let mut spatial_index = spatial_index.borrow_mut();
                                            if let Some(WbblWebappGraphEntity::Group(group)) =
                                                entities
                                                    .get_mut(&WbblWebappGraphEntityId::GroupId(key))
                                            {
                                                if segment.to_string() == client_id {
                                                    match change {
                                                        yrs::types::EntryChange::Inserted(_) => {
                                                            locally_selected_entities
                                                                .borrow_mut()
                                                                .insert(
                                                                WbblWebappGraphEntityId::GroupId(
                                                                    key,
                                                                ),
                                                            );
                                                            group.selected = true;
                                                        }
                                                        yrs::types::EntryChange::Removed(_) => {
                                                            locally_selected_entities
                                                                .borrow_mut()
                                                                .remove(
                                                                &WbblWebappGraphEntityId::GroupId(
                                                                    key,
                                                                ),
                                                            );
                                                            group.selected = false;
                                                        }
                                                        _ => {}
                                                    };
                                                }
                                                match change {
                                                    yrs::types::EntryChange::Inserted(_) => {
                                                        group.selections.insert(key);
                                                    }
                                                    yrs::types::EntryChange::Removed(_) => {
                                                        group.selections.remove(&key);
                                                    }
                                                    _ => {}
                                                };

                                                update_entity(
                                                    &WbblWebappGraphEntityId::GroupId(group.id),
                                                    &&WbblWebappGraphEntity::Group(group.clone()),
                                                    &mut entities,
                                                    &mut js_entities,
                                                    &mut spatial_index,
                                                );
                                            }
                                        }
                                    }
                                }
                                _ => {}
                            };
                        }
                    };
                }
            }
        });

        let doc_subscription = graph
            .observe_update_v2({
                let listeners: Arc<RefCell<Vec<(u32, js_sys::Function)>>> = listeners.clone();
                let graph_worker = graph_worker.clone();
                move |_, update| {
                    for (_, listener) in listeners.borrow().iter() {
                        let _ = listener
                            .call0(&JsValue::UNDEFINED)
                            .inspect_err(|err| log!("Publish error: {:?}", err));
                    }
                    let update = update.update.clone();
                    let snapshot_js_value = serde_wasm_bindgen::to_value(
                        &WbblGraphWebWorkerRequestMessage::ReceiveUpdate(update),
                    )
                    .unwrap();
                    // let update = js_sys::Reflect::get(
                    //     &snapshot_js_value,
                    //     &js_sys::JsString::from_str("ReceiveUpdate").unwrap(),
                    // )
                    // .unwrap();
                    graph_worker.post_message(&snapshot_js_value).unwrap();
                }
            })
            .unwrap();

        let subscriptions = vec![
            nodes_subscription,
            edges_subscription,
            node_group_selections_subscription,
            doc_subscription,
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
            spatial_index: spatial_index.clone(),
            js_entities: js_entities.clone(),
            graph_worker: graph_worker.clone(),
            worker_responder,
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
                MapPrelim::<bool>::new(),
            );
            store
                .undo_manager
                .expand_scope(&local_node_group_selections);
        }
        store.undo_manager.include_origin(store.graph.client_id()); // only track changes originating from local peer
        store.undo_manager.expand_scope(&store.edges.as_ref());

        store
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
        let js_entities = self.js_entities.borrow();
        let mut js_entities: Vec<(WbblWebappGraphEntityId, JsValue)> = js_entities
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        js_entities.sort_unstable_by_key(|(id, _)| id.clone());
        let nodes: Vec<JsValue> = js_entities
            .iter()
            .filter_map(|(k, v)| match k {
                WbblWebappGraphEntityId::NodeId(_) => Some(v),
                _ => None,
            })
            .cloned()
            .collect();
        let edges: Vec<JsValue> = js_entities
            .iter()
            .filter_map(|(k, v)| match k {
                WbblWebappGraphEntityId::EdgeId(_) => Some(v),
                _ => None,
            })
            .cloned()
            .collect();

        let node_groups: Vec<JsValue> = js_entities
            .iter()
            .filter_map(|(k, v)| match k {
                WbblWebappGraphEntityId::GroupId(_) => Some(v),
                _ => None,
            })
            .cloned()
            .collect();
        let node_groups: JsValue = js_sys::Array::from_iter(node_groups).into();
        let node_groups: JsValue = js_sys::Array::from_iter([
            js_sys::JsString::from_str("node_groups").unwrap().into(),
            node_groups,
        ])
        .into();
        let edges: JsValue = js_sys::Array::from_iter(edges).into();
        let edges: JsValue =
            js_sys::Array::from_iter([js_sys::JsString::from_str("edges").unwrap().into(), edges])
                .into();

        let nodes: JsValue = js_sys::Array::from_iter(nodes).into();
        let nodes: JsValue =
            js_sys::Array::from_iter([js_sys::JsString::from_str("nodes").unwrap().into(), nodes])
                .into();

        let computed_types: JsValue = self.computed_types.borrow().clone();
        let graph_types = js_sys::Array::from_iter([
            js_sys::JsString::from_str("computed_types").unwrap().into(),
            computed_types,
        ])
        .into();

        js_sys::Object::from_entries(
            &js_sys::Array::from_iter([nodes, edges, node_groups, graph_types]).into(),
        )
        .map(|x| x.into())
        .map_err(|_| WbblWebappStoreError::SerializationFailure)
    }

    pub fn remove_node(&mut self, node_id: &str) -> Result<(), WbblWebappStoreError> {
        {
            let mut mut_transaction = self.graph.transact_mut_with(self.graph.client_id());
            let node = get_map(&node_id, &mut_transaction, &self.nodes)?;
            let type_name = get_atomic_string(&"type", &mut_transaction, &node)?;
            if let Some(WbblWebappNodeType::Output) = from_type_name(&type_name) {
                return Err(WbblWebappStoreError::CannotDeleteOutputNode);
            }
            let node_id = try_into_u128(&node_id)?;
            delete_node_and_associated_edges(
                &mut mut_transaction,
                node_id,
                &self.nodes,
                &self.edges,
                &self.entities.borrow(),
            )?;
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
            delete_node_group_and_associated_nodes_and_edges(
                &mut mut_transaction,
                group_id_uuid,
                &self.nodes,
                &self.edges,
                &self.node_group_selections,
                &self.entities.borrow(),
            )?;
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
            node.encode(&mut mut_transaction, &mut self.nodes)
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
            if !selected {
                if let Some(WbblWebappGraphEntity::Node(node)) = self
                    .entities
                    .borrow()
                    .get(&WbblWebappGraphEntityId::NodeId(node_id))
                {
                    if let Some(group_id) = node.group_id {
                        let node_group_selections: MapRef = get_map(
                            &id.to_string(),
                            &mut_transaction,
                            &self.node_group_selections,
                        )?;
                        node_group_selections.remove(
                            &mut mut_transaction,
                            &uuid::Uuid::from_u128(group_id).to_string(),
                        );
                    }
                }
            }
            encode_selection(&mut mut_transaction, node_id, &self.nodes, id, selected)?;
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
            let source_uuid = try_into_u128(source)?;
            let target_uuid = try_into_u128(target)?;
            let edge = {
                let entities = self.entities.borrow();
                match (
                    entities.get(&WbblWebappGraphEntityId::NodeId(source_uuid)),
                    entities.get(&WbblWebappGraphEntityId::NodeId(target_uuid)),
                ) {
                    (
                        Some(WbblWebappGraphEntity::Node(source_node)),
                        Some(WbblWebappGraphEntity::Node(target_node)),
                    ) => {
                        let group_id = if source_node.group_id == target_node.group_id {
                            source_node.group_id
                        } else {
                            None
                        };
                        Ok(WbblWebappEdge::new(
                            &source_uuid,
                            &target_uuid,
                            source_handle,
                            target_handle,
                            group_id,
                        ))
                    }
                    _ => Err(WbblWebappStoreError::NotFound),
                }
            }?;

            let mut mut_transaction = self.graph.transact_mut_with(self.graph.client_id());
            edge.encode(&mut mut_transaction, &mut self.edges)?;
        }
        Ok(())
    }

    fn get_selection_snapshot(&self) -> Result<WbblWebappGraphSnapshot, WbblWebappStoreError> {
        let mut nodes: Vec<WbblWebappNode> = Vec::new();
        let mut edges: Vec<WbblWebappEdge> = Vec::new();
        for entity_id in self.locally_selected_entities.borrow().iter() {
            let entities = self.entities.borrow();
            let entity = match entities.get(entity_id) {
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
        let entities = self.entities.borrow();
        let group = entities.get(&WbblWebappGraphEntityId::GroupId(id));
        if group.is_none() {
            return Err(WbblWebappStoreError::NotFound);
        }
        if let WbblWebappGraphEntity::Group(group) = group.unwrap() {
            {
                let node_ids = group.nodes.clone();

                let edge_ids = get_mutual_edges(&node_ids, &entities);
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
            nodes,
            edges: vec![],
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
            if let Some(position) = position {
                snapshot.recenter(&position);
            }
            for node in snapshot.nodes.iter() {
                node.encode(&mut mut_transaction, &self.nodes)?;
            }
            for edge in snapshot.edges.iter() {
                edge.encode(&mut mut_transaction, &self.edges)?;
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
        let entities = self.entities.borrow();
        let group: &WbblWebappNodeGroup =
            match entities.get(&WbblWebappGraphEntityId::GroupId(group_uuid)) {
                Some(WbblWebappGraphEntity::Group(group)) => Ok(group),
                None => Err(WbblWebappStoreError::NotFound),
                _ => Err(WbblWebappStoreError::MalformedId),
            }?;
        let nodes_in_group: HashSet<u128> = group.nodes.clone();
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
            let id = self.graph.client_id();
            let mut mut_transaction: TransactionMut<'_> =
                self.graph.transact_mut_with(self.graph.client_id());
            let edge_id = try_into_u128(edge_id)?;
            if !selected {
                if let Some(WbblWebappGraphEntity::Edge(edge)) = self
                    .entities
                    .borrow()
                    .get(&WbblWebappGraphEntityId::EdgeId(edge_id))
                {
                    if let Some(group_id) = edge.group_id {
                        let node_group_selections: MapRef = get_map(
                            &id.to_string(),
                            &mut_transaction,
                            &self.node_group_selections,
                        )?;
                        node_group_selections.remove(
                            &mut mut_transaction,
                            &uuid::Uuid::from_u128(group_id).to_string(),
                        );
                    }
                }
            }
            encode_selection(&mut mut_transaction, edge_id, &self.edges, id, selected)?;
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
        let group = {
            let entities = self.entities.borrow();
            entities
                .get(&WbblWebappGraphEntityId::GroupId(group_uuid))
                .cloned()
        };

        if group.is_none() {
            return Err(WbblWebappStoreError::NotFound);
        }
        let group: WbblWebappNodeGroup = group.unwrap().try_into()?;

        let edge_ids = {
            let entities = self.entities.borrow();
            get_mutual_edges(&group.nodes, &entities)
        };

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
        for e in edge_ids {
            encode_selection(&mut txn, e, &self.edges, id.clone(), true)?;
        }

        Ok(())
    }

    pub fn deselect_group(&mut self, group_id: &str) -> Result<(), WbblWebappStoreError> {
        let id = self.graph.client_id();
        let group_uuid: u128 = try_into_u128(group_id)?;

        let group = {
            let entities = self.entities.borrow();
            entities
                .get(&WbblWebappGraphEntityId::GroupId(group_uuid))
                .cloned()
        };
        if group.is_none() {
            return Err(WbblWebappStoreError::NotFound);
        }
        let group: WbblWebappNodeGroup = group.unwrap().try_into()?;
        let edge_ids = {
            let entities = self.entities.borrow();
            get_mutual_edges(&group.nodes, &entities)
        };
        let mut txn = self.graph.transact_mut_with(self.graph.client_id());
        let node_group_selections = get_map(&id.to_string(), &txn, &self.node_group_selections)?;
        node_group_selections.remove(&mut txn, group_id);
        for n in group.nodes.iter() {
            encode_selection(&mut txn, *n, &self.nodes, id, false)?;
        }
        for e in edge_ids {
            encode_selection(&mut txn, e, &self.edges, id, false)?;
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
            {
                let entities = self.entities.borrow();
                let node_uuid = try_into_u128(node_id)?;
                if let Some(WbblWebappGraphEntity::Node(source_node)) =
                    entities.get(&WbblWebappGraphEntityId::NodeId(node_uuid))
                {
                    let preview_node = NewWbblWebappNode::new(
                        source_node.position.x + 350.0,
                        source_node.position.y,
                        WbblWebappNodeType::Preview,
                    )?;
                    preview_node.encode(mut_transaction, &mut self.nodes)?;
                    let source = uuid::Uuid::from_str(node_id)
                        .map_err(|_| WbblWebappStoreError::MalformedId)?;

                    let edge =
                        WbblWebappEdge::new(&(source.as_u128()), &preview_node.id, 0, 0, None);
                    edge.encode(mut_transaction, &mut self.edges)?;
                } else {
                    return Err(WbblWebappStoreError::NotFound);
                }
            }
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
            let uuid =
                uuid::Uuid::parse_str(edge_id).map_err(|_| WbblWebappStoreError::MalformedId)?;
            let mut txn = self.graph.transact_mut_with(self.graph.client_id());
            let edge = get_map(edge_id, &txn, &self.edges)?;
            let edge = WbblWebappEdge::decode(uuid.as_u128(), &txn, &edge, self.graph.client_id())?;
            let new_node =
                NewWbblWebappNode::new(position_x, position_y, WbblWebappNodeType::Junction)?;
            new_node.encode(&mut txn, &self.nodes)?;
            let edge_1 = WbblWebappEdge::new(
                &edge.source,
                &new_node.id,
                edge.source_handle,
                0,
                edge.group_id,
            );
            let edge_2 = WbblWebappEdge::new(
                &new_node.id,
                &edge.target,
                0,
                edge.target_handle,
                edge.group_id,
            );
            edge_1.encode(&mut txn, &self.edges)?;
            edge_2.encode(&mut txn, &self.edges)?;
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
