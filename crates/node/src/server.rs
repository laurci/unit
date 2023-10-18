use std::path::PathBuf;

use axum::{
    extract::{
        ws::{Message, WebSocket},
        Query, State, WebSocketUpgrade,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use log::{error, info};
use serde::Deserialize;
use tokio::task::JoinHandle;
use unit_index::Index;
use unit_runtime_proto::{CrossbarContent, CrossbarMessage, WsMessage};
use unit_utils::{err::bail, gen_uuid, Result};

use crate::{
    bus::{Bus, BusMessage},
    config::CONFIG,
    runtime::{Runtime, RuntimeEnv},
};

#[derive(Clone)]
pub struct WsState {
    bus: Bus,
}

impl WsState {
    pub fn new(bus: Bus) -> Self {
        Self { bus }
    }
}

#[derive(Deserialize)]
struct WsUpgradeQueryParams {
    app: String,
}

pub async fn serve_ws(addr: String, bus: Bus) -> Result<()> {
    let addr = addr.parse()?;
    let state = WsState::new(bus);

    let app = Router::new()
        .route("/ws", get(ws_upgrade_handler))
        .with_state(state);

    info!("ws server listening on ws://{}", &addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

async fn ws_upgrade_handler(
    ws: WebSocketUpgrade,
    query: Query<WsUpgradeQueryParams>,
    State(state): State<WsState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| async move {
        let runtime_result = handle_socket(socket, query.app.clone(), state.bus).await;

        if runtime_result.is_err() {
            error!("runtime error {:?}", runtime_result);
        }

        ()
    })
}

fn socket_rx_task(
    connection_id: String,
    mut socket_rx: SplitStream<WebSocket>,
    bus: Bus,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        while let Some(msg) = socket_rx.next().await {
            if let Ok(msg) = msg {
                bus.send(BusMessage::RxWsMessage {
                    connection_id: connection_id.clone(),
                    message: msg,
                });
            } else {
                break;
            }
        }

        bus.send(BusMessage::Kill {
            connection_id: connection_id.clone(),
        });
    })
}

fn socket_tx_task(
    root_connection_id: String,
    mut socket_tx: SplitSink<WebSocket, Message>,
    bus: Bus,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        while let Ok(msg) = bus.subscribe().recv().await {
            match msg {
                BusMessage::TxWsMessage {
                    connection_id,
                    message,
                } => {
                    if connection_id != root_connection_id {
                        continue;
                    }

                    let _ = socket_tx.send(message).await;
                }
                BusMessage::Kill { connection_id } => {
                    if connection_id != connection_id {
                        continue;
                    }

                    break;
                }
                _ => {}
            };
        }
    })
}

async fn runtime_task(root_connection_id: String, bus: Bus, app_name: String) -> Result<()> {
    let index = Index::load(CONFIG.storage_path.clone())?;
    let Some(index_entry) = index
        .entries()
        .iter()
        .find(|e| e.abi_header.name == app_name)
    else {
        bail!("app not found");
    };
    let app_path: PathBuf = CONFIG.storage_path.parse()?;
    let app_path = app_path.join(index_entry.path.clone());

    let runtime_env = RuntimeEnv::new(root_connection_id.clone(), bus.clone());
    let mut runtime = Runtime::new(
        index_entry.abi_header.name.clone(),
        app_path.to_str().unwrap().to_owned(),
        index_entry.abi_header.clone(),
        runtime_env,
    )?;

    runtime.start()?;

    bus.clone().send(BusMessage::Ready {
        connection_id: root_connection_id.clone(),
    });

    while let Ok(msg) = bus.subscribe().recv().await {
        match msg {
            BusMessage::Kill { connection_id } => {
                if connection_id != root_connection_id {
                    continue;
                }

                break;
            }
            BusMessage::RxWsMessage {
                connection_id,
                message,
            } => {
                if connection_id != root_connection_id {
                    continue;
                }

                let message = match message {
                    Message::Text(text) => WsMessage::Text(text),
                    Message::Binary(bin) => WsMessage::Binary(bin),
                    _ => continue,
                };

                runtime.message(message)?;
            }
            BusMessage::CrossbarMessage(msg) => {
                let content = match msg.content {
                    unit_crossbar::CrossbarContent::Text(text) => CrossbarContent::Text(text),
                    unit_crossbar::CrossbarContent::Binary(bin) => CrossbarContent::Binary(bin),
                };

                runtime.crossbar_event(CrossbarMessage {
                    topic: msg.topic,
                    content,
                })?;
            }
            _ => {}
        };
    }

    info!("[{}] stop runtime", root_connection_id);

    runtime.stop()?;

    Ok(())
}

#[allow(unused)]
fn test_delayed_ready_task(connection_id: String, bus: Bus, ms: u64) {
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(ms)).await;

        bus.send(BusMessage::Ready {
            connection_id: connection_id.clone(),
        });
    });
}

#[allow(unused)]
fn test_loop_ws_tx_task(connection_id: String, bus: Bus, ms: u64) {
    tokio::spawn(async move {
        let mut i = 0;
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(ms)).await;

            bus.send(BusMessage::TxWsMessage {
                connection_id: connection_id.clone(),
                message: Message::Text(format!("hello {}", i)),
            });

            i += 1;
        }
    });
}

async fn handle_socket(socket: WebSocket, app_name: String, bus: Bus) -> Result<()> {
    let connection_id = gen_uuid();
    let (socket_tx, socket_rx) = socket.split();

    let socket_rx_handle = socket_rx_task(connection_id.clone(), socket_rx, bus.clone());
    let socket_tx_handle = socket_tx_task(connection_id.clone(), socket_tx, bus.clone());

    // test_delayed_ready_task(connection_id.clone(), bus.clone(), 1_000);
    // test_loop_ws_tx_task(connection_id.clone(), bus.clone(), 3_000);

    let runtime_result = runtime_task(connection_id.clone(), bus.clone(), app_name).await;
    if runtime_result.is_err() {
        socket_rx_handle.abort();
        socket_tx_handle.abort();

        bus.send(BusMessage::Kill {
            connection_id: connection_id.clone(),
        });

        return Err(runtime_result.unwrap_err());
    }

    Ok(())
}
