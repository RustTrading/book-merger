pub mod exchange_tools;
pub mod book_streamer;
pub mod connector;
pub mod error;
mod bitstamp;
mod binance;

use std::sync::Arc;
use tokio::sync::{RwLock, watch};
use exchange_tools::AggregatedBook;

#[macro_use] extern crate lazy_static;
lazy_static! {
  static ref AGGREGATOR: Arc<RwLock<AggregatedBook>> = Arc::new(RwLock::new(AggregatedBook::new()));
  static ref WATCHSTATE: Arc<RwLock<(watch::Sender<bool>, watch::Receiver<bool>)>> = Arc::new(RwLock::new(watch::channel(false)));
}
