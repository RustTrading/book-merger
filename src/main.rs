use futures_util::{StreamExt, SinkExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message, tungstenite::error::Error};
use serde_json::json;
use url::Url;
const BITSTAMP_WSS: &str = "wss://ws.bitstamp.net";
const BINANCE_WSS_ETHBTC_20: &str = "wss://stream.binance.com:9443/ws/ethbtc@depth20@100ms";

use futures::join;

async fn connect_exchange(exchange_stream: &str, subscriber : Option<String>) -> Result<(), Error> {
  let url = Url::parse(exchange_stream).expect("bad url string");
  let (ws_stream, _) = connect_async(url).await?;
  println!("connected");
  let (mut out_stream, input_stream) = ws_stream.split();
  if let Some(message) = subscriber {
    println!("subscribing...");
    match out_stream.send(Message::Text(message))
    .await {
      Ok(res) => println!("{:?}", res),
      Err(e) => println!("{:?}", e)
    };
  }  
  let read_future = input_stream.for_each(|message| async {
    println!("{}, {:?}", exchange_stream, message.unwrap().to_text());
  });
  read_future.await;
  Ok::<(), Error>(())
}

#[tokio::main]
async fn main() -> Result<(), Error>{
  let subscribe_ethbtc = json!({
    "event": "bts:subscribe",
    "data": {
      "channel": "order_book_ethbtc"
    }
  }
  ).to_string();
  let _unsubscribe_ethbtc = json!({
    "event": "bts:unsubscribe",
    "data": {
      "channel": "order_book_ethbtc"
    }
  }
  ).to_string();
  let _= join![
    tokio::spawn(async move { connect_exchange(BITSTAMP_WSS, Some(subscribe_ethbtc)).await }),
    tokio::spawn(async move { connect_exchange(BINANCE_WSS_ETHBTC_20, None).await })
  ];
  Ok(())
}
