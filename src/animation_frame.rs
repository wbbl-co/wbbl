use std::cell::{Cell, RefCell};
use std::rc::Rc;

use wasm_bindgen::closure::Closure;
use wasm_bindgen::prelude::*;
use web_sys::Window;

pub struct AnimationFrameHandler {
    window: Rc<web_sys::Window>,
    closure: Rc<RefCell<Closure<dyn FnMut()>>>,
    handle: Rc<Cell<Option<i32>>>,
}

pub trait AnimationFrameProcessor {
    fn process_frame(&mut self) -> bool;
}

impl AnimationFrameHandler {
    pub fn new(window: Window) -> Self {
        let handle = Rc::new(Cell::new(None));
        let window: Rc<web_sys::Window> = window.into();
        let closure = Rc::new(RefCell::new(Closure::new(move || {})));
        let result = Self {
            window,
            closure,
            handle,
        };

        result
    }

    pub fn set_processor<Processor: AnimationFrameProcessor>(
        &mut self,
        processor: Rc<RefCell<Processor>>,
    ) where
        Processor: 'static + AnimationFrameProcessor,
    {
        self.closure.replace(Closure::new({
            let handle = self.handle.clone();
            let window = self.window.clone();
            let closure = self.closure.clone();
            let processor = processor.clone();
            move || {
                handle.set(None);
                if processor.clone().as_ref().borrow_mut().process_frame() {
                    handle.set(
                        window
                            .request_animation_frame(closure.borrow_mut().as_ref().unchecked_ref())
                            .unwrap()
                            .into(),
                    );
                }
            }
        }));
    }

    pub fn start(&mut self) {
        if let Some(handle) = self.handle.take() {
            self.window
                .cancel_animation_frame(handle)
                .expect("Failed to cancel animation frame");
        }

        let handle = self
            .window
            .request_animation_frame(self.closure.borrow_mut().as_ref().unchecked_ref())
            .expect("Failed to request animation frame");
        self.handle.set(Some(handle));
    }

    pub fn cancel(&mut self) {
        if let Some(handle) = self.handle.take() {
            self.window
                .cancel_animation_frame(handle)
                .expect("Failed to cancel animation frame");
        }
    }
}

impl Drop for AnimationFrameHandler {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            self.window
                .cancel_animation_frame(handle)
                .expect("Failed to cancel animation frame");
        }
    }
}
