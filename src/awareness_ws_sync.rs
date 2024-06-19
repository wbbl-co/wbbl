use std::{array, cell::RefCell, error::Error, fmt::Display, rc::Rc};

use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::{js_sys::Function, window, WebSocket};
use yrs::{
    encoding::read::Cursor,
    sync::{awareness, Awareness, SyncMessage},
    updates::decoder::DecoderV1,
};

#[derive(Debug)]
pub enum WebSocketError {
    JsError(JsValue),
    MissingHostname,
}

impl Display for WebSocketError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WebSocketError::JsError(err) => f.write_fmt(format_args!("JsErr {:?}", err)),
            WebSocketError::MissingHostname => f.write_str("Missing Hostname"),
        }
    }
}

impl Error for WebSocketError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }

    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }
}

#[allow(unused)]
pub struct AwarenessWebsocketSync {
    websocket: Rc<RefCell<WebSocket>>,
    keep_alive: Rc<RefCell<Closure<dyn FnMut()>>>,
    on_open: Rc<RefCell<Closure<dyn FnMut()>>>,
    on_error: Rc<RefCell<Closure<dyn FnMut()>>>,
    on_close: Rc<RefCell<Closure<dyn FnMut(web_sys::CloseEvent)>>>,
    on_message: Rc<RefCell<Closure<dyn FnMut(web_sys::MessageEvent)>>>,
    awareness: Rc<RefCell<Awareness>>,
    keep_alive_handle: i32,
}

fn create_websocket_connection(relative_path: &str) -> Result<WebSocket, WebSocketError> {
    if let Some((Ok(protocol), Ok(hostname))) =
        window().map(|x| (x.location().protocol(), x.location().hostname()))
    {
        let socket = WebSocket::new(&format!("{}://{}{}", protocol, hostname, relative_path));
        if let Ok(socket) = socket {
            socket.set_binary_type(web_sys::BinaryType::Arraybuffer);
        }
        return socket.map_err(|err| WebSocketError::JsError(err));
    }

    Err(WebSocketError::MissingHostname)
}

fn install_listeners(
    ws: &WebSocket,
    on_open: &Rc<RefCell<Closure<dyn FnMut()>>>,
    on_error: &Rc<RefCell<Closure<dyn FnMut()>>>,
    on_close: &Rc<RefCell<Closure<dyn FnMut(web_sys::CloseEvent)>>>,
    on_message: &Rc<RefCell<Closure<dyn FnMut(web_sys::MessageEvent)>>>,
) {
    ws.set_onopen(Some(on_open.borrow().as_ref().unchecked_ref()));
    ws.set_onclose(Some(on_close.borrow().as_ref().unchecked_ref()));
    ws.set_onmessage(Some(on_message.borrow().as_ref().unchecked_ref()));
    ws.set_onerror(Some(on_error.borrow().as_ref().unchecked_ref()));
}

