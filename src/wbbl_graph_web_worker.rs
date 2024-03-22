use crate::{
    graph_transfer_types::WbblWebappGraphSnapshot, graph_types::Graph,
    preview_renderer::render_preview,
};
use serde::{Deserialize, Serialize};
use std::panic;
use wasm_bindgen::prelude::*;
use web_sys::OffscreenCanvas;

#[wasm_bindgen]
pub struct WbblGraphWebWorkerMain {
    current_graph: Option<Graph>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum WbblGraphWebWorkerMessage {
    SetSnapshot(WbblWebappGraphSnapshot),
}

#[wasm_bindgen]
pub enum WbblGraphWebWorkerError {
    MalformedMessage,
}

#[wasm_bindgen]
impl WbblGraphWebWorkerMain {
    pub fn new() -> WbblGraphWebWorkerMain {
        panic::set_hook(Box::new(console_error_panic_hook::hook));
        WbblGraphWebWorkerMain {
            current_graph: None,
        }
    }

    pub fn handle_message(&mut self, value: JsValue) -> Result<(), WbblGraphWebWorkerError> {
        let message = serde_wasm_bindgen::from_value::<WbblGraphWebWorkerMessage>(value)
            .map_err(|_| WbblGraphWebWorkerError::MalformedMessage)?;
        match message {
            WbblGraphWebWorkerMessage::SetSnapshot(_) => {}
        };
        Ok(())
    }

    pub async fn render(&self, offscreen_canvas: OffscreenCanvas) {
        let instance = wgpu::Instance::default();
        render_preview(offscreen_canvas, instance).await;
    }
}
