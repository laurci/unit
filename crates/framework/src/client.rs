use unit_runtime_proto::{encode_runtime_proto_message, WsMessage};

use crate::vm_internals;

pub fn send_text(message: String) {
    let message = WsMessage::Text(message);
    let bytes = encode_runtime_proto_message(&message).unwrap();

    unsafe {
        vm_internals::unit_send_message(bytes.as_ptr() as _, bytes.len() as _);
    }
}

pub fn send_bytes(message: Vec<u8>) {
    let message = WsMessage::Binary(message);
    let bytes = encode_runtime_proto_message(&message).unwrap();

    unsafe {
        vm_internals::unit_send_message(bytes.as_ptr() as _, bytes.len() as _);
    }
}