impl AwarenessWebsocketSync {
    pub fn try_create(
        awareness: Rc<RefCell<Awareness>>,
        connect_path: &str,
    ) -> Result<AwarenessWebsocketSync, WebSocketError> {
        let websocket = Rc::new(RefCell::new(create_websocket_connection(connect_path)?));

        let keep_alive = Rc::new(RefCell::new({
            let websocket = websocket.clone();
            Closure::new(move || {
                let ws = websocket.borrow();
                if ws.ready_state() == 1 {
                    let _ = ws.send_with_str("PING");
                }
            })
        }));

        let keep_alive_handle = window()
            .expect("Expected Window")
            .set_interval_with_callback_and_timeout_and_arguments_0(
                keep_alive.borrow().as_ref().unchecked_ref(),
                1200,
            )
            .map_err(|err| WebSocketError::JsError(err))?;

        let on_message = Rc::new(RefCell::new({
            let websocket = websocket.clone();
            let awareness = awareness.clone();
            Closure::wrap(Box::new(move |message: web_sys::MessageEvent| {
                if message
                    .data()
                    .dyn_into::<web_sys::js_sys::JsString>()
                    .is_ok()
                {
                    // ignore message. Probably just PONG
                } else if let Ok(array_buffer) =
                    message.data().dyn_into::<web_sys::js_sys::ArrayBuffer>()
                {
                    let bin = web_sys::js_sys::Uint8Array::new(&array_buffer).to_vec();
                    let cursor: Cursor = Cursor::new(&bin);
                    let mut decoder = DecoderV1::new(cursor);
                    let reader = yrs::sync::protocol::MessageReader::new(&mut decoder);

                    for message in reader {
                        match message {
                            Ok(message) => match message {
                                yrs::sync::Message::Sync(SyncMessage::SyncStep1(_)) => todo!(),
                                yrs::sync::Message::Sync(SyncMessage::SyncStep2(_)) => todo!(),
                                yrs::sync::Message::Sync(SyncMessage::Update(_)) => todo!(),
                                yrs::sync::Message::Auth(_) => {}
                                yrs::sync::Message::AwarenessQuery => {}
                                yrs::sync::Message::Awareness(awareness_update) => todo!(),
                                yrs::sync::Message::Custom(_, _) => {}
                            },
                            Err(_) => {
                                websocket
                                    .borrow()
                                    .close_with_code_and_reason(1001, "MALFORMED PAYLOAD");
                                break;
                            }
                        }
                    }
                }
            }) as Box<dyn FnMut(web_sys::MessageEvent)>)
        }));

        let on_open = Rc::new(RefCell::new({
            let websocket = websocket.clone();
            Closure::wrap(Box::new(move || {}) as Box<dyn FnMut()>)
        }));

        let on_error = Rc::new(RefCell::new({
            let websocket = websocket.clone();
            Closure::wrap(Box::new(move || {}) as Box<dyn FnMut()>)
        }));

        let on_close = Rc::new(RefCell::new(Closure::wrap(
            Box::new(|_: web_sys::CloseEvent| {}) as Box<dyn FnMut(web_sys::CloseEvent)>,
        )));

        on_close.replace({
            let on_open = on_open.clone();
            let on_close = on_close.clone();
            let on_error = on_error.clone();
            let on_message = on_message.clone();

            let websocket = websocket.clone();
            let connect_path = connect_path.to_string();
            Closure::wrap(Box::new(move |message: web_sys::CloseEvent| {
                // Reopen websocket connection if not cleanly closed
                if message.code() != 1000 {
                    // TODO: Add some sort of backoff mechanism to prevent
                    // outages causing bad things to happen
                    if let Ok(new_websocket) = create_websocket_connection(&connect_path) {
                        websocket.replace(new_websocket);
                        install_listeners(
                            &websocket.borrow(),
                            &on_open,
                            &on_error,
                            &on_close,
                            &on_message,
                        );
                    }
                }
            }) as Box<dyn FnMut(web_sys::CloseEvent)>)
        });

        install_listeners(
            &websocket.borrow(),
            &on_open,
            &on_error,
            &on_close,
            &on_message,
        );

        Ok(AwarenessWebsocketSync {
            websocket,
            awareness: awareness.clone(),
            keep_alive: keep_alive.clone(),
            on_close: on_close.clone(),
            on_message: on_message.clone(),
            on_open: on_open.clone(),
            keep_alive_handle,
            on_error: on_error.clone(),
        })
    }
}

impl Drop for AwarenessWebsocketSync {
    fn drop(&mut self) {
        let keep_alive_handle = self.keep_alive_handle;
        window().inspect(move |window| {
            window.clear_interval_with_handle(keep_alive_handle);
        });
        let ws = self.websocket.borrow();
        if ws.ready_state() == 1 {
            let _ = self.websocket.borrow().close_with_code(1000);
        }
    }
}
