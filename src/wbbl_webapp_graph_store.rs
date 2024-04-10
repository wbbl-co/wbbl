use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
    str::FromStr,
    sync::Arc,
};

use glam::Vec2;
use wasm_bindgen::prelude::*;
use web_sys::{js_sys, MessageEvent, Worker};
use yrs::{block::Prelim, types::ToJson, Map, MapPrelim, MapRef, Transact, TransactionMut, Value};

use crate::{
    data_types::AbstractDataType,
    graph_transfer_types::{
        from_type_name, get_type_name, Any, WbblWebappEdge, WbblWebappGraphSnapshot,
        WbblWebappNode, WbblWebappNodeType, WbbleComputedNodeSize, WbblePosition,
    },
    graph_types::PortId,
    log,
    wbbl_graph_web_worker::{WbblGraphWebWorkerRequestMessage, WbblGraphWebWorkerResponseMessage},
};

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
) -> Result<HashMap<String, Any>, WbblWebappGraphStoreError> {
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
    ) -> Result<WbblWebappNode, WbblWebappGraphStoreError> {
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
            computed: None,
            dragging: false,
            resizing: false,
            selected: false,
            selectable: true,
            deletable: self.node_type != WbblWebappNodeType::Output,
            connectable: true,
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
        }
    }
    pub fn new(
        position_x: f64,
        position_y: f64,
        node_type: WbblWebappNodeType,
    ) -> NewWbblWebappNode {
        NewWbblWebappNode {
            id: uuid::Uuid::new_v4().as_u128(),
            position: WbblePosition {
                x: position_x,
                y: position_y,
            },
            node_type,
            data: NewWbblWebappNode::get_initial_data(node_type),
        }
    }

    pub fn new_with_data(
        position_x: f64,
        position_y: f64,
        node_type: WbblWebappNodeType,
        data: JsValue,
    ) -> NewWbblWebappNode {
        let data = serde_wasm_bindgen::from_value::<HashMap<String, Any>>(data).unwrap();
        NewWbblWebappNode {
            id: uuid::Uuid::new_v4().as_u128(),
            position: WbblePosition {
                x: position_x,
                y: position_y,
            },
            node_type,
            data,
        }
    }
}

impl WbblWebappGraphSnapshot {
    pub(crate) fn get_snapshot<Transaction: yrs::ReadTxn>(
        read_txn: &Transaction,
        graph_id: u128,
        nodes_map_ref: &yrs::MapRef,
        edges_map_ref: &yrs::MapRef,
        node_selections_map: &yrs::MapRef,
        edge_selections_map: &yrs::MapRef,
    ) -> Result<WbblWebappGraphSnapshot, WbblWebappGraphStoreError> {
        let mut nodes: Vec<WbblWebappNode> = Vec::new();
        let mut edges: Vec<WbblWebappEdge> = Vec::new();
        for node in nodes_map_ref.iter(read_txn) {
            let key = node.0.to_owned();
            let node_values = node.1;
            let key =
                uuid::Uuid::from_str(&key).map_err(|_| WbblWebappGraphStoreError::MalformedId)?;
            let node: WbblWebappNode = match node_values {
                yrs::Value::YMap(map) => {
                    WbblWebappNode::decode(&key.as_u128(), read_txn, &map, node_selections_map)
                }
                _ => Err(WbblWebappGraphStoreError::UnexpectedStructure),
            }?;
            nodes.push(node);
        }

        for edge in edges_map_ref.iter(read_txn) {
            let key = edge.0.to_owned();
            let edge_values = edge.1;
            let key =
                uuid::Uuid::from_str(&key).map_err(|_| WbblWebappGraphStoreError::MalformedId)?;

            let edge: WbblWebappEdge = match edge_values {
                yrs::Value::YMap(map) => {
                    WbblWebappEdge::decode(key.as_u128(), read_txn, &map, edge_selections_map)
                }
                _ => Err(WbblWebappGraphStoreError::UnexpectedStructure),
            }?;
            edges.push(edge);
        }

        nodes.sort_by_key(|n| n.id.clone());
        edges.sort_by_key(|e| e.id.clone());
        Ok(WbblWebappGraphSnapshot {
            id: graph_id,
            nodes,
            edges,
            computed_types: None,
        })
    }
}

