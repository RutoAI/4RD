use std::net::SocketAddr;

use async_tungstenite::tokio::accept_async;
use async_tungstenite::tokio::connect_async;
use futures::{future, pin_mut, StreamExt, SinkExt};
use tokio::net::{TcpListener, TcpStream};
use url::Url;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let listen_addr = "127.0.0.1:6940";
    let listener = TcpListener::bind(listen_addr).await?;
    log::info!("Listening on ws://{}", listen_addr);

    loop {
        let (stream, addr) = listener.accept().await?;
        // spawn a task and log errors from the handler
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, addr).await {
                log::error!("connection error ({}): {}", addr, e);
            }
        });
    }
}

async fn handle_connection(stream: TcpStream, _peer: SocketAddr) -> anyhow::Result<()> {
    let ws_stream = accept_async(stream).await?;
    log::info!("Accepted client connection from {}", _peer);

    // Connect to Binance fstream websocket
    let target = Url::parse("wss://fstream.binance.com/ws")?;
    let (upstream_ws, _resp) = connect_async(target).await?;
    log::info!("Connected to upstream Binance");

    // Split streams
    let (mut client_write, mut client_read) = ws_stream.split();
    let (mut upstream_write, mut upstream_read) = upstream_ws.split();

    // Forward client -> upstream
    let c2u = async {
        while let Some(msg) = client_read.next().await {
            let msg = msg?;
            if msg.is_close() {
                upstream_write.send(msg).await?;
                break;
            }
            upstream_write.send(msg).await?;
        }
        Ok::<(), anyhow::Error>(())
    };

    // Forward upstream -> client
    let u2c = async {
        while let Some(msg) = upstream_read.next().await {
            let msg = msg?;
            if msg.is_close() {
                client_write.send(msg).await?;
                break;
            }
            client_write.send(msg).await?;
        }
        Ok::<(), anyhow::Error>(())
    };

    pin_mut!(c2u, u2c);
    future::select(c2u, u2c).await;

    Ok(())
}
