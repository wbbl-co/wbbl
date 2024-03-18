use std::{collections::HashMap, sync::Arc};

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use web_sys::js_sys;
use yrs::{types::ToJson, Map, MapPrelim, MapRef, Transact, TransactionMut};

const GRAPH_YRS_NODES_MAP_KEY: &str = "nodes";
const GRAPH_YRS_EDGES_MAP_KEY: &str = "edges";

#[derive(Clone, Serialize, Deserialize)]
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
pub struct WbblWebappGraphStore {
    next_listener_handle: u32,
    listeners: HashMap<u32, js_sys::Function>,
    undo_manager: yrs::UndoManager,
    graph: Arc<yrs::Doc>,
    nodes: yrs::MapRef,
    edges: yrs::MapRef,
    computed_node_sizes: HashMap<String, WbbleComputedNodeSize>,
}

#[wasm_bindgen]
#[derive(Copy, Clone, Deserialize, Serialize)]
pub struct WbblePosition {
    pub x: f64,
    pub y: f64,
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct WbbleComputedNodeSize {
    pub width: Option<f64>,
    pub height: Option<f64>,
    #[serde(alias = "positionAbsolute")]
    pub position_absolute: Option<WbblePosition>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct WbblWebappNode {
    pub id: String,
    pub position: WbblePosition,
    #[serde(alias = "type")]
    pub type_name: String,
    pub data: Any,
    pub computed: Option<WbbleComputedNodeSize>,
    pub dragging: bool,
    pub resizing: bool,
    pub selected: bool,
    pub selectable: bool,
    pub connectable: bool,
    pub deletable: bool,
}

#[wasm_bindgen]

pub struct NewWbblWebappNode {
    id: String,
    position: WbblePosition,
    type_name: String,
    data: js_sys::Object,
}

impl NewWbblWebappNode {
    fn encode_data(
        &self,
        transaction: &mut TransactionMut,
        node_ref: &mut yrs::MapRef,
    ) -> Result<Any, WbblWebappGraphStoreError> {
        let mut map: HashMap<String, yrs::Any> = HashMap::new();

        for kv in js_sys::Object::entries(&self.data) {
            let arr = kv.into();
            let key = js_sys::Array::get(&arr, 0).as_string();
            let value = js_sys::Array::get(&arr, 1);
            // Probably not that fast, but whatever
            if let Some(key) = key {
                if let Ok(Ok(value)) = js_sys::JSON::stringify(&value)
                    .map(|json| yrs::Any::from_json(&ToString::to_string(&json)))
                {
                    map.insert(key.to_owned(), value);
                }
            }
        }
        let prelim_map = MapPrelim::from(map.clone());
        node_ref.insert(transaction, "data", prelim_map);

        Ok((&yrs::Any::Map(map.into())).into())
    }

    pub(crate) fn encode(
        &self,
        transaction: &mut TransactionMut,
        nodes: &mut yrs::MapRef,
    ) -> Result<WbblWebappNode, WbblWebappGraphStoreError> {
        let data = {
            let prelim_map: yrs::MapPrelim<yrs::Any> = yrs::MapPrelim::new();
            let mut node_ref = nodes.insert(transaction, self.id.clone(), prelim_map);
            node_ref.insert(transaction, "type", self.type_name.clone());
            node_ref.insert(transaction, "x", self.position.x);
            node_ref.insert(transaction, "y", self.position.y);
            node_ref.insert(transaction, "dragging", false);
            node_ref.insert(transaction, "resizing", false);
            node_ref.insert(transaction, "selected", false);
            self.encode_data(transaction, &mut node_ref)
        }?;

        Ok(WbblWebappNode {
            id: self.id.clone(),
            position: self.position,
            type_name: self.type_name.clone(),
            data,
            computed: None,
            dragging: false,
            resizing: false,
            selected: false,
            selectable: true,
            deletable: true,
            connectable: true,
        })
    }
}
#[wasm_bindgen]
impl NewWbblWebappNode {
    pub fn new(
        position_x: f64,
        position_y: f64,
        type_name: &str,
        data: js_sys::Object,
    ) -> NewWbblWebappNode {
        NewWbblWebappNode {
            id: uuid::Uuid::new_v4().to_string(),
            position: WbblePosition {
                x: position_x,
                y: position_y,
            },
            type_name: type_name.to_owned(),
            data,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct WbblWebappEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub source_handle: String,
    pub target_handle: String,
    pub deletable: bool,
    pub selectable: bool,
    pub selected: bool,
    pub updatable: bool,
}

impl WbblWebappEdge {
    pub fn new(
        source: &str,
        target: &str,
        source_handle: &str,
        target_handle: &str,
    ) -> WbblWebappEdge {
        WbblWebappEdge {
            id: uuid::Uuid::new_v4().to_string(),
            source: source.to_owned(),
            target: target.to_owned(),
            source_handle: source_handle.to_owned(),
            target_handle: target_handle.to_owned(),
            deletable: true,
            selected: false,
            selectable: true,
            updatable: true,
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
        key: &String,
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
        Ok(WbblWebappNode {
            id: key.clone(),
            position: WbblePosition { x, y },
            computed: None,
            type_name: type_name.to_owned(),
            dragging,
            resizing,
            selected,
            data: (&data.to_json(txn)).into(),
            connectable: true,
            selectable: true,
            deletable: &type_name != "output",
        })
    }
}

impl WbblWebappEdge {
    pub(crate) fn decode<Txn: yrs::ReadTxn>(
        key: String,
        txn: &Txn,
        map: &yrs::MapRef,
    ) -> Result<WbblWebappEdge, WbblWebappGraphStoreError> {
        let source = get_atomic_string("source", txn, map)?;
        let target = get_atomic_string("target", txn, map)?;
        let source_handle = get_atomic_string("source_handle", txn, map)?;
        let selected = get_bool("selected", txn, map)?;
        let target_handle = get_atomic_string("source_handle", txn, map)?;

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
            yrs::Any::String(self.source.clone().into()),
        );
        prelim_map.insert(
            "target".to_owned(),
            yrs::Any::String(self.target.clone().into()),
        );
        prelim_map.insert(
            "source_handle".to_owned(),
            yrs::Any::String(self.source_handle.clone().into()),
        );
        prelim_map.insert(
            "target_handle".to_owned(),
            yrs::Any::String(self.target_handle.clone().into()),
        );
        prelim_map.insert("selected".to_owned(), yrs::Any::Bool(self.selected));

        map.insert(txn, self.id.clone(), yrs::MapPrelim::from(prelim_map));
        Ok(())
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct WbblWebappGraphSnapshot {
    pub nodes: Vec<WbblWebappNode>,
    pub edges: Vec<WbblWebappEdge>,
}

#[wasm_bindgen]
#[derive(Debug)]
pub enum WbblWebappGraphStoreError {
    UnexpectedStructure,
    FailedToUndo,
    FailedToRedo,
    FailedToEmit,
    NotFound,
}

#[wasm_bindgen]
impl WbblWebappGraphStore {
    pub fn empty() -> Self {
        let graph = yrs::Doc::new();
        let nodes = graph.get_or_insert_map(GRAPH_YRS_NODES_MAP_KEY.to_owned());
        let edges = graph.get_or_insert_map(GRAPH_YRS_EDGES_MAP_KEY.to_owned());
        let mut undo_manager = yrs::UndoManager::new(&graph, &nodes);
        undo_manager.include_origin(graph.client_id()); // only track changes originating from local peer
        undo_manager.expand_scope(&edges);
        WbblWebappGraphStore {
            next_listener_handle: 0,
            listeners: HashMap::new(),
            undo_manager,
            graph: Arc::new(graph),
            nodes,
            edges,
            computed_node_sizes: HashMap::new(),
        }
    }

    pub fn emit(&self) -> Result<(), WbblWebappGraphStoreError> {
        for listener in self.listeners.values() {
            listener
                .call0(&JsValue::UNDEFINED)
                .map_err(|_| WbblWebappGraphStoreError::FailedToEmit)?;
        }
        Ok(())
    }

    pub fn subscribe(&mut self, subscriber: js_sys::Function) -> u32 {
        let handle = self.next_listener_handle;
        self.listeners.insert(handle, subscriber);
        self.next_listener_handle = self.next_listener_handle + 1;
        handle
    }

    pub fn unsubscribe(&mut self, handle: u32) {
        self.listeners.remove(&handle);
    }

    pub fn undo(&mut self) -> Result<bool, WbblWebappGraphStoreError> {
        let result = self
            .undo_manager
            .undo()
            .map_err(|_| WbblWebappGraphStoreError::FailedToUndo)?;
        if result {
            self.emit()?;
        }
        Ok(result)
    }

    pub fn redo(&mut self) -> Result<bool, WbblWebappGraphStoreError> {
        let result = self
            .undo_manager
            .redo()
            .map_err(|_| WbblWebappGraphStoreError::FailedToRedo)?;
        if result {
            self.emit()?;
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
        let snapshot = self.get_snapshot_raw()?;
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
        self.emit()?;

        Ok(())
    }

    pub fn remove_edge(&mut self, edge_id: &str) -> Result<(), WbblWebappGraphStoreError> {
        {
            let mut mut_transaction = self.graph.transact_mut();
            self.edges.remove(&mut mut_transaction, edge_id);
        }

        // Important that emit is called here. Rather than before the drop
        // as this could trigger a panic as the store value may be read
        self.emit()?;

        Ok(())
    }

    pub fn add_node(&mut self, node: NewWbblWebappNode) -> Result<(), WbblWebappGraphStoreError> {
        {
            let mut mut_transaction = self.graph.transact_mut();
            node.encode(&mut mut_transaction, &mut self.nodes)
        }?;

        // Important that emit is called here. Rather than before the drop
        // as this could trigger a panic as the store value may be read
        self.emit()?;

        Ok(())
    }

    fn get_snapshot_raw(&self) -> Result<WbblWebappGraphSnapshot, WbblWebappGraphStoreError> {
        let read_txn = self.graph.transact();
        let mut nodes: Vec<WbblWebappNode> = Vec::new();
        let mut edges: Vec<WbblWebappEdge> = Vec::new();
        for node in self.nodes.iter(&read_txn) {
            let key = node.0.to_owned();
            let node_values = node.1;
            let mut node: WbblWebappNode = match node_values {
                yrs::Value::YMap(map) => WbblWebappNode::decode(&key, &read_txn, &map),
                _ => Err(WbblWebappGraphStoreError::UnexpectedStructure),
            }?;
            node.computed = self.computed_node_sizes.get(&key).map(|s| s.clone());
            nodes.push(node);
        }

        for edge in self.edges.iter(&read_txn) {
            let key = edge.0.to_owned();
            let edge_values = edge.1;
            let edge: WbblWebappEdge = match edge_values {
                yrs::Value::YMap(map) => WbblWebappEdge::decode(key, &read_txn, &map),
                _ => Err(WbblWebappGraphStoreError::UnexpectedStructure),
            }?;
            edges.push(edge);
        }

        nodes.sort_by_key(|n| n.id.clone());
        edges.sort_by_key(|n| n.id.clone());
        Ok(WbblWebappGraphSnapshot { nodes, edges })
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
        if let Some(maybe_computed) = self.computed_node_sizes.get_mut(node_id) {
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
        self.emit()?;

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
            if let Some(maybe_computed) = self.computed_node_sizes.get_mut(node_id) {
                maybe_computed.position_absolute = Some(WbblePosition {
                    x: position_absolute_x.unwrap_or(0.0),
                    y: position_absolute_y.unwrap_or(0.0),
                })
            } else {
                self.computed_node_sizes.insert(
                    node_id.to_owned(),
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
        self.emit()?;

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
        self.emit()?;

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
        self.emit()?;

        Ok(())
    }

    pub fn add_edge(
        &mut self,
        source: &str,
        target: &str,
        source_handle: &str,
        target_handle: &str,
    ) -> Result<(), WbblWebappGraphStoreError> {
        {
            let edge = WbblWebappEdge::new(source, target, source_handle, target_handle);

            let mut mut_transaction = self.graph.transact_mut();
            edge.encode(&mut mut_transaction, &self.edges)?;
        }

        // Important that emit is called here. Rather than before the drop
        // as this could trigger a panic as the store value may be read
        self.emit()?;

        Ok(())
    }

    pub fn replace_edge(
        &mut self,
        edge_id: &str,
        source: &str,
        target: &str,
        source_handle: &str,
        target_handle: &str,
        selected: bool,
    ) -> Result<(), WbblWebappGraphStoreError> {
        {
            let mut mut_transaction = self.graph.transact_mut();
            let edge = WbblWebappEdge {
                id: edge_id.to_owned(),
                source: source.to_owned(),
                target: target.to_owned(),
                source_handle: source_handle.to_owned(),
                target_handle: target_handle.to_owned(),
                deletable: true,
                selectable: true,
                updatable: true,
                selected,
            };

            edge.encode(&mut mut_transaction, &self.edges)?;
        }

        // Important that emit is called here. Rather than before the drop
        // as this could trigger a panic as the store value may be read
        self.emit()?;

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
                _ => Err(WbblWebappGraphStoreError::UnexpectedStructure),
            }?;
        }
        // Important that emit is called here. Rather than before the drop
        // as this could trigger a panic as the store value may be read
        self.emit()?;

        Ok(())
    }
}
