use futures_util::{SinkExt, StreamExt};
use multisnake_shared::SnakeMessage;
use std::thread;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};

pub fn spawn_network_thread(
    url: String,
    mut from_client_rx: mpsc::UnboundedReceiver<SnakeMessage>,
    from_server_tx: std::sync::mpsc::Sender<SnakeMessage>,
) {
    thread::spawn(move || {
        let tokio_runtime = Runtime::new().unwrap();

        tokio_runtime.block_on(async move {
            let (ws_stream, _) = connect_async(&url).await.expect("Failed to connect");
            println!("Connected to WebSocket server at {}", url);
            let (mut ws_tx, mut ws_rx) = ws_stream.split();

            tokio::spawn(async move {
                while let Some(message) = from_client_rx.recv().await {
                    // Send messages from client to server
                    let json = serde_json::to_string(&message).unwrap();
                    ws_tx.send(Message::Text(json.into())).await.unwrap();
                }
            });

            loop {
                if let Some(msg) = ws_rx.next().await {
                    match msg {
                        Ok(Message::Text(txt)) => {
                            if let Ok(server_msg) = serde_json::from_str::<SnakeMessage>(&txt) {
                                from_server_tx.send(server_msg).unwrap();
                            }
                        }
                        Ok(_) => {}
                        Err(e) => {
                            println!("WebSocket error: {}", e);
                            break;
                        }
                    }
                }
            }
        });
    });
}
