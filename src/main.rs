
use book_merger::exchange_tools::{BINANCE_WSS, BITSTAMP_WSS, Exchange};
use book_merger::book_streamer::BookStreamer;
use serde_json::json;
use book_merger::client::error::Error;
use clap::{Arg, App};

async fn grpc_server(exchanges: Vec<(Exchange, Option<String>)>) -> Result<(), Error> {
  let mut worker = BookStreamer { 
    exchanges
    };  
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
  use futures_util::try_join;
  use book_merger::{client::error::Error, test::server, exchange_tools::Exchange, client::grpc_client};
  use crate::grpc_server;
  #[tokio::test(flavor = "multi_thread")] 
    async fn mock_servers() -> Result<(), Error> {
      let exchanges = vec![
        (Exchange::Binance("ws://127.0.0.1:8080".to_owned()), None),
        (Exchange::Other("ws://127.0.0.1:3030".to_owned()), None)
      ];
      match try_join!(
        tokio::spawn(async move {
           server("127.0.0.1:8080".to_owned()).await
        }),
        tokio::spawn(async move {
          server("127.0.0.1:3030".to_owned()).await
        }),
        tokio::spawn(async move {
          grpc_server(exchanges).await
        }),
        tokio::spawn(async move {
          grpc_client().await
        })
    ) {
        Ok(_) => Ok(()),
        Err(e) => Err(Error::JoinError(e))
      } 
    }
}