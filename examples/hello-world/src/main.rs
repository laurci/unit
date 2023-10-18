use std::sync::Mutex;

use unit::{log, Message};

unit::application! {
    name = "hello-world",
}

unit::data! { event_count: Mutex<u32> = Mutex::new(0) }

#[unit::init]
async fn init() {
    let count = 12;
    log!("hello world! count = {c}", c = count);
}

#[unit::cleanup]
async fn cleanup() {
    log!("bye bye!");
}

#[unit::message]
async fn message(message: &Message) {
    log!("got message {:?}", message);
    unit::client::send_text(format!("echo {:?}", message));
}

#[unit::topic(name = "test")]
async fn test(message: CrossbarContent) {
    log!("topic callback {} {:?}", event.topic, message);
}
