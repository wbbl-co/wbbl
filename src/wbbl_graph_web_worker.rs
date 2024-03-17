use crate::preview_renderer::render_preview;
use std::panic;
use wasm_bindgen::prelude::*;
use web_sys::OffscreenCanvas;

#[wasm_bindgen]
pub struct WbblGraphWebWorkerMain {}

#[wasm_bindgen]
impl WbblGraphWebWorkerMain {
    pub fn new() -> WbblGraphWebWorkerMain {
        panic::set_hook(Box::new(console_error_panic_hook::hook));
        WbblGraphWebWorkerMain {}
    }

    pub async fn render(&self, offscreen_canvas: OffscreenCanvas) {
        let instance = wgpu::Instance::default();
        render_preview(offscreen_canvas, instance).await;
    }
}
