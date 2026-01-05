use futures_util::{SinkExt, StreamExt};
use multisnake_shared::SnakeMessage;
use std::thread;
use tokio_tungstenite::{connect_async, tungstenite::Message};

pub fn spawn_network_thread(
    url: String,
    mut from_client_rx: tokio::sync::mpsc::UnboundedReceiver<SnakeMessage>,
    from_server_tx: std::sync::mpsc::Sender<SnakeMessage>,
) {
    thread::spawn(move || {
        let tokio_runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        tokio_runtime.block_on(async move {
            let (ws_stream, _) = match connect_async(&url).await {
                Ok(ws) => ws,
                Err(_) => return,
            };

            let (mut ws_tx, mut ws_rx) = ws_stream.split();

            loop {
                tokio::select! {
                    client_msg = from_client_rx.recv() => {
                        match client_msg {
                            Some(msg) => {
                                let json = serde_json::to_string(&msg).unwrap();
                                if ws_tx.send(Message::Text(json.into())).await.is_err() {
                                    break;
                                }
                            }
                            None => {
                                // TODO: minor check
                                println!("Main thread dropped sender. Closing network...");
                                let _ = ws_tx.close().await;
                                break;
                            }
                        }
                    }
                    server_msg = ws_rx.next() => {
                        match server_msg {
                            Some(Ok(Message::Text(txt))) => {
                                if let Ok(parsed) = serde_json::from_str::<SnakeMessage>(&txt) {
                                    if from_server_tx.send(parsed).is_err() {
                                        break;
                                    }
                                }
                            }
                            // TODO: minor check
                            Some(Ok(Message::Close(_))) | None => break,
                            _ => {}
                        }
                    }
                }
            }
        });
    });
}
