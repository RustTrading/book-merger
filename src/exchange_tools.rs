use serde::{Serialize, Deserialize};
use tokio_tungstenite::tungstenite::protocol::Message;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::BTreeMap;

pub const BITSTAMP_WSS: &str = "wss://ws.bitstamp.net";
pub const BINANCE_WSS_ETHBTC_20: &str = "wss://stream.binance.com:9443/ws/ethbtc@depth20@100ms";

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OrderBook {
  pub bids: Vec<(Decimal, Decimal)>,
  pub asks: Vec<(Decimal, Decimal)>,
}

#[derive(Debug, Clone, Copy)]
pub enum Exchanges {
  Bitstamp(&'static str),
  Binance(&'static str),
}

pub struct AggregatedBook {
  asks: BTreeMap<Decimal, Decimal>,
  bids: BTreeMap<Decimal, Decimal>,
  spread: Decimal,
}

impl AggregatedBook {
  pub fn new() -> Self {
    Self {
      asks: BTreeMap::new(),
      bids: BTreeMap::new(),
      spread: dec!(0),
    }
  } 
  pub fn update(&mut self, orderbook: OrderBook) {
     for (price, volume) in orderbook.asks.into_iter() {
      let mut aggregated = volume;  
      if let Some(value) = self.asks.get(&price) {
          aggregated += value;
        } 
        self.asks.insert(price, aggregated);
     }
     for (price, volume) in orderbook.bids.into_iter() {
      let mut aggregated = volume;  
      if let Some(value) = self.asks.get(&price) {
          aggregated += value;
        } 
        self.bids.insert(price, aggregated);
     }
     self.spread = self.asks.first_key_value().unwrap().0 - self.bids.last_key_value().unwrap().0;
  }
}


impl Exchanges {
  pub fn value(self) -> &'static str {
    match self {
      Self::Bitstamp(value) => value,
      Self::Binance(value) => value,
    }
  }
}

use crate::{binance, bitstamp};

pub fn parse_book(exchange: Exchanges, message: Message) -> Result<(Decimal, OrderBook), serde_json::Error> {
 let message_str =  message.to_text().unwrap();
  match exchange {
    Exchanges::Bitstamp(_) => {
      let order_book: Result<bitstamp::Event,_> = serde_json::from_str(message_str);
      match order_book {
        Ok(val) => { 
          Ok((val.data.timestamp, OrderBook {
            asks: val.data.asks,
            bids: val.data.bids,
          }))
        },
        Err(e) => { 
          Err(e)
        }
      }
    },
    Exchanges::Binance(_) => {
      let order_book: Result<binance::OrderBook, _> = serde_json::from_str(message_str);
      match order_book {
        Ok(val) => { 
          Ok((val.lastUpdateId, OrderBook {
            asks: val.asks,
            bids: val.bids,
          }))
        },
        Err(e) =>  { 
          Err(e)
        },
      }
    }
  }
}
