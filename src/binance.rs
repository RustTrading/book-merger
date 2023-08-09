use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OrderBook {
  lastUpdateId: u64,
  bids: Vec<(f64, f64)>,
  asks: Vec<(f64, f64)>
}