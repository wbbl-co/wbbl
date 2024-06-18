use std::str::FromStr;

use wgpu::naga::Span;

use crate::store_errors::WbblWebappStoreError;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
#[macro_export]
macro_rules! log {
    ( $( $t:tt )* ) => {
        if cfg!(debug_assertions) {
                web_sys::console::log_1(&format!( $( $t )* ).into());
        }
    }
}

pub fn make_span(line_number: u32) -> Span {
    Span::new(line_number, line_number)
}

pub fn try_into_u128(value: &str) -> Result<u128, WbblWebappStoreError> {
    uuid::Uuid::from_str(value)
        .map_err(|_| WbblWebappStoreError::MalformedId)
        .map(|x| x.as_u128())
}
