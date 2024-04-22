use crate::{
    animation_frame::{AnimationFrameHandler, AnimationFrameProcessor},
    builtin_geometry::BuiltInGeometry,
    data_types::AbstractDataType,
    graph_functions,
    graph_transfer_types::{GRAPH_YRS_EDGES_MAP_KEY, GRAPH_YRS_NODES_MAP_KEY},
    graph_types::{Edge, Graph, Node, PortId},
    log,
    preview_renderer::{PreviewRendererResources, SharedPreviewRendererResources},
    test_fragment_shader::make_fragment_shader_module,
    utils::try_into_u128,
    vertex_shader::make_vertex_shader_module,
    yrs_utils::get_map,
};
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, collections::HashMap, panic, rc::Rc, str::FromStr, sync::Arc};
use wasm_bindgen::prelude::*;
use web_sys::{DedicatedWorkerGlobalScope, OffscreenCanvas, Window};
use yrs::{updates::decoder::Decode, DeepObservable, Subscription, Transact, Update};

#[allow(unused)]
pub struct WbblGraphWebWorkerMain {
    doc: Arc<yrs::Doc>,
    graph: Arc<RefCell<Graph>>,
    preview_resources: HashMap<u128, Arc<RefCell<PreviewRendererResources>>>,
    shared_preview_resources: Arc<SharedPreviewRendererResources>,
    animation_frame_handler: Arc<RefCell<AnimationFrameHandler>>,
    worker_scope: Arc<DedicatedWorkerGlobalScope>,
    subscriptions: Vec<Subscription>,
    nodes: Arc<yrs::MapRef>,
    edges: Arc<yrs::MapRef>,
}

#[wasm_bindgen]
pub struct WbblGraphWebWorkerJsWrapper {
    main: Rc<RefCell<WbblGraphWebWorkerMain>>,
}

#[wasm_bindgen]
impl WbblGraphWebWorkerJsWrapper {
    pub async fn new(
        window: Window,
        worker_scope: DedicatedWorkerGlobalScope,
    ) -> WbblGraphWebWorkerJsWrapper {
        let animation_frame_handler: Arc<RefCell<AnimationFrameHandler>> =
            Arc::new(AnimationFrameHandler::new(window).into());
        let worker_scope = Arc::new(worker_scope);
        let main: Rc<RefCell<WbblGraphWebWorkerMain>> = Rc::new(
            WbblGraphWebWorkerMain::new(animation_frame_handler.clone(), worker_scope.clone())
                .await
                .into(),
        );
        {
            animation_frame_handler
                .as_ref()
                .borrow_mut()
                .set_processor(main.clone());
        }

        WbblGraphWebWorkerJsWrapper { main }
    }

    pub fn handle_message(&mut self, value: JsValue) -> Result<(), WbblGraphWebWorkerError> {
        self.main.as_ref().borrow_mut().handle_message(value)
    }

    #[cfg(target_arch = "wasm32")]
    pub fn register_canvas(
        &mut self,
        node_id: &str,
        offscreen_canvas: OffscreenCanvas,
    ) -> Result<(), WbblGraphWebWorkerError> {
        self.main
            .as_ref()
            .borrow_mut()
            .register_canvas(node_id, offscreen_canvas)
    }

