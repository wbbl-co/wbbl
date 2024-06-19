use web_sys::{window, WebSocket};

pub fn create_websocket_connection(
    relative_path: &str,
) -> Result<WebSocket, Box<dyn std::error::Error>> {
    if let Some((Ok(protocol), Ok(hostname))) =
        window().map(|x| (x.location().protocol(), x.location().hostname()))
}