fn get_atomic_string<Txn: yrs::ReadTxn>(
    key: &str,
    txn: &Txn,
    map: &yrs::MapRef,
) -> Result<String, WbblWebappGraphStoreError> {
    match map.get(txn, key) {
        Some(yrs::Value::Any(yrs::Any::String(result))) => Ok((*result).to_owned()),
        _ => Err(WbblWebappGraphStoreError::UnexpectedStructure),
    }
}

fn get_atomic_u128_from_string<Txn: yrs::ReadTxn>(
    key: &str,
    txn: &Txn,
    map: &yrs::MapRef,
) -> Result<u128, WbblWebappGraphStoreError> {
    let str = get_atomic_string(key, txn, map)?;
    uuid::Uuid::from_str(&str)
        .map_err(|_| WbblWebappGraphStoreError::MalformedId)
        .map(|uuid| uuid.as_u128())
}

fn get_atomic_bigint<Txn: yrs::ReadTxn>(
    key: &str,
    txn: &Txn,
    map: &yrs::MapRef,
) -> Result<i64, WbblWebappGraphStoreError> {
    match map.get(txn, key) {
        Some(yrs::Value::Any(yrs::Any::BigInt(result))) => Ok(result),
        None => Err(WbblWebappGraphStoreError::NotFound),
        _ => Err(WbblWebappGraphStoreError::UnexpectedStructure),
    }
}

fn get_float_64<Txn: yrs::ReadTxn>(
    key: &str,
    txn: &Txn,
    map: &yrs::MapRef,
) -> Result<f64, WbblWebappGraphStoreError> {
    match map.get(txn, key) {
        Some(yrs::Value::Any(yrs::Any::Number(result))) => Ok(result),
        None => Err(WbblWebappGraphStoreError::NotFound),
        _ => Err(WbblWebappGraphStoreError::UnexpectedStructure),
    }
}

fn get_map<Txn: yrs::ReadTxn>(
    key: &str,
    txn: &Txn,
    map: &yrs::MapRef,
) -> Result<MapRef, WbblWebappGraphStoreError> {
    match map.get(txn, key) {
        Some(yrs::Value::YMap(map_ref)) => Ok(map_ref),
        None => Err(WbblWebappGraphStoreError::NotFound),
        _ => Err(WbblWebappGraphStoreError::UnexpectedStructure),
    }
}

fn get_or_insert_map<T: Prelim>(
    key: &str,
    txn: &mut TransactionMut,
    map: &yrs::MapRef,
    default_value: T,
) -> Result<MapRef, WbblWebappGraphStoreError> {
    match map.get(txn, key) {
        Some(yrs::Value::YMap(map_ref)) => Ok(map_ref),
        None => {
            map.insert(txn, key, default_value);
            get_map(key, txn, map)
        }
        _ => Err(WbblWebappGraphStoreError::UnexpectedStructure),
    }
}

fn get_bool<Txn: yrs::ReadTxn>(
    key: &str,
    txn: &Txn,
    map: &yrs::MapRef,
) -> Result<bool, WbblWebappGraphStoreError> {
    match map.get(txn, key) {
        Some(yrs::Value::Any(yrs::Any::Bool(result))) => Ok(result),
        None => Err(WbblWebappGraphStoreError::NotFound),
        _ => Err(WbblWebappGraphStoreError::UnexpectedStructure),
    }
}

