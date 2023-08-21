use crate::client::error::Error;
use num_traits::cast::ToPrimitive;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Serialize, Deserialize};
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use tokio::sync::RwLock;
use tokio_tungstenite::tungstenite::protocol::Message;

pub const BITSTAMP_WSS: &str = "wss://ws.bitstamp.net";
pub const BINANCE_WSS: &str = "wss://stream.binance.com:9443/ws/{}@depth10@100ms";

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OrderBook {
  pub exchange: String,
  pub bids: Vec<Level>,
  pub asks: Vec<Level>,
}

#[derive(Debug, Clone)]
pub enum Exchange {
  Bitstamp(String),
  Binance(String),
  Other(String),
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
    .rev().filter(|val| val.amount.to_f64().unwrap() > 0.0)
    .take(level_num)
    .collect();
    let asks = self.asks.iter()
    .map(|val| val.1.iter()
    .map(|val2| Level { exchange: val2.0.clone(), price: *val.0, amount: *val2.1 })
    .collect::<Vec<Level>>())
    .flatten().filter(|val| val.amount.to_f64().unwrap() > 0.0)
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

impl Exchange {
  pub fn value(self) -> String {
    match self {
      Self::Bitstamp(value) => value,
      Self::Binance(value) => value,
      Self::Other(value) => value
    }
  }
}

impl ToString for Exchange {
  fn to_string(&self) -> String {
    match self {
      Self::Binance(_) => String::from("binance"),
      Self::Bitstamp(_) => String::from("bitstamp"),
      Self::Other(_) => String::from("other"),
    }
  }
}

use crate::{binance, bitstamp};

pub async fn parse_book(exchange: Exchange, message: Message, prev_update_state: Arc<RwLock<(i64, i64)>>) -> Result<OrderBook, Error> {
  //println!("received message: {:?}", message);
  let message_str =  message.to_text().unwrap();
  match exchange {
    Exchange::Bitstamp(_) => {
      let order_book: Result<bitstamp::Event,_> = serde_json::from_str(message_str);
      match order_book {
        Ok(val) => { 
          let update_state = val.data.timestamp.to_i64().unwrap();
          if update_state >= prev_update_state.read().await.0 + 1 {
            prev_update_state.write().await.0 = update_state;
            return Ok(OrderBook {
              exchange: String::from("bitstamp"),
              asks: val.data.asks.into_iter().map(|(price, amount)| Level { exchange: String::from("bitstamp"), price, amount}).collect(),
              bids: val.data.bids.into_iter().map(|(price, amount)| Level { exchange: String::from("bitstamp"), price, amount}).collect(),
            });
          } 
          Err(Error::OutdatedUpdate())
        },
        Err(e) => { 
          Err(e.into())
        }
      }
    },
    Exchange::Binance(_) => {
      let order_book: Result<binance::OrderBook, _> = serde_json::from_str(message_str);
      match order_book {
        Ok(val) => { 
          let update_state = val.lastUpdateId.to_i64().unwrap();
          if update_state >= prev_update_state.read().await.1 + 1 {
            prev_update_state.write().await.1 = update_state;
           return Ok(OrderBook {
            exchange: String::from("binance"),
            asks: val.asks.into_iter().map(|(price, amount)| Level { exchange: String::from("binance"), price, amount}).collect(),
            bids: val.bids.into_iter().map(|(price, amount)| Level { exchange: String::from("binance"), price, amount}).collect(),
          });
          }
          Err(Error::OutdatedUpdate())
        },
        Err(e) =>  { 
          Err(e.into())
        },
      }
    },
    Exchange::Other(_) => {
      let order_book: Result<binance::OrderBook, _> = serde_json::from_str(message_str);
      match order_book {
        Ok(val) => { 
          Ok(OrderBook {
            exchange: String::from("other"),
            asks: val.asks.into_iter().map(|(price, amount)| Level { exchange: String::from("other"), price, amount}).collect(),
            bids: val.bids.into_iter().map(|(price, amount)| Level { exchange: String::from("other"), price, amount}).collect(),
          })
        },
        Err(e) =>  { 
          Err(e.into())
        },
      }
    }
  }
}
