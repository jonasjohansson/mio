//! WebSocket server bridge using tokio-tungstenite.
//!
//! Runs as an async task on the tokio runtime. Broadcasts messages
//! from the serial protocol to all connected WS clients.
//! Also forwards incoming WS messages back to the app event loop.

use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tokio_tungstenite::tungstenite::Message;

/// Shared client count, readable from the TUI.
pub type ClientCount = Arc<AtomicUsize>;

/// Start the WebSocket server as a tokio task.
/// Returns the client count and the join handle.
pub async fn start_server(
    host: &str,
    port: u16,
    broadcast_rx: broadcast::Sender<String>,
    incoming_tx: tokio::sync::mpsc::Sender<String>,
) -> Result<(ClientCount, tokio::task::JoinHandle<()>)> {
    let addr: SocketAddr = format!("{}:{}", host, port).parse()?;
    let listener = TcpListener::bind(&addr).await?;
    let client_count = Arc::new(AtomicUsize::new(0));
    let count_clone = client_count.clone();

    eprintln!("[mio] WebSocket server listening on {}", addr);

    let handle = tokio::spawn(async move {
        loop {
            match listener.accept().await {
                Ok((stream, peer)) => {
                    let count = count_clone.clone();
                    let mut rx = broadcast_rx.subscribe();
                    let tx = incoming_tx.clone();

                    tokio::spawn(async move {
                        count.fetch_add(1, Ordering::Relaxed);

                        let ws_stream = match tokio_tungstenite::accept_async(stream).await {
                            Ok(ws) => ws,
                            Err(e) => {
                                eprintln!("[mio] WS handshake failed for {}: {}", peer, e);
                                count.fetch_sub(1, Ordering::Relaxed);
                                return;
                            }
                        };

                        let (mut ws_sink, mut ws_source) = ws_stream.split();

                        // Task: forward broadcast messages to this client
                        let sink_task = tokio::spawn(async move {
                            while let Ok(msg) = rx.recv().await {
                                if ws_sink.send(Message::text(msg)).await.is_err() {
                                    break;
                                }
                            }
                        });

                        // Read incoming messages from the client
                        while let Some(msg) = ws_source.next().await {
                            match msg {
                                Ok(Message::Text(text)) => {
                                    let _ = tx.send(text.to_string()).await;
                                }
                                Ok(Message::Close(_)) => break,
                                Err(_) => break,
                                _ => {}
                            }
                        }

                        sink_task.abort();
                        count.fetch_sub(1, Ordering::Relaxed);
                    });
                }
                Err(e) => {
                    eprintln!("[mio] WS accept error: {}", e);
                }
            }
        }
    });

    Ok((client_count, handle))
}
