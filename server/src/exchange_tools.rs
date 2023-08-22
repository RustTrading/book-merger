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

pub const BITSTAMP_WSS: &str = "wss://ws.bitstamp.net";
pub const BINANCE_WSS: &str = "wss://stream.binance.com:9443/ws/{}@depth10@100ms";

pub const BITSTAMP_API: &str = "https://www.bitstamp.net/api/v2/order_book/{}/";
pub const BINANCE_API: &str = "https://api.binance.com/api/v3/depth?symbol={}&limit=100";

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OrderBook {
  pub exchange: String,
  pub bids: Vec<Level>,
  pub asks: Vec<Level>,
}

impl From<binance::OrderBook> for OrderBook {
  fn from(ob: binance::OrderBook) -> Self {
    Self {
      exchange: String::from("binance"),
      asks: ob.asks.into_iter().map(|(price, amount)| Level { exchange: String::from("binance"), price, amount}).collect(),
      bids: ob.bids.into_iter().map(|(price, amount)| Level { exchange: String::from("binance"), price, amount}).collect(),
    }
  }
}

impl From<bitstamp::OrderBook> for OrderBook {
  fn from(ob: bitstamp::OrderBook) -> Self {
    Self {
      exchange: String::from("bitstamp"),
      asks: ob.asks.into_iter().map(|(price, amount)| Level { exchange: String::from("bitstamp"), price, amount}).collect(),
      bids: ob.bids.into_iter().map(|(price, amount)| Level { exchange: String::from("bitstamp"), price, amount}).collect(),
    }
  }
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
  pub currency_pair: String,
  pub last_update_id_binance: Decimal,
  pub last_update_id_bitstamp: Decimal,
  pub update_counter: i64,
  pub asks: BTreeMap<Decimal, HashMap<String, Decimal>>,
  pub bids: BTreeMap<Decimal, HashMap<String, Decimal>>,
  pub spread: Decimal,
}

impl AggregatedBook {
  pub fn new(currency_pair: String) -> Self {
    Self {
      currency_pair,
      last_update_id_binance: dec!(0),
      last_update_id_bitstamp: dec!(0),
      update_counter: 0,
      asks: BTreeMap::new(),
      bids: BTreeMap::new(),
      spread: dec!(0),
    }
  }


  pub fn dump_levels(&self, side: OrderSide) -> Box<dyn DoubleEndedIterator<Item = Vec<Level>> + '_> {
    let levels = match side {
      OrderSide::Ask => &self.asks,
      OrderSide::Bid => &self.bids,
    };
    Box::new(levels.iter()
    .map(move |val| val.1.iter()
    .map(|val2| Level { exchange: val2.0.clone(), price: *val.0, amount: *val2.1 })
    .filter(|val| val.amount.to_f64().unwrap() > 0.0).collect::<Vec<Level>>()))
  }

  pub fn get_levels(&self, level_num: usize) -> Summary {
    let bids: Vec<_> = self.dump_levels(OrderSide::Bid).rev().take(level_num)
    .flatten().collect();
    let asks: Vec<_> = self.dump_levels(OrderSide::Ask).take(level_num)
    .flatten().collect();
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
  pub async fn update(&mut self, orderbook: OrderBook) {
    let mut ob_vec = vec![orderbook];
    if self.update_counter > 1000 || self.update_counter == 0 {
      let body = reqwest::get(BINANCE_API.replace("{}", &self.currency_pair.to_ascii_uppercase()))
      .await.unwrap().text().await.unwrap();
      println!("updating snapshort binance...");
      let order_book_binance: binance::OrderBook = serde_json::from_str(&body).unwrap();
      let body = reqwest::get(BITSTAMP_API.replace("{}", &self.currency_pair))
      .await.unwrap().text().await.unwrap();
      println!("updating snapshort bitstamp...");
      let order_book_bitstamp: bitstamp::OrderBook = serde_json::from_str(&body).unwrap();
      self.update_counter = 1;
      self.last_update_id_binance = order_book_binance.lastUpdateId;
      self.last_update_id_bitstamp = order_book_bitstamp.timestamp;
      ob_vec = vec![order_book_binance.into(), order_book_bitstamp.into()];
      self.asks.clear();
      self.bids.clear();
    }
    for ob in ob_vec {
      for side in OrderSide::iter() { 
        let storage = match side {
          OrderSide::Ask => &ob.asks,
          OrderSide::Bid => &ob.bids
        };
        for level in storage.into_iter() {
          self.insert_level(side, level.clone());
        }
      }
    }
    self.spread = self.asks.first_key_value().unwrap().0 - self.bids.last_key_value().unwrap().0;
    self.update_counter += 1;
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

async fn sync_update(prev_update_state: Option<Arc<RwLock<i64>>>, current_state : Decimal) -> bool {
  let mut update_is_ok = false;
  if let Some(prev_state) = prev_update_state {
    let update_state = current_state.to_i64().unwrap();
    if update_state >= *prev_state.read().await + 1 {
      *prev_state.write().await = update_state;
      update_is_ok = true;
    }
  } else {
    update_is_ok = true;
  } 
  update_is_ok
}

pub async fn parse_book(exchange: Exchange, message: &str, prev_update_state: Option<Arc<RwLock<i64>>>) -> Result<OrderBook, Error> {
  //println!("received message: {:?}", message);
  match exchange {
    Exchange::Bitstamp(_) => {
      let order_book: Result<bitstamp::Event,_> = serde_json::from_str(message);
      match order_book {
        Ok(val) => { 
          if sync_update(prev_update_state, val.data.timestamp).await {
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
      let order_book: Result<binance::OrderBook, _> = serde_json::from_str(message);
      match order_book {
        Ok(val) => { 
          if sync_update(prev_update_state, val.lastUpdateId).await {
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
      let order_book: Result<binance::OrderBook, _> = serde_json::from_str(message);
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

#[cfg(test)]
pub mod test {
  use crate::exchange_tools::Level;
  use num_traits::cast::ToPrimitive;
  use rust_decimal::Decimal;
  use rust_decimal_macros::dec;
  use std::collections::{BTreeMap, HashMap};
#[test] 
fn collect_echanges() {
  let mut test_map: BTreeMap<Decimal, HashMap<String, Decimal>> = BTreeMap::new();
    
  test_map.insert(dec!(1), HashMap::<String, Decimal>::from([(String::from("ex1"), dec!(123)),(String::from("ex2"), dec!(124))]));
  test_map.insert(dec!(2), HashMap::<String, Decimal>::from([(String::from("ex1"), dec!(125)),(String::from("ex2"), dec!(126))]));
  test_map.insert(dec!(3), HashMap::<String, Decimal>::from([(String::from("ex1"), dec!(127))]));
  test_map.insert(dec!(4), HashMap::<String, Decimal>::from([(String::from("ex2"), dec!(128))]));

  let non_flattened: Vec<Vec<_>> =  test_map.iter()
    .map(move |val| val.1.iter()
    .map(|val2| Level { exchange: val2.0.clone(), price: *val.0, amount: *val2.1 })
    .filter(|val| val.amount.to_f64().unwrap() > 0.0).collect()).take(3).collect();
    assert!(non_flattened.len() == 3);
  } 
}