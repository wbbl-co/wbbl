use std::{cell::RefCell, collections::HashMap, str::FromStr, sync::Arc};

use wasm_bindgen::prelude::*;
use web_sys::{js_sys, MessageEvent, Worker};
use yrs::{types::ToJson, Map, MapPrelim, MapRef, Transact, TransactionMut};

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
    edges: yrs::MapRef,
    computed_node_sizes: HashMap<u128, WbbleComputedNodeSize>,
    graph_worker: Worker,
    worker_responder: Closure<dyn FnMut(MessageEvent) -> ()>,
    computed_types: Arc<RefCell<HashMap<PortId, AbstractDataType>>>,
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct NewWbblWebappNode {
    id: u128,
    position: WbblePosition,
    node_type: WbblWebappNodeType,
    data: HashMap<String, Any>,
}

impl NewWbblWebappNode {
    fn encode_data(
        &self,
        transaction: &mut TransactionMut,
        node_ref: &mut yrs::MapRef,
    ) -> Result<HashMap<String, Any>, WbblWebappGraphStoreError> {
        let mut map: HashMap<String, yrs::Any> = HashMap::new();
        for (key, value) in self.data.iter() {
            // Probably not that fast, but whatever
            let yrs_value = value.to_yrs();
            map.insert(key.to_owned(), yrs_value);
        }
        let prelim_map = MapPrelim::from(map.clone());
        node_ref.insert(transaction, "data", prelim_map);

        Ok(map.iter().map(|(k, v)| (k.clone(), v.into())).collect())
    }

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
            node_ref.insert(transaction, "selected", false);
            self.encode_data(transaction, &mut node_ref)
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
        _ => Err(WbblWebappGraphStoreError::UnexpectedStructure),
    }
}

impl WbblWebappNode {
    pub(crate) fn decode<Txn: yrs::ReadTxn>(
        key: &u128,
        txn: &Txn,
        map: &yrs::MapRef,
    ) -> Result<WbblWebappNode, WbblWebappGraphStoreError> {
        let type_name: String = get_atomic_string("type", txn, map)?;
        let data = get_map("data", txn, map)?;
        let x = get_float_64("x", txn, map)?;
        let y = get_float_64("y", txn, map)?;
        let dragging = get_bool("dragging", txn, map)?;
        let resizing = get_bool("resizing", txn, map)?;
        let selected = get_bool("selected", txn, map)?;
        let data = &data.to_json(txn);
        let data = match data {
            yrs::Any::Map(hash_map) => Ok(hash_map.clone()),
            _ => Err(WbblWebappGraphStoreError::UnexpectedStructure),
        }?;
        let node_type = match from_type_name(&type_name) {
            Some(typ) => Ok(typ),
            None => Err(WbblWebappGraphStoreError::UnknownNodeType),
        }?;
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
}

impl WbblWebappEdge {
    pub(crate) fn decode<Txn: yrs::ReadTxn>(
        key: u128,
        txn: &Txn,
        map: &yrs::MapRef,
    ) -> Result<WbblWebappEdge, WbblWebappGraphStoreError> {
        let source = get_atomic_u128_from_string("source", txn, map)?;
        let target = get_atomic_u128_from_string("target", txn, map)?;
        let source_handle = get_atomic_bigint("source_handle", txn, map)?;
        let selected = get_bool("selected", txn, map)?;
        let target_handle = get_atomic_bigint("target_handle", txn, map)?;

        Ok(WbblWebappEdge {
            id: key,
            source,
            target,
            source_handle,
            target_handle,
            deletable: true,
            selectable: true,
            updatable: true,
            selected,
        })
    }

