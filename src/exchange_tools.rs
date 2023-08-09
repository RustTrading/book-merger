use serde::{Serialize, Deserialize};
use tokio_tungstenite::tungstenite::protocol::Message;
use rust_decimal::Decimal;

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
  pub bids: Vec<(Decimal, Decimal)>,
  pub asks: Vec<(Decimal, Decimal)>,
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