impl WbblWebappNode {
    pub(crate) fn decode<Txn: yrs::ReadTxn>(
        key: &u128,
        txn: &Txn,
        map: &yrs::MapRef,
        selections: &yrs::MapRef,
    ) -> Result<WbblWebappNode, WbblWebappGraphStoreError> {
        let type_name: String = get_atomic_string("type", txn, map)?;
        let data = get_map("data", txn, map)?;
        let x = get_float_64("x", txn, map)?;
        let y = get_float_64("y", txn, map)?;
        let dragging = get_bool("dragging", txn, map)?;
        let resizing = get_bool("resizing", txn, map)?;

        let data = &data.to_json(txn);
        let data = match data {
            yrs::Any::Map(hash_map) => Ok(hash_map.clone()),
            _ => Err(WbblWebappGraphStoreError::UnexpectedStructure),
        }?;
        let node_type = match from_type_name(&type_name) {
            Some(typ) => Ok(typ),
            None => Err(WbblWebappGraphStoreError::UnknownNodeType),
        }?;
        let key_str = uuid::Uuid::from_u128(*key).to_string();
        let selected = selections.iter(txn).any(|(_, selection)| match selection {
            Value::YMap(selection) => selection.contains_key(txn, &key_str),
            _ => false,
        });
        Ok(WbblWebappNode {
            id: *key,
            position: WbblePosition { x, y },
            computed: None,
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
        })
    }