    pub(crate) fn encode(
        &self,
        txn: &mut TransactionMut,
        map: &yrs::MapRef,
    ) -> Result<(), WbblWebappGraphStoreError> {
        let mut prelim_map: HashMap<String, yrs::Any> = HashMap::new();
        prelim_map.insert(
            "source".to_owned(),
            yrs::Any::String(uuid::Uuid::from_u128(self.source).to_string().into()),
        );
        prelim_map.insert(
            "target".to_owned(),
            yrs::Any::String(uuid::Uuid::from_u128(self.target).to_string().into()),
        );
        prelim_map.insert(
            "source_handle".to_owned(),
            yrs::Any::BigInt(self.source_handle),
        );
        prelim_map.insert(
            "target_handle".to_owned(),
            yrs::Any::BigInt(self.target_handle),
        );
        prelim_map.insert("selected".to_owned(), yrs::Any::Bool(self.selected));

        map.insert(
            txn,
            uuid::Uuid::from_u128(self.id).to_string(),
            yrs::MapPrelim::from(prelim_map),
        );
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
}

#[wasm_bindgen]
impl WbblWebappGraphStore {
    pub fn empty(graph_worker: Worker) -> Self {
        let graph = yrs::Doc::new();
        let nodes = graph.get_or_insert_map(GRAPH_YRS_NODES_MAP_KEY.to_owned());
        let edges = graph.get_or_insert_map(GRAPH_YRS_EDGES_MAP_KEY.to_owned());
        let mut undo_manager = yrs::UndoManager::new(&graph, &nodes);
        undo_manager.include_origin(graph.client_id()); // only track changes originating from local peer
        undo_manager.expand_scope(&edges);
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
            computed_node_sizes: HashMap::new(),
            computed_types: computed_types.clone(),
            graph_worker: graph_worker.clone(),
            worker_responder,
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
        store.emit(true).unwrap();
        store
    }

    pub fn emit(&self, should_publish_to_worker: bool) -> Result<(), WbblWebappGraphStoreError> {
        for (_, listener) in self.listeners.borrow().iter() {
            listener
                .call0(&JsValue::UNDEFINED)
                .map_err(|_| WbblWebappGraphStoreError::FailedToEmit)?;
        }
        if should_publish_to_worker {
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
            let mut mut_transaction = self.graph.transact_mut();
            self.nodes.remove(&mut mut_transaction, node_id);
        }

        // Important that emit is called here. Rather than before the drop
        // as this could trigger a panic as the store value may be read
        self.emit(true)?;

        Ok(())
    }

    pub fn remove_edge(&mut self, edge_id: &str) -> Result<(), WbblWebappGraphStoreError> {
        {
            let mut mut_transaction = self.graph.transact_mut();
            self.edges.remove(&mut mut_transaction, edge_id);
        }

        // Important that emit is called here. Rather than before the drop
        // as this could trigger a panic as the store value may be read
        self.emit(true)?;

        Ok(())
    }

    pub fn add_node(&mut self, node: NewWbblWebappNode) -> Result<(), WbblWebappGraphStoreError> {
        {
            let mut mut_transaction = self.graph.transact_mut();
            node.encode(&mut mut_transaction, &mut self.nodes)
        }?;

        // Important that emit is called here. Rather than before the drop
        // as this could trigger a panic as the store value may be read
        self.emit(true)?;

        Ok(())
    }

    fn get_snapshot_raw(&self) -> Result<WbblWebappGraphSnapshot, WbblWebappGraphStoreError> {
        let read_txn = self.graph.transact();
        let mut nodes: Vec<WbblWebappNode> = Vec::new();
        let mut edges: Vec<WbblWebappEdge> = Vec::new();
        for node in self.nodes.iter(&read_txn) {
            let key = node.0.to_owned();
            let node_values = node.1;
            let key =
                uuid::Uuid::from_str(&key).map_err(|_| WbblWebappGraphStoreError::MalformedId)?;
            let mut node: WbblWebappNode = match node_values {
                yrs::Value::YMap(map) => WbblWebappNode::decode(&key.as_u128(), &read_txn, &map),
                _ => Err(WbblWebappGraphStoreError::UnexpectedStructure),
            }?;
            node.computed = self
                .computed_node_sizes
                .get(&key.as_u128())
                .map(|s| s.clone());
            nodes.push(node);
        }

        for edge in self.edges.iter(&read_txn) {
            let key = edge.0.to_owned();
            let edge_values = edge.1;
            let key =
                uuid::Uuid::from_str(&key).map_err(|_| WbblWebappGraphStoreError::MalformedId)?;

            let edge: WbblWebappEdge = match edge_values {
                yrs::Value::YMap(map) => WbblWebappEdge::decode(key.as_u128(), &read_txn, &map),
                _ => Err(WbblWebappGraphStoreError::UnexpectedStructure),
            }?;
            edges.push(edge);
        }

        nodes.sort_by_key(|n| n.id.clone());
        edges.sort_by_key(|e| e.id.clone());
        Ok(WbblWebappGraphSnapshot {
            id: self.id,
            nodes,
            edges,
            computed_types: None,
        })
    }

