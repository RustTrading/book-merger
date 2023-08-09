use futures_util::{StreamExt, SinkExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message, tungstenite::error::Error};
use futures::join;
use url::Url;

use book_merger::exchange_tools::{BINANCE_WSS_ETHBTC_20, BITSTAMP_WSS, Exchanges, parse_book};
use serde_json::json;

async fn connect_exchange(exchange: Exchanges, subscriber : Option<String>) -> Result<(), Error> {
  let exchange_stream = exchange.value();
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
    parse_book(exchange, message.unwrap());
  });
  read_future.await;
  Ok::<(), Error>(())
}

#[tokio::main]
async fn main() -> Result<(), Error>{
  let subscribe_ethbtc: String = json!({
    "event": "bts:subscribe",
    "data": {
      "channel": "order_book_ethbtc"
    }
  }
  ).to_string();

  let _unsubscribe_eth_btc: String = json!({
    "event": "bts:unsubscribe",
    "data": {
    "channel": "order_book_ethbtc"
  }
  }
  ).to_string();
  let _= join![
    tokio::spawn(async move { connect_exchange(Exchanges::Bitstamp(BITSTAMP_WSS), Some(subscribe_ethbtc)).await }),
    tokio::spawn(async move { connect_exchange(Exchanges::Binance(BINANCE_WSS_ETHBTC_20), None).await })
  ];
  Ok(())
}
