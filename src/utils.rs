use wgpu::naga::Span;

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
