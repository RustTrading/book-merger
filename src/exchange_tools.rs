use serde::{Serialize, Deserialize};
use tokio_tungstenite::tungstenite::protocol::Message;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::{BTreeMap, HashMap};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

pub const BITSTAMP_WSS: &str = "wss://ws.bitstamp.net";
pub const BINANCE_WSS_ETHBTC_20: &str = "wss://stream.binance.com:9443/ws/ethbtc@depth20@100ms";

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OrderBook {
  pub exchange: String,
  pub bids: Vec<Level>,
  pub asks: Vec<Level>,
}

#[derive(Debug, Clone, Copy)]
pub enum Exchanges {
  Bitstamp(&'static str),
  Binance(&'static str),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Level {
  pub exchange: String,
  pub price: Decimal,
  pub amount: Decimal,
}
#[derive(Debug, EnumIter, Clone, Copy)]
pub enum OrderSide {
  Ask,
  Bid,
}

#[derive(Debug)]
pub struct Summary {
  pub asks: Vec<Level>,
  pub bids: Vec<Level>,
  pub spread: Decimal,
}

pub struct AggregatedBook {
  pub asks: BTreeMap<Decimal, HashMap<String, Decimal>>,
  pub bids: BTreeMap<Decimal, HashMap<String, Decimal>>,
  pub spread: Decimal,
}

impl AggregatedBook {
  pub fn new() -> Self {
    Self {
      asks: BTreeMap::new(),
      bids: BTreeMap::new(),
      spread: dec!(0),
    }
  }

  pub fn get_levels(&self, level_num: usize) -> Summary {
    let bids = self.bids.iter()
    .map(|val| val.1.iter()
    .map(|val2| Level { exchange: val2.0.clone(), price: *val.0, amount: *val2.1 }).collect::<Vec<Level>>())
    .flatten()
    .rev()
    .take(level_num)
    .collect();
    let asks = self.asks.iter()
    .map(|val| val.1.iter()
    .map(|val2| Level { exchange: val2.0.clone(), price: *val.0, amount: *val2.1 })
    .collect::<Vec<Level>>())
    .flatten()
    .take(level_num)
    .collect();
    Summary { asks, bids, spread: self.spread }
  } 

  pub fn insert_level(&mut self, order_side: OrderSide, level: Level) {
    let storage = match order_side {
        OrderSide::Ask => {
          &mut self.asks
        },
        OrderSide::Bid => {
          &mut self.bids
        }
    };
    if !storage.contains_key(&level.price) {
      storage.insert(level.price, HashMap::new());
    }
    if let Some(amount_map)= storage.get_mut(&level.price) {
      amount_map.insert(level.exchange, level.amount);
    }
  }
  pub fn update(&mut self, orderbook: OrderBook) {
    for side in OrderSide::iter() { 
      let storage = match side {
        OrderSide::Ask => &orderbook.asks,
        OrderSide::Bid => &orderbook.bids
      };
      for level in storage.into_iter() {
        self.insert_level(side, level.clone());
      }
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

impl ToString for Exchanges {
  fn to_string(&self) -> String {
    match self {
      Self::Binance(_) => String::from("binance"),
      Self::Bitstamp(_) => String::from("bitstamp"),
    }
  }
}

use crate::{binance, bitstamp};

pub fn parse_book(exchange: Exchanges, message: Message) -> Result<OrderBook, serde_json::Error> {
 let message_str =  message.to_text().unwrap();
  match exchange {
    Exchanges::Bitstamp(_) => {
      let order_book: Result<bitstamp::Event,_> = serde_json::from_str(message_str);
      match order_book {
        Ok(val) => { 
          Ok(OrderBook {
            exchange: String::from("bitstamp"),
            asks: val.data.asks.into_iter().map(|(price, amount)| Level { exchange: String::from("bitstamp"), price, amount}).collect(),
            bids: val.data.bids.into_iter().map(|(price, amount)| Level { exchange: String::from("bitstamp"), price, amount}).collect(),
          })
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
          Ok(OrderBook {
            exchange: String::from("binance"),
            asks: val.asks.into_iter().map(|(price, amount)| Level { exchange: String::from("binance"), price, amount}).collect(),
            bids: val.bids.into_iter().map(|(price, amount)| Level { exchange: String::from("binance"), price, amount}).collect(),
          })
        },
        Err(e) =>  { 
          Err(e)
        },
      }
    }
  }
}
