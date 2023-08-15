use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message, tungstenite::error::Error};
use crate::exchange_tools::{Exchange, parse_book, OrderBook};
use futures_util::{StreamExt, SinkExt};

use url::Url;

pub async fn connect_exchange(
  exchange: Exchange, 
  subscriber : Option<String>, 
  tx: mpsc::Sender<OrderBook>) -> Result<(), Error> {
  let exchange_stream = exchange.clone().value();
  let url = Url::parse(&exchange_stream).expect("bad url string");
  println!("connecting to {:?}", url);
  let (ws_stream, _) = connect_async(url).await?;
  println!("connected {:?}", exchange);
  let (mut out_stream, input_stream) = ws_stream.split();
  if let Some(message) = subscriber {
    println!("subscribing...");
    match out_stream.send(Message::Text(message))
    .await {
      Ok(_) => println!("ok"),
      Err(e) => println!("{:?}", e)
    };
  }  
  let read_future = input_stream.for_each(|message| async {
    //println!("{:?}", message);
    if let Ok(order_book) = parse_book(exchange.clone(), message.unwrap()) {
      let _ = tx.send(order_book).await;
    }
  });
  read_future.await;
  Ok::<(), Error>(())
}