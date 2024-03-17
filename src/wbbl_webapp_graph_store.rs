use std::{collections::HashMap, sync::Arc};

use wasm_bindgen::prelude::*;
use web_sys::js_sys;
use yrs::{types::ToJson, Map, MapRef, Observable, Transact};

const GRAPH_YRS_NODES_MAP_KEY: &str = "nodes";
const GRAPH_YRS_EDGES_MAP_KEY: &str = "edges";

#[wasm_bindgen]
pub struct WbblWebappGraphStore {
    next_listener_handle: u32,
    listeners: HashMap<u32, js_sys::Function>,
    undo_manager: yrs::UndoManager,
    graph: Arc<yrs::Doc>,
    nodes: yrs::MapRef,
    edges: yrs::MapRef,
    cached_snapshot: Option<WbblWebappGraphSnapshot>,
}

#[wasm_bindgen]
#[derive(Copy, Clone)]
pub struct WbblePosition {
    pub x: f64,
    pub y: f64,
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct WbblWebappNode {
    #[wasm_bindgen(getter_with_clone)]
    pub id: String,
    pub position: WbblePosition,
    #[wasm_bindgen(getter_with_clone, js_name=type)]
    pub type_name: String,
    #[wasm_bindgen(getter_with_clone)]
    pub data: WbblWebappData,
    graph: Arc<yrs::Doc>,
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct WbblWebappEdge {
    #[wasm_bindgen(getter_with_clone)]
    pub id: String,
    #[wasm_bindgen(getter_with_clone, js_name=type)]
    pub type_name: String,
    #[wasm_bindgen(getter_with_clone)]
    pub data: WbblWebappData,
    graph: Arc<yrs::Doc>,
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct WbblWebappData {
    graph: Arc<yrs::Doc>,
    data: yrs::MapRef,
}

#[wasm_bindgen]
impl WbblWebappData {
    pub fn eq(a: &WbblWebappData, b: &WbblWebappData) -> bool {
        let read_transact = a.graph.transact();
        a.data
            .to_json(&read_transact)
            .eq(&b.data.to_json(&read_transact))
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

impl WbblWebappNode {
    pub(crate) fn decode<Txn: yrs::ReadTxn>(
        graph: Arc<yrs::Doc>,
        key: String,
        txn: &Txn,
        map: &yrs::MapRef,
    ) -> Result<WbblWebappNode, WbblWebappGraphStoreError> {
        let type_name: String = get_atomic_string("type", txn, map)?;
        let data = get_map("data", txn, map)?;
        let x = get_float_64("x", txn, map)?;
        let y = get_float_64("y", txn, map)?;

        let wrapped_data = WbblWebappData {
            data,
            graph: graph.clone(),
        };
        Ok(WbblWebappNode {
            id: key,
            position: WbblePosition { x, y },
            type_name,
            data: wrapped_data,
            graph,
        })
    }
}

impl WbblWebappEdge {
    pub(crate) fn decode<Txn: yrs::ReadTxn>(
        graph: Arc<yrs::Doc>,
        key: String,
        txn: &Txn,
        map: &yrs::MapRef,
    ) -> Result<WbblWebappEdge, WbblWebappGraphStoreError> {
        let type_name: String = get_atomic_string("type", txn, map)?;
        let data = get_map("data", txn, map)?;
        let wrapped_data = WbblWebappData {
            data,
            graph: graph.clone(),
        };
        Ok(WbblWebappEdge {
            id: key,
            type_name,
            data: wrapped_data,
            graph,
        })
    }
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
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
            cached_snapshot: None,
        }
    }

    fn emit(&self) -> Result<(), WbblWebappGraphStoreError> {
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
            self.invalidate_snapshot();
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
            self.invalidate_snapshot();
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

    fn invalidate_snapshot(&mut self) {
        self.cached_snapshot = None;
    }

    pub fn get_snapshot(&mut self) -> Result<WbblWebappGraphSnapshot, WbblWebappGraphStoreError> {
        match &self.cached_snapshot {
            Some(result) => Ok(result.clone()),
            None => {
                let snapshot = self.get_snapshot_raw()?;
                self.cached_snapshot = Some(snapshot.clone());
                Ok(snapshot)
            }
        }
    }

    fn get_snapshot_raw(&self) -> Result<WbblWebappGraphSnapshot, WbblWebappGraphStoreError> {
        let read_txn = self.graph.transact();
        let mut nodes: Vec<WbblWebappNode> = Vec::new();
        let mut edges: Vec<WbblWebappEdge> = Vec::new();
        for node in self.nodes.iter(&read_txn) {
            let key = node.0.to_owned();
            let node_values = node.1;
            let node: WbblWebappNode = match node_values {
                yrs::Value::YMap(map) => {
                    WbblWebappNode::decode(self.graph.clone(), key, &read_txn, &map)
                }
                _ => Err(WbblWebappGraphStoreError::UnexpectedStructure),
            }?;
            nodes.push(node);
        }

        for edge in self.edges.iter(&read_txn) {
            let key = edge.0.to_owned();
            let edge_values = edge.1;
            let edge: WbblWebappEdge = match edge_values {
                yrs::Value::YMap(map) => {
                    WbblWebappEdge::decode(self.graph.clone(), key, &read_txn, &map)
                }
                _ => Err(WbblWebappGraphStoreError::UnexpectedStructure),
            }?;
            edges.push(edge);
        }

        Ok(WbblWebappGraphSnapshot { nodes, edges })
    }
}
