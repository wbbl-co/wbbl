use crate::{
    animation_frame::{AnimationFrameHandler, AnimationFrameProcessor},
    builtin_geometry::BuiltInGeometry,
    data_types::AbstractDataType,
    graph_functions,
    graph_transfer_types::WbblWebappGraphSnapshot,
    graph_types::{Graph, PortId},
    preview_renderer::{PreviewRendererResources, SharedPreviewRendererResources},
    test_fragment_shader::make_fragment_shader_module,
    vertex_shader::make_vertex_shader_module,
};
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, collections::HashMap, panic, rc::Rc, str::FromStr, sync::Arc};
use wasm_bindgen::prelude::*;
use web_sys::{DedicatedWorkerGlobalScope, OffscreenCanvas, Window};

pub struct WbblGraphWebWorkerMain {
    current_graph: Option<Graph>,
    preview_resources: HashMap<u128, Arc<RefCell<PreviewRendererResources>>>,
    shared_preview_resources: Arc<SharedPreviewRendererResources>,
    animation_frame_handler: Arc<RefCell<AnimationFrameHandler>>,
    worker_scope: Arc<DedicatedWorkerGlobalScope>,
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
    SetSnapshot(WbblWebappGraphSnapshot),
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
        let result = WbblGraphWebWorkerMain {
            current_graph: None,
            shared_preview_resources: shared_preview_resources.into(),
            preview_resources: HashMap::new().into(),
            animation_frame_handler,
            worker_scope: worker_scope.clone(),
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
            WbblGraphWebWorkerRequestMessage::SetSnapshot(snapshot) => {
                let graph: Graph = snapshot.into();
                if Some(&graph) != self.current_graph.as_ref() {
                    match graph_functions::narrow_abstract_types(&graph) {
                        Ok(types) => {
                            let result = self.post_message(
                                WbblGraphWebWorkerResponseMessage::TypesUpdated(types),
                            );
                            self.current_graph = Some(graph);
                            result
                        }
                        Err(_) => self.post_message(
                            WbblGraphWebWorkerResponseMessage::TypeUnificationFailure,
                        ),
                    }
                } else {
                    Ok(())
                }
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
