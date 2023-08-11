use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OrderBook { 
  pub lastUpdateId: Decimal,
  pub bids: Vec<(Decimal, Decimal)>,
  pub asks: Vec<(Decimal, Decimal)>
}