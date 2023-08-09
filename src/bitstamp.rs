
use rust_decimal::Decimal;
use serde::{Serialize, Deserialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OrderBook
{
  pub timestamp: Decimal,
  pub microtimestamp: Decimal,
  pub bids: Vec<(Decimal, Decimal)>,
  pub asks: Vec<(Decimal, Decimal)>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Event {
  pub event: String,
  pub channel: String,
  pub data: OrderBook,
}
