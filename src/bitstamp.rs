#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OrderBook
{
  timestamp: String,
  microtimestamp: String,
  bids: Vec<(f64, f64)>,
  asks: Vec<(f64, f64)>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Message {
  event: String,
  channel: String,
  data:  OrderBook,
}