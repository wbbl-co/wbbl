use std::{cell::RefCell, error::Error, fmt::Display, rc::Rc};

use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::{window, WebSocket};
use yrs::{
    encoding::read::Cursor,
    sync::{Awareness, Message, SyncMessage},
    updates::{
        decoder::{Decode, DecoderV1},
        encoder::Encode,
    },
    ReadTxn, Transact, Update,
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

type OnMessage = Closure<dyn FnMut(web_sys::MessageEvent)>;
type OnClose = Closure<dyn FnMut(web_sys::CloseEvent)>;
type OnOpen = Closure<dyn FnMut()>;
type OnError = Closure<dyn FnMut()>;

#[allow(unused)]
pub struct AwarenessWebsocketSync {
    websocket: Rc<RefCell<WebSocket>>,
    keep_alive: Rc<RefCell<Closure<dyn FnMut()>>>,
    on_open: Rc<RefCell<OnOpen>>,
    on_error: Rc<RefCell<OnError>>,
    on_close: Rc<RefCell<OnClose>>,
    on_message: Rc<RefCell<OnMessage>>,
    awareness: Rc<RefCell<Awareness>>,
    keep_alive_handle: i32,
    subscriptions: Vec<yrs::Subscription>,
}

fn create_websocket_connection(
    relative_path: &str,
    client_id: u64,
) -> Result<WebSocket, WebSocketError> {
    if let Some((Ok(protocol), Ok(hostname))) =
        window().map(|x| (x.location().protocol(), x.location().host()))
    {
        let socket = WebSocket::new(&format!(
            "{}//{}{}/{}",
            protocol, hostname, relative_path, client_id
        ));
        if let Ok(socket) = &socket {
            socket.set_binary_type(web_sys::BinaryType::Arraybuffer);
        }

        return socket.map_err(WebSocketError::JsError);
    }

    Err(WebSocketError::MissingHostname)
}

fn install_listeners(
    ws: &WebSocket,
    on_open: &Rc<RefCell<OnOpen>>,
    on_error: &Rc<RefCell<OnError>>,
    on_close: &Rc<RefCell<OnClose>>,
    on_message: &Rc<RefCell<OnMessage>>,
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
        let websocket = Rc::new(RefCell::new(create_websocket_connection(
            connect_path,
            awareness.borrow().client_id(),
        )?));

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
                120_000,
            )
            .map_err(WebSocketError::JsError)?;

        let on_message_processor = {
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
                                yrs::sync::Message::Sync(SyncMessage::SyncStep1(sv)) => {
                                    // Reply with sync step 2
                                    let update = {
                                        awareness
                                            .borrow()
                                            .doc()
                                            .transact()
                                            .encode_state_as_update_v1(&sv)
                                    };
                                    if websocket
                                        .borrow()
                                        .send_with_u8_array(
                                            Message::Sync(SyncMessage::SyncStep2(update))
                                                .encode_v1()
                                                .as_slice(),
                                        )
                                        .is_err()
                                    {
                                        let _ = websocket
                                            .borrow()
                                            .close_with_code_and_reason(1001, "FAILED TO SEND");
                                        break;
                                    }
                                }
                                yrs::sync::Message::Sync(SyncMessage::SyncStep2(update)) => {
                                    let awareness = awareness.borrow_mut();
                                    let mut txn = awareness.doc().transact_mut();
                                    if let Ok(update) = Update::decode_v1(&update) {
                                        txn.apply_update(update);
                                    } else {
                                        let _ = websocket.borrow().close_with_code_and_reason(
                                            1001,
                                            "FAILED TO APPLY UPDATE",
                                        );
                                        break;
                                    }
                                }
                                yrs::sync::Message::Sync(SyncMessage::Update(update)) => {
                                    let awareness = awareness.borrow_mut();
                                    let mut txn = awareness.doc().transact_mut();
                                    if let Ok(update) = Update::decode_v1(&update) {
                                        txn.apply_update(update);
                                    } else {
                                        let _ = websocket.borrow().close_with_code_and_reason(
                                            1001,
                                            "FAILED TO APPLY UPDATE",
                                        );
                                        break;
                                    }
                                }
                                yrs::sync::Message::Auth(_) => {
                                    let _ = websocket
                                        .borrow()
                                        .close_with_code_and_reason(1001, "UNEXPECTED MESSAGE");
                                }
                                yrs::sync::Message::AwarenessQuery => {
                                    let awareness = awareness.borrow_mut();
                                    let websocket = websocket.borrow();
                                    let message = Message::Awareness(awareness.update().unwrap());
                                    if websocket
                                        .send_with_u8_array(message.encode_v1().as_slice())
                                        .is_err()
                                    {
                                        let _ = websocket
                                            .close_with_code_and_reason(1001, "FAILED TO SEND");
                                    }
                                }
                                yrs::sync::Message::Awareness(awareness_update) => {
                                    let mut awareness = awareness.borrow_mut();
                                    if awareness.apply_update(awareness_update).is_err() {
                                        let _ = websocket
                                            .borrow()
                                            .close_with_code_and_reason(1001, "MALFORMED PAYLOAD");
                                        break;
                                    }
                                }
                                yrs::sync::Message::Custom(10, _) => {
                                    // TODO: Handle done message, need to transition into delta state
                                }
                                yrs::sync::Message::Custom(_, _) => {}
                            },
                            Err(_) => {
                                let _ = websocket
                                    .borrow()
                                    .close_with_code_and_reason(1001, "MALFORMED PAYLOAD");
                                break;
                            }
                        }
                    }
                }
            }) as Box<dyn FnMut(web_sys::MessageEvent)>)
        };

        let on_message = Rc::new(RefCell::new({
            Closure::wrap(Box::new(move |message: web_sys::MessageEvent| {
                let _ = window()
                    .expect("EXPECTED WINDOW")
                    .set_timeout_with_callback_and_timeout_and_arguments_1(
                        on_message_processor.as_ref().unchecked_ref(),
                        100,
                        &message,
                    );
            }) as Box<dyn FnMut(web_sys::MessageEvent)>)
        }));

        let on_open = Rc::new(RefCell::new({
            let websocket = websocket.clone();
            let awareness = awareness.clone();
            Closure::wrap(Box::new(move || {
                let ws = websocket.borrow();
                let awareness = awareness.borrow();
                let state_vector = awareness.doc().transact().state_vector();
                if ws
                    .send_with_u8_array(
                        Message::Sync(SyncMessage::SyncStep1(state_vector))
                            .encode_v1()
                            .as_slice(),
                    )
                    .is_err()
                {
                    let _ = websocket
                        .borrow()
                        .close_with_code_and_reason(1001, "FAILED TO SEND");
                    return;
                }

                let awareness_message = Message::Awareness(awareness.update().unwrap());
                if ws
                    .send_with_u8_array(awareness_message.encode_v1().as_slice())
                    .is_err()
                {
                    let _ = ws.close_with_code_and_reason(1001, "FAILED TO SEND");
                    return;
                }

                let awareness_query_message = Message::AwarenessQuery;
                if ws
                    .send_with_u8_array(awareness_query_message.encode_v1().as_slice())
                    .is_err()
                {
                    let _ = ws.close_with_code_and_reason(1001, "FAILED TO SEND");
                }
            }) as Box<dyn FnMut()>)
        }));

        let on_error = Rc::new(RefCell::new({
            let websocket = websocket.clone();
            Closure::wrap(Box::new(move || {
                let _ = websocket
                    .borrow()
                    .close_with_code_and_reason(1001, "WEBSOCKET ERROR");
            }) as Box<dyn FnMut()>)
        }));

        let on_close = Rc::new(RefCell::new(Closure::wrap(
            Box::new(|_: web_sys::CloseEvent| {}) as Box<dyn FnMut(web_sys::CloseEvent)>,
        )));

        on_close.replace({
            let on_open = on_open.clone();
            let on_close = on_close.clone();
            let on_error = on_error.clone();
            let on_message = on_message.clone();

            let client_id = awareness.borrow().client_id();
            let websocket = websocket.clone();
            let connect_path = connect_path.to_string();
            Closure::wrap(Box::new(move |message: web_sys::CloseEvent| {
                // Reopen websocket connection if not cleanly closed
                if message.code() == 1001 {
                    // TODO: Add some sort of backoff mechanism to prevent
                    // outages causing bad things to happen
                    if let Ok(new_websocket) = create_websocket_connection(&connect_path, client_id)
                    {
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

        let doc_subscription = {
            awareness
                .borrow()
                .doc()
                .observe_update_v1({
                    let websocket = websocket.clone();
                    move |_, update| {
                        let ws = websocket.borrow();
                        if ws.ready_state() == 1
                            && ws
                                .send_with_u8_array(
                                    yrs::sync::Message::Sync(SyncMessage::Update(
                                        update.update.to_vec(),
                                    ))
                                    .encode_v1()
                                    .as_slice(),
                                )
                                .is_err()
                        {
                            let _ = ws.close_with_code_and_reason(1001, "FAILED TO SEND");
                        }
                    }
                })
                .expect("Successful Subscription")
        };

        let awareness_subscription = {
            awareness.borrow().on_update({
                let websocket = websocket.clone();
                move |evt| {
                    let ws = websocket.borrow();
                    if ws.ready_state() == 1 {
                        if let Some(update) = evt.awareness_update() {
                            let message = Message::Awareness(update.clone());
                            if ws
                                .send_with_u8_array(message.encode_v1().as_slice())
                                .is_err()
                            {
                                let _ = ws.close_with_code_and_reason(1001, "FAILED TO SEND");
                            }
                        }
                    }
                }
            })
        };

        Ok(AwarenessWebsocketSync {
            websocket,
            awareness: awareness.clone(),
            keep_alive: keep_alive.clone(),
            on_close: on_close.clone(),
            on_message: on_message.clone(),
            on_open: on_open.clone(),
            keep_alive_handle,
            on_error: on_error.clone(),
            subscriptions: vec![doc_subscription, awareness_subscription],
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
            let _ = ws.close_with_code(1000);
        }
    }
}
