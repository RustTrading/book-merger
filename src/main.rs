
use book_merger::exchange_tools::{BINANCE_WSS, BITSTAMP_WSS, Exchange};
use book_merger::book_streamer::BookStreamer;
use serde_json::json;
use book_merger::client::error::Error;
use clap::{Arg, App};

async fn grpc_server(exchanges: Vec<(Exchange, Option<String>)>) -> Result<(), Error> {
  let mut worker = BookStreamer::new(exchanges);
  worker.run().await
}

#[tokio::main]
async fn main() -> Result<(), Error> {
  let matches = App::new("book-merger")
    .version("0.0.1")
    .about("create combined orderbook for Binance and Bitstamp")
    .arg(Arg::new("currencies")
    .long("currencies")
    .required(false)
    .takes_value(true)
    .help("provides pair of currencies for orderbook data")
  )
  .get_matches();
  let mut currencies = "ethbtc";
   
  let frmt = json!({
    "event": "bts:subscribe",
    "data": {
      "channel": "order_book_{}"
    }
  }).to_string();
  
  if matches.is_present("currencies") {
    if let Some(val) = matches.value_of("currencies") {
      currencies = val; 
   }
  }

  let subscribe_ethbtc: String = frmt.replace("{}", currencies);
  let binance_wss_currency = BINANCE_WSS.replace("{}", currencies);
  let exchanges = vec![
    (Exchange::Binance(binance_wss_currency), None),
    (Exchange::Bitstamp(BITSTAMP_WSS.to_owned()), Some(subscribe_ethbtc))];
  grpc_server(exchanges).await
}

#[cfg(test)]
pub mod test {
  use book_merger::{test::server, exchange_tools::Exchange, client::grpc_client};
  use tokio::{select, time, time::Duration};
  use serde_json::json;
  use crate::grpc_server;
  #[tokio::test(flavor = "multi_thread")] 
    async fn mock_servers() {
      let exchanges = vec![
        (Exchange::Binance("ws://127.0.0.1:8080".to_owned()), None),
        (Exchange::Other("ws://127.0.0.1:3030".to_owned()), None)
      ];
      let sleep = time::sleep(Duration::from_millis(10000));
      tokio::pin!(sleep);
      select!{
      _  = tokio::spawn(async move {
            server("127.0.0.1:8080".to_owned()).await
         }) => {}
      _ = tokio::spawn(async move {
            server("127.0.0.1:3030".to_owned()).await
        }) => {}
        _  = tokio::spawn(async move {
          grpc_server(exchanges).await
        }) => { }
        _  = tokio::spawn(async move {
          grpc_client().await
        }) => {}  
        () = &mut sleep => {
          println!("timer elapsed");
        }
    }
  }
    #[tokio::test(flavor = "multi_thread")]
    async fn client_server() {
      let sleep = time::sleep(Duration::from_millis(10000));
      tokio::pin!(sleep);
      select!{
        _ = tokio::spawn(async move {
          let subscribe_bitstamp: String = json!({
            "event": "bts:subscribe",
            "data": {
              "channel": "order_book_ethbtc"
            }
          }).to_string();       
          let bitstamp: String = "wss://ws.bitstamp.net".to_owned();
          let binance: String = "wss://stream.binance.com:9443/ws/ethbtc@depth20@100ms".to_owned();
          let exchanges = vec![
            (Exchange::Binance(binance), None),
            (Exchange::Bitstamp(bitstamp), Some(subscribe_bitstamp))];
          grpc_server(exchanges).await
        }) => {}
        _ = tokio::spawn(async move {
          grpc_client().await
        }) => {}
        () = &mut sleep => {
          println!("timer elapsed");
        }
      }
    }
}