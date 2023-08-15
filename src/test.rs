use std::{
  io::Error as IoError,
  net::SocketAddr,
};

use async_stream::stream;
use futures_util::pin_mut;
use itertools::Itertools;
use rust_decimal_macros::dec;
use rust_decimal::Decimal;
use futures_util::{StreamExt, SinkExt};
use crate::binance::OrderBook;

use tokio::net::{TcpListener, TcpStream};

async fn handle_connection(raw_stream: TcpStream, addr: SocketAddr) {
  println!("Incoming TCP connection from: {}", addr);

  let ws_stream = tokio_tungstenite::accept_async(raw_stream)
      .await
      .expect("Error during the websocket handshake occurred");
  println!("WebSocket connection established: {}", addr);

  let (mut outgoing, _incoming) = ws_stream.split();
  let out_stream = stream! {
    for j in (0..100).cycle() {
      let msg = serde_json::to_string(&OrderBook { 
        lastUpdateId: dec!(1), 
            asks: (1..16).map(|i| (Decimal::from_str_exact(&(100.0 + i as f64).to_string()).unwrap(), Decimal::from(j*j))).collect_vec(),
            bids: (1..16).map(|i| (Decimal::from_str_exact(&(100.0 - i as f64).to_string()).unwrap(), Decimal::from(2 * j*j))).collect_vec(),
           }).unwrap();
        yield msg;
      }
    };
   pin_mut!(out_stream);
   while let Some(v) = out_stream.next().await {
      let _ = outgoing.send(v.into()).await;
   }
}

// addr i.e 127.0.0.1:8080
pub async fn server(addr: String) -> Result<(), IoError> {
  let try_socket = TcpListener::bind(&addr).await;
  let listener = try_socket.expect("Failed to bind");
  println!("Listening on: {}", addr);
  while let Ok((stream, addr)) = listener.accept().await {
    tokio::spawn(handle_connection(stream, addr));
  }
  Ok(())
}