use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Debug)]
pub enum WbblWebappStoreError {
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
    SerializationFailure,
    CannotDeleteOutputNode,
    SubscriptionFailure,
}