    pub fn set_computed_node_dimension(
        &mut self,
        node_id: &str,
        width: Option<f64>,
        height: Option<f64>,
        resizing: Option<bool>,
    ) -> Result<(), WbblWebappGraphStoreError> {
        {
            let mut mut_transaction = self.graph.transact_mut();
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
            let mut mut_transaction = self.graph.transact_mut();

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
            let mut mut_transaction = self.graph.transact_mut();

            match self.nodes.get(&mut_transaction, node_id) {
                Some(yrs::Value::YMap(node_ref)) => {
                    node_ref.insert(&mut mut_transaction, "selected", yrs::Any::Bool(selected));
                    Ok(())
                }
                _ => Err(WbblWebappGraphStoreError::UnexpectedStructure),
            }?;
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
            let mut mut_transaction = self.graph.transact_mut();
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
            let source =
                uuid::Uuid::from_str(source).map_err(|_| WbblWebappGraphStoreError::MalformedId)?;
            let target =
                uuid::Uuid::from_str(target).map_err(|_| WbblWebappGraphStoreError::MalformedId)?;

            let edge = WbblWebappEdge::new(
                &(source.as_u128()),
                &(target.as_u128()),
                source_handle,
                target_handle,
            );

            let mut mut_transaction = self.graph.transact_mut();
            edge.encode(&mut mut_transaction, &self.edges)?;
        }

        // Important that emit is called here. Rather than before the drop
        // as this could trigger a panic as the store value may be read
        self.emit(true)?;

        Ok(())
    }

    pub fn replace_edge(
        &mut self,
        edge_id: &str,
        source: &str,
        target: &str,
        source_handle: i64,
        target_handle: i64,
        selected: bool,
    ) -> Result<(), WbblWebappGraphStoreError> {
        {
            let mut mut_transaction = self.graph.transact_mut();
            let edge_id = uuid::Uuid::from_str(edge_id)
                .map_err(|_| WbblWebappGraphStoreError::MalformedId)?;
            let source =
                uuid::Uuid::from_str(source).map_err(|_| WbblWebappGraphStoreError::MalformedId)?;
            let target =
                uuid::Uuid::from_str(target).map_err(|_| WbblWebappGraphStoreError::MalformedId)?;
            let edge = WbblWebappEdge {
                id: edge_id.as_u128(),
                source: source.as_u128(),
                target: target.as_u128(),
                source_handle,
                target_handle,
                deletable: true,
                selectable: true,
                updatable: true,
                selected,
            };

            edge.encode(&mut mut_transaction, &self.edges)?;
        }

        // Important that emit is called here. Rather than before the drop
        // as this could trigger a panic as the store value may be read
        self.emit(true)?;

        Ok(())
    }

    pub fn set_edge_selection(
        &mut self,
        edge_id: &str,
        selected: bool,
    ) -> Result<(), WbblWebappGraphStoreError> {
        {
            let mut mut_transaction = self.graph.transact_mut();
            match self.edges.get(&mut_transaction, edge_id) {
                Some(yrs::Value::YMap(edge_ref)) => {
                    edge_ref.insert(&mut mut_transaction, "selected", yrs::Any::Bool(selected));
                    Ok(())
                }
                None => Ok(()),
                _ => Err(WbblWebappGraphStoreError::UnexpectedStructure),
            }?;
        }
        // Important that emit is called here. Rather than before the drop
        // as this could trigger a panic as the store value may be read
        self.emit(false)?;

        Ok(())
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
