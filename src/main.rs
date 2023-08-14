
use book_merger::exchange_tools::{BINANCE_WSS, BITSTAMP_WSS, Exchange};
use book_merger::book_streamer::BookStreamer;
use serde_json::json;
use book_merger::error::Error;
use clap::{Arg, App};

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

  let mut worker = BookStreamer { 
    exchanges: vec![
      (Exchange::Binance(binance_wss_currency), None),
      (Exchange::Bitstamp(BITSTAMP_WSS.to_owned()), Some(subscribe_ethbtc))],
    };  
    worker.run().await
}
