pub const BITSTAMP_WSS: &str = "wss://ws.bitstamp.net";
pub const BINANCE_WSS_ETHBTC_20: &str = "wss://stream.binance.com:9443/ws/ethbtc@depth20@100ms";

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

/* 
pub fn parse_book(exchangeWSS: &str, message: Message) -> OrderBook {
  
}
*/