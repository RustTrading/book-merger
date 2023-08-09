use serde::{Serialize, Deserialize};
use tokio_tungstenite::tungstenite::protocol::Message;

pub const BITSTAMP_WSS: &str = "wss://ws.bitstamp.net";
pub const BINANCE_WSS_ETHBTC_20: &str = "wss://stream.binance.com:9443/ws/ethbtc@depth20@100ms";


#[derive(Debug, Clone, Copy)]
pub enum Exchanges {
  Bitstamp(&'static str),
  Binance(&'static str),
}

impl Exchanges {
  pub fn value(self) -> &'static str {
    match self {
      Self::Bitstamp(value) => value,
      Self::Binance(value) => value,
    }
  }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OrderBook {
  bids: Vec<(f64, f64)>,
  asks: Vec<(f64, f64)>,
}

pub fn parse_book(exchange: Exchanges, message: Message) -> OrderBook {
  println!("{:?}, {:?}", exchange, message.to_text());
  OrderBook { bids: Vec::new(), asks: Vec::new() }
}