    pub fn deregister_canvas(&mut self, node_id: &str) -> Result<(), WbblGraphWebWorkerError> {
        self.main.as_ref().borrow_mut().deregister_canvas(node_id)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum WbblGraphWebWorkerRequestMessage {
    Poll,
    ReceiveUpdate(Vec<u8>),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum WbblGraphWebWorkerResponseMessage {
    Ready,
    TypesUpdated(HashMap<PortId, AbstractDataType>),
    TypeUnificationFailure,
}

#[wasm_bindgen]
pub enum WbblGraphWebWorkerError {
    MalformedMessage,
    MalformedId,
    WebGpuError,
    CouldNotPostMessage,
    CouldNotUnifyTypes,
}

impl WbblGraphWebWorkerMain {
    pub async fn new(
        animation_frame_handler: Arc<RefCell<AnimationFrameHandler>>,
        worker_scope: Arc<DedicatedWorkerGlobalScope>,
    ) -> WbblGraphWebWorkerMain {
        panic::set_hook(Box::new(console_error_panic_hook::hook));
        let shared_preview_resources = SharedPreviewRendererResources::new()
            .await
            .expect("Expected success");
        let graph = Arc::new(RefCell::new(Graph {
            id: uuid::Uuid::new_v4().as_u128(),
            nodes: HashMap::new(),
            edges: HashMap::new(),
            input_ports: HashMap::new(),
            output_ports: HashMap::new(),
        }));
        let doc = Arc::new(yrs::Doc::new());
        let nodes = Arc::new(doc.get_or_insert_map(GRAPH_YRS_NODES_MAP_KEY.to_owned()));
        let edges = Arc::new(doc.get_or_insert_map(GRAPH_YRS_EDGES_MAP_KEY.to_owned()));

        let nodes_subscription = nodes.observe_deep({
            let graph = graph.clone();
            let nodes = nodes.clone();
            move |txn, evts| {
                for evt in evts.iter() {
                    if let yrs::types::Event::Map(map_evt) = evt {
                        let path = map_evt.path();
                        if path.len() == 0 {
                            // Add/Remove node
                            for (key, change) in map_evt.keys(txn) {
                                if let Ok(key) = try_into_u128(key) {
                                    let mut graph = graph.borrow_mut();
                                    match change {
                                        yrs::types::EntryChange::Inserted(new_node) => {
                                            match new_node {
                                                yrs::Value::YMap(new_node) => {
                                                    let _ =
                                                        Node::insert_new(txn, new_node, &mut graph)
                                                            .inspect_err(|err| {
                                                                log!("Nodes Err 3 {:?}", err)
                                                            });
                                                }
                                                _ => {}
                                            };
                                        }
                                        yrs::types::EntryChange::Removed(_) => {
                                            if let Some(prev_node) = graph.nodes.remove(&key) {
                                                for port_id in prev_node.port_ids() {
                                                    match port_id {
                                                        PortId::Output(port_id) => {
                                                            if let Some(output_port) =
                                                                graph.output_ports.remove(&port_id)
                                                            {
                                                                for edge_id in output_port
                                                                    .outgoing_edges
                                                                    .iter()
                                                                {
                                                                    graph.edges.remove(&edge_id);
                                                                }
                                                            }
                                                        }
                                                        PortId::Input(port_id) => {
                                                            if let Some(input_port) =
                                                                graph.input_ports.remove(&port_id)
                                                            {
                                                                if let Some(edge_id) =
                                                                    input_port.incoming_edge
                                                                {
                                                                    graph.edges.remove(&edge_id);
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        yrs::types::EntryChange::Updated(_, _) => {
                                            // Updates are not observed at this level, so ignore
                                        }
                                    }
                                }
                            }
                        } else if path.len() >= 1 {
                            match (path.get(0), path.get(1)) {
                                // Here we need to test if fields on the data were replaced
                                (
                                    Some(yrs::types::PathSegment::Key(key)),
                                    Some(yrs::types::PathSegment::Key(field)),
                                ) if field.to_string() == "data" => {
                                    // Only update node if data changed.
                                    // We don't care about the other node properties
                                    if let Ok(_) = try_into_u128(key) {
                                        let mut graph = graph.borrow_mut();
                                        if let Ok(new_node) = get_map(key, txn, &nodes) {
                                            let _ =
                                                Node::update_existing(txn, &new_node, &mut graph)
                                                    .inspect_err(|err| {
                                                        log!("Nodes Err 2 {:?}", err)
                                                    });
                                        }
                                    }
                                }
                                // Here we need to test if the data was replaced as a whole
                                (Some(yrs::types::PathSegment::Key(key)), None) => {
                                    if let Ok(_) = try_into_u128(key) {
                                        let mut graph = graph.borrow_mut();
                                        // Node updated
                                        let keys = map_evt.keys(txn);
                                        if let Some(data_change) = keys.get("data") {
                                            match data_change {
                                                yrs::types::EntryChange::Inserted(_) => {
                                                    // Do nothing. Should probably log here. Data should never be inserted but rather updated
                                                }
                                                yrs::types::EntryChange::Updated(_, _) => {
                                                    if let Ok(node) = get_map(key, txn, &nodes) {
                                                        let _ = Node::update_existing(
                                                            txn, &node, &mut graph,
                                                        )
                                                        .inspect_err(|err| {
                                                            log!("Nodes Err {:?}", err)
                                                        });
                                                    }
                                                }
                                                yrs::types::EntryChange::Removed(_) => {
                                                    // Do nothing. Should probably log here. Data should never be deleted
                                                }
                                            }
                                        }
                                    }
                                }
                                _ => {}
                            };
                        }
                    }
                }
            }
        });

        let edges_subscription = edges.observe_deep({
            let graph = graph.clone();
            move |txn, evts| {
                for evt in evts.iter() {
                    if let yrs::types::Event::Map(map_evt) = evt {
                        let path = map_evt.path();
                        if path.len() == 0 {
                            for (key, change) in map_evt.keys(txn).iter() {
                                if let Ok(edge_uuid) = try_into_u128(key) {
                                    match change {
                                        yrs::types::EntryChange::Inserted(yrs::Value::YMap(
                                            new_edge,
                                        )) => {
                                            let mut graph = graph.borrow_mut();
                                            let _ = Edge::insert_new(txn, &new_edge, &mut graph)
                                                .inspect_err(|err| log!("Edges Err {:?}", err));
                                        }
                                        yrs::types::EntryChange::Removed(_) => {
                                            let mut graph = graph.borrow_mut();
                                            if let Some(prev_edge) = graph.edges.remove(&edge_uuid)
                                            {
                                                let input_port_id = prev_edge.input_port;
                                                if let Some(input_port) =
                                                    graph.input_ports.get_mut(&input_port_id)
                                                {
                                                    input_port.incoming_edge = None;
                                                }

                                                let output_port_id = prev_edge.output_port;
                                                if let Some(output_port) =
                                                    graph.output_ports.get_mut(&output_port_id)
                                                {
                                                    output_port.outgoing_edges = output_port
                                                        .outgoing_edges
                                                        .iter()
                                                        .filter(|x| *x != &edge_uuid)
                                                        .cloned()
                                                        .collect()
                                                }
                                            }
                                        }
                                        _ => {
                                            // Ignored
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });

        let doc_subscription = doc
            .observe_after_transaction({
                let graph = graph.clone();
                let worker_scope = worker_scope.clone();
                move |_| {
                    match graph_functions::narrow_abstract_types(&graph.borrow()) {
                        Ok(types) => {
                            let _ = worker_scope
                                .post_message(
                                    &serde_wasm_bindgen::to_value(
                                        &WbblGraphWebWorkerResponseMessage::TypesUpdated(types),
                                    )
                                    .unwrap(),
                                )
                                .unwrap();
                        }
                        Err(_) => {
                            let _ = worker_scope
                                .post_message(
                                    &serde_wasm_bindgen::to_value(
                                        &WbblGraphWebWorkerResponseMessage::TypeUnificationFailure,
                                    )
                                    .unwrap(),
                                )
                                .unwrap();
                        }
                    };
                }
            })
            .unwrap();
        // doc.integrate(&mut txn, update);
        let result = WbblGraphWebWorkerMain {
            doc,
            graph,
            shared_preview_resources: shared_preview_resources.into(),
            preview_resources: HashMap::new().into(),
            animation_frame_handler,
            worker_scope: worker_scope.clone(),
            subscriptions: vec![nodes_subscription, edges_subscription, doc_subscription],
            nodes,
            edges,
        };
        result
    }

    fn post_message(
        &self,
        message: WbblGraphWebWorkerResponseMessage,
    ) -> Result<(), WbblGraphWebWorkerError> {
        self.worker_scope
            .post_message(&serde_wasm_bindgen::to_value(&message).unwrap())
            .map_err(|_| WbblGraphWebWorkerError::CouldNotPostMessage)
    }

    pub fn handle_message(&mut self, value: JsValue) -> Result<(), WbblGraphWebWorkerError> {
        let message = serde_wasm_bindgen::from_value::<WbblGraphWebWorkerRequestMessage>(value)
            .map_err(|_| WbblGraphWebWorkerError::MalformedMessage)?;
        match message {
            WbblGraphWebWorkerRequestMessage::Poll => {
                self.post_message(WbblGraphWebWorkerResponseMessage::Ready)
            }
            WbblGraphWebWorkerRequestMessage::ReceiveUpdate(update) => {
                self.doc
                    .transact_mut()
                    .apply_update(Update::decode_v2(&update).unwrap());
                Ok(())
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn register_canvas(
        &mut self,
        node_id: &str,
        offscreen_canvas: OffscreenCanvas,
    ) -> Result<(), WbblGraphWebWorkerError> {
        let id = uuid::Uuid::from_str(node_id).map_err(|_| WbblGraphWebWorkerError::MalformedId)?;
        let resources = PreviewRendererResources::new_from_offscreen_canvas(
            self.shared_preview_resources.clone(),
            BuiltInGeometry::UVSphere,
            offscreen_canvas,
            make_vertex_shader_module(),
            make_fragment_shader_module(),
        )
        .map_err(|_| WbblGraphWebWorkerError::WebGpuError)?;
        self.preview_resources
            .insert(id.as_u128(), RefCell::new(resources).into());
        if self.preview_resources.len() == 1 {
            self.animation_frame_handler.as_ref().borrow_mut().start();
        }
        Ok(())
    }

    pub fn deregister_canvas(&mut self, node_id: &str) -> Result<(), WbblGraphWebWorkerError> {
        let id = uuid::Uuid::from_str(node_id).map_err(|_| WbblGraphWebWorkerError::MalformedId)?;
        self.preview_resources.remove(&id.as_u128());
        if self.preview_resources.len() == 0 {
            self.animation_frame_handler.as_ref().borrow_mut().cancel();
        }
        Ok(())
    }
}

impl AnimationFrameProcessor for WbblGraphWebWorkerMain {
    fn process_frame(&mut self) -> bool {
        for resource in self.preview_resources.values_mut() {
            resource
                .as_ref()
                .borrow_mut()
                .render(self.shared_preview_resources.clone());
        }
        self.preview_resources.len() > 0
    }
}