    pub(crate) fn encode(
        &self,
        transaction: &mut TransactionMut,
        nodes: &mut yrs::MapRef,
    ) -> Result<(), WbblWebappGraphStoreError> {
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
) -> Result<(), WbblWebappGraphStoreError> {
    let edge = match get_map(edge_id, transaction, edges) {
        Ok(edge) => Ok(Some(edge)),
        Err(WbblWebappGraphStoreError::NotFound) => Ok(None),
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
        Err(WbblWebappGraphStoreError::NotFound) => Ok(()),
        Err(err) => Err(err),
    }?;
    match get_map(&target, transaction, nodes) {
        Ok(out_node) => {
            let out_edges = get_map("out_edges", transaction, &out_node)?;
            out_edges.remove(transaction, edge_id);
            Ok(())
        }
        Err(WbblWebappGraphStoreError::NotFound) => Ok(()),
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
) -> Result<(), WbblWebappGraphStoreError> {
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

fn delete_node(
    transaction: &mut TransactionMut,
    node_id: &str,
    nodes: &mut MapRef,
    edges: &mut MapRef,
    node_selections: &mut MapRef,
    edge_selections: &mut MapRef,
) -> Result<(), WbblWebappGraphStoreError> {
    let node = match get_map(node_id, transaction, &nodes) {
        Ok(node) => Ok(Some(node)),
        Err(WbblWebappGraphStoreError::NotFound) => Ok(None),
        Err(err) => Err(err),
    }?;
    if node.is_none() {
        return Ok(());
    }
    let node = node.unwrap();

    delete_associated_edges(transaction, &node, nodes, edges, edge_selections)?;
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

impl WbblWebappEdge {
    pub(crate) fn decode<Txn: yrs::ReadTxn>(
        key: u128,
        txn: &Txn,
        map: &yrs::MapRef,
        selections: &yrs::MapRef,
    ) -> Result<WbblWebappEdge, WbblWebappGraphStoreError> {
        let source = get_atomic_u128_from_string("source", txn, map)?;
        let target = get_atomic_u128_from_string("target", txn, map)?;
        let source_handle = get_atomic_bigint("source_handle", txn, map)?;
        let target_handle = get_atomic_bigint("target_handle", txn, map)?;
        let key_str = uuid::Uuid::from_u128(key).to_string();
        let selected = selections.iter(txn).any(|(_, selection)| match selection {
            Value::YMap(selection) => selection.contains_key(txn, &key_str),
            _ => false,
        });
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
    ) -> Result<(), WbblWebappGraphStoreError> {
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
            (_, _) => Err(WbblWebappGraphStoreError::UnexpectedStructure),
        }?;
        Ok(())
    }

    pub(crate) fn encode(
        &self,
        txn: &mut TransactionMut,
        edges: &mut yrs::MapRef,
        nodes: &yrs::MapRef,
    ) -> Result<(), WbblWebappGraphStoreError> {
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
#[derive(Debug)]
pub enum WbblWebappGraphStoreError {
    UnexpectedStructure,
    UnknownNodeType,
    FailedToUndo,
    FailedToRedo,
    FailedToEmit,
    NotFound,
    MalformedId,
    ClipboardFailure,
    ClipboardNotFound,
    ClipboardContentsFailure,
}

#[wasm_bindgen]
impl WbblWebappGraphStore {
    pub fn empty(graph_worker: Worker) -> Self {
        let graph = yrs::Doc::new();
        let node_selections = graph.get_or_insert_map(GRAPH_YRS_NODE_SELECTIONS_MAP_KEY.to_owned());
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
                                .map_err(|_| WbblWebappGraphStoreError::FailedToEmit)
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
            node_selections,
            edge_selections,
            computed_node_sizes: HashMap::new(),
            computed_types: computed_types.clone(),
            graph_worker: graph_worker.clone(),
            worker_responder,
            initialized: false,
        };

        let output_node = NewWbblWebappNode::new(600.0, 500.0, WbblWebappNodeType::Output);
        store.add_node(output_node.clone()).unwrap();

        let slab_node = NewWbblWebappNode::new(200.0, 500.0, WbblWebappNodeType::Slab);
        store.add_node(slab_node.clone()).unwrap();

        store
            .add_edge(
                &uuid::Uuid::from_u128(slab_node.id).to_string(),
                &uuid::Uuid::from_u128(output_node.id).to_string(),
                0,
                0,
            )
            .unwrap();

        store.undo_manager.include_origin(store.graph.client_id()); // only track changes originating from local peer
        store.undo_manager.expand_scope(&store.edges);
        store.undo_manager.expand_scope(&store.node_selections);
        store.undo_manager.expand_scope(&store.edge_selections);
        store.initialized = true;
        store.emit(true).unwrap();
        store
    }

    pub fn emit(&self, should_publish_to_worker: bool) -> Result<(), WbblWebappGraphStoreError> {
        for (_, listener) in self.listeners.borrow().iter() {
            listener
                .call0(&JsValue::UNDEFINED)
                .map_err(|_| WbblWebappGraphStoreError::FailedToEmit)?;
        }
        if should_publish_to_worker && self.initialized {
            let snapshot = self.get_snapshot_raw()?;
            let snapshot_js_value = serde_wasm_bindgen::to_value(
                &WbblGraphWebWorkerRequestMessage::SetSnapshot(snapshot),
            )
            .map_err(|_| WbblWebappGraphStoreError::UnexpectedStructure)?;

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

    pub fn undo(&mut self) -> Result<bool, WbblWebappGraphStoreError> {
        let result = self
            .undo_manager
            .undo()
            .map_err(|_| WbblWebappGraphStoreError::FailedToUndo)?;
        if result {
            self.emit(true)?;
        }
        Ok(result)
    }

    pub fn redo(&mut self) -> Result<bool, WbblWebappGraphStoreError> {
        let result = self
            .undo_manager
            .redo()
            .map_err(|_| WbblWebappGraphStoreError::FailedToRedo)?;
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

    pub fn get_snapshot(&mut self) -> Result<JsValue, WbblWebappGraphStoreError> {
        let mut snapshot = self.get_snapshot_raw()?;
        snapshot.computed_types = Some(self.computed_types.borrow().clone());
        serde_wasm_bindgen::to_value(&snapshot)
            .map_err(|_| WbblWebappGraphStoreError::UnexpectedStructure)
    }

    pub fn remove_node(&mut self, node_id: &str) -> Result<(), WbblWebappGraphStoreError> {
        {
            let mut mut_transaction = self.graph.transact_mut_with(self.graph.client_id());
            delete_node(
                &mut mut_transaction,
                node_id,
                &mut self.nodes,
                &mut self.edges,
                &mut self.node_selections,
                &mut self.edge_selections,
            )?;
        }

        // Important that emit is called here. Rather than before the drop
        // as this could trigger a panic as the store value may be read
        self.emit(true)?;

        Ok(())
    }

    pub fn remove_selected_nodes_and_edges(&mut self) -> Result<(), WbblWebappGraphStoreError> {
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
                            )?;
                            Ok(())
                        }
                    }
                    Err(WbblWebappGraphStoreError::NotFound) => Ok(()),
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

    pub fn remove_edge(&mut self, edge_id: &str) -> Result<(), WbblWebappGraphStoreError> {
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

    pub fn add_node(&mut self, node: NewWbblWebappNode) -> Result<(), WbblWebappGraphStoreError> {
        {
            let mut mut_transaction = self.graph.transact_mut_with(self.graph.client_id());
            node.encode(&mut mut_transaction, &mut self.nodes)
        }?;

        // Important that emit is called here. Rather than before the drop
        // as this could trigger a panic as the store value may be read
        self.emit(true)?;

        Ok(())
    }

    fn get_snapshot_raw(&self) -> Result<WbblWebappGraphSnapshot, WbblWebappGraphStoreError> {
        let read_txn = self.graph.transact();
        let mut snapshot = WbblWebappGraphSnapshot::get_snapshot(
            &read_txn,
            self.id,
            &self.nodes,
            &self.edges,
            &self.node_selections,
            &self.edge_selections,
        )?;
        for node in snapshot.nodes.iter_mut() {
            node.computed = self.computed_node_sizes.get(&node.id).map(|s| s.clone());
        }
        Ok(snapshot)
    }

    pub fn set_computed_node_dimension(
        &mut self,
        node_id: &str,
        width: Option<f64>,
        height: Option<f64>,
        resizing: Option<bool>,
    ) -> Result<(), WbblWebappGraphStoreError> {
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
                _ => Err(WbblWebappGraphStoreError::UnexpectedStructure),
            }?;
        }

        let node_id = uuid::Uuid::from_str(node_id)
            .map(|id| id.as_u128())
            .map_err(|_| WbblWebappGraphStoreError::MalformedId)?;

        if let Some(maybe_computed) = self.computed_node_sizes.get_mut(&node_id) {
            maybe_computed.width = width;
            maybe_computed.height = height;
        } else {
            self.computed_node_sizes.insert(
                node_id.to_owned(),
                WbbleComputedNodeSize {
                    width,
                    height,
                    position_absolute: None,
                },
            );
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
        position_absolute_x: Option<f64>,
        position_absolute_y: Option<f64>,
        dragging: Option<bool>,
    ) -> Result<(), WbblWebappGraphStoreError> {
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
                _ => Err(WbblWebappGraphStoreError::UnexpectedStructure),
            }?;
            let node_id = uuid::Uuid::from_str(node_id)
                .map(|id| id.as_u128())
                .map_err(|_| WbblWebappGraphStoreError::MalformedId)?;
            if let Some(maybe_computed) = self.computed_node_sizes.get_mut(&node_id) {
                maybe_computed.position_absolute = Some(WbblePosition {
                    x: position_absolute_x.unwrap_or(0.0),
                    y: position_absolute_y.unwrap_or(0.0),
                })
            } else {
                self.computed_node_sizes.insert(
                    node_id,
                    WbbleComputedNodeSize {
                        width: None,
                        height: None,
                        position_absolute: Some(WbblePosition {
                            x: position_absolute_x.unwrap_or(0.0),
                            y: position_absolute_y.unwrap_or(0.0),
                        }),
                    },
                );
            }
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
    ) -> Result<(), WbblWebappGraphStoreError> {
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

    pub fn replace_node(
        &mut self,
        node: &NewWbblWebappNode,
    ) -> Result<(), WbblWebappGraphStoreError> {
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
    ) -> Result<(), WbblWebappGraphStoreError> {
        {
            let source_uuid =
                uuid::Uuid::from_str(source).map_err(|_| WbblWebappGraphStoreError::MalformedId)?;
            let target_uuid =
                uuid::Uuid::from_str(target).map_err(|_| WbblWebappGraphStoreError::MalformedId)?;

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

    fn get_selection_snapshot(&self) -> Result<WbblWebappGraphSnapshot, WbblWebappGraphStoreError> {
        use std::collections::HashSet;

        let selected_nodes: HashSet<u128> = self
            .get_locally_selected_nodes()?
            .iter()
            .map(|n| uuid::Uuid::parse_str(n).unwrap().as_u128())
            .collect();
        let selected_edges: HashSet<u128> = self
            .get_locally_selected_edges()?
            .iter()
            .map(|e| uuid::Uuid::parse_str(e).unwrap().as_u128())
            .collect();
        let mut snapshot = self.get_snapshot_raw()?;
        snapshot.edges = snapshot
            .edges
            .into_iter()
            .filter(|e| selected_edges.contains(&e.id))
            .collect();

        snapshot.nodes = snapshot
            .nodes
            .into_iter()
            .filter(|n| selected_nodes.contains(&n.id))
            .collect();
        Ok(snapshot)
    }

    fn get_node_snapshot(
        &self,
        node_id: &str,
    ) -> Result<WbblWebappGraphSnapshot, WbblWebappGraphStoreError> {
        let node_id = uuid::Uuid::parse_str(node_id)
            .map_err(|_| WbblWebappGraphStoreError::MalformedId)?
            .as_u128();

        let mut snapshot = self.get_snapshot_raw()?;
        snapshot.edges = vec![];

        snapshot.nodes = snapshot
            .nodes
            .into_iter()
            .filter(|n| n.id == node_id)
            .collect();
        Ok(snapshot)
    }

    pub fn duplicate(&mut self) -> Result<(), WbblWebappGraphStoreError> {
        let mut snapshot = self.get_selection_snapshot()?;
        snapshot.offset(&Vec2::new(200.0, 200.0));
        self.integrate_snapshot(None, &mut snapshot)?;
        Ok(())
    }

    pub fn duplicate_node(&mut self, node_id: &str) -> Result<(), WbblWebappGraphStoreError> {
        let mut snapshot = self.get_node_snapshot(node_id)?;
        snapshot.offset(&Vec2::new(200.0, 200.0));
        self.integrate_snapshot(None, &mut snapshot)?;
        Ok(())
    }

    #[cfg(web_sys_unstable_apis)]
    pub async fn copy_node(&mut self, node_id: &str) -> Result<(), WbblWebappGraphStoreError> {
        use crate::dot_converter::to_dot;
        let snapshot = self.get_node_snapshot(node_id)?;
        let clipboard_contents = to_dot(&snapshot);
        let window = web_sys::window().expect("Missing Window");
        if let Some(clipboard) = window.navigator().clipboard() {
            wasm_bindgen_futures::JsFuture::from(clipboard.write_text(&clipboard_contents))
                .await
                .map_err(|_| WbblWebappGraphStoreError::ClipboardFailure)?;
        };

        Ok(())
    }

    #[cfg(web_sys_unstable_apis)]
    pub async fn copy(&self) -> Result<(), WbblWebappGraphStoreError> {
        use crate::dot_converter::to_dot;
        let snapshot = self.get_selection_snapshot()?;
        let clipboard_contents = to_dot(&snapshot);
        let window = web_sys::window().expect("Missing Window");
        if let Some(clipboard) = window.navigator().clipboard() {
            wasm_bindgen_futures::JsFuture::from(clipboard.write_text(&clipboard_contents))
                .await
                .map_err(|_| WbblWebappGraphStoreError::ClipboardFailure)?;
        };

        Ok(())
    }

    fn integrate_snapshot(
        &mut self,
        position: Option<Vec2>,
        snapshot: &mut WbblWebappGraphSnapshot,
    ) -> Result<(), WbblWebappGraphStoreError> {
        {
            let mut mut_transaction = self.graph.transact_mut_with(self.graph.client_id());
            snapshot.reassign_ids();
            snapshot.filter_out_output_ports();
            if let Some(position) = position {
                snapshot.recenter(&position);
            }
            for node in snapshot.nodes.iter() {
                node.encode(&mut mut_transaction, &mut self.nodes)?;
            }
            for edge in snapshot.edges.iter() {
                edge.encode(&mut mut_transaction, &mut self.edges, &mut self.nodes)?;
            }
        }
        self.emit(true)?;
        Ok(())
    }

    #[cfg(web_sys_unstable_apis)]
    pub async fn paste(
        &mut self,
        cursor_position: &[f32],
    ) -> Result<(), WbblWebappGraphStoreError> {
        use web_sys::js_sys::JsString;

        use crate::dot_converter::from_dot;

        let window = web_sys::window().expect("Missing Window");
        if let Some(clipboard) = window.navigator().clipboard() {
            let value = wasm_bindgen_futures::JsFuture::from(clipboard.read_text())
                .await
                .map_err(|_| WbblWebappGraphStoreError::ClipboardFailure)?;
            let str: String = value.dyn_into::<JsString>().unwrap().into();
            let mut snapshot =
                from_dot(&str).map_err(|_| WbblWebappGraphStoreError::ClipboardContentsFailure)?;
            let position = Vec2::from_slice(cursor_position);
            self.integrate_snapshot(Some(position), &mut snapshot)?;
            return Ok(());
        };
        Err(WbblWebappGraphStoreError::ClipboardNotFound)
    }

    pub fn set_edge_selection(
        &mut self,
        edge_id: &str,
        selected: bool,
    ) -> Result<(), WbblWebappGraphStoreError> {
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

    pub fn get_locally_selected_nodes(&self) -> Result<Vec<String>, WbblWebappGraphStoreError> {
        let txn = self.graph.transact();
        let id = self.graph.client_id().to_string();
        let maybe_selection_for_current_user = match get_map(&id, &txn, &self.node_selections) {
            Ok(selection) => Ok(Some(selection)),
            Err(WbblWebappGraphStoreError::NotFound) => Ok(None),
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

    pub fn get_locally_selected_edges(&self) -> Result<Vec<String>, WbblWebappGraphStoreError> {
        let txn = self.graph.transact();
        let id = self.graph.client_id().to_string();
        let maybe_selection_for_current_user = match get_map(&id, &txn, &self.edge_selections) {
            Ok(selection) => Ok(Some(selection)),
            Err(WbblWebappGraphStoreError::NotFound) => Ok(None),
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
    ) -> Result<JsValue, WbblWebappGraphStoreError> {
        match (
            serde_wasm_bindgen::from_value::<AbstractDataType>(type_a),
            serde_wasm_bindgen::from_value::<AbstractDataType>(type_b),
        ) {
            (Ok(a), Ok(b)) => Ok(serde_wasm_bindgen::to_value(
                &AbstractDataType::get_most_specific_type(a, b),
            )
            .unwrap()),
            (Ok(_), Err(_)) => Err(WbblWebappGraphStoreError::UnexpectedStructure),
            (Err(_), Ok(_)) => Err(WbblWebappGraphStoreError::UnexpectedStructure),
            (Err(_), Err(_)) => Err(WbblWebappGraphStoreError::UnexpectedStructure),
        }
    }

    pub fn link_to_preview(&mut self, node_id: &str) -> Result<(), WbblWebappGraphStoreError> {
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
                        (_, _) => Err(WbblWebappGraphStoreError::UnexpectedStructure),
                    }
                }
                _ => Err(WbblWebappGraphStoreError::UnexpectedStructure),
            }?;
            let preview_node =
                NewWbblWebappNode::new(position.0 + 350.0, position.1, WbblWebappNodeType::Preview);
            preview_node.encode(mut_transaction, &mut self.nodes)?;
            let source = uuid::Uuid::from_str(node_id)
                .map_err(|_| WbblWebappGraphStoreError::MalformedId)?;
            let edge = WbblWebappEdge::new(&(source.as_u128()), &preview_node.id, 0, 0);
            edge.encode(mut_transaction, &mut self.edges, &self.nodes)?;
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
