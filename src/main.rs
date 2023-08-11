use futures_util::{StreamExt, SinkExt};
use std::sync::Arc;
use std::pin::Pin;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message, tungstenite::error::Error};
use tokio::sync::{mpsc, watch, RwLock};
use futures::join;
use url::Url;
use tonic::{transport::Server, Request, Response, Status};
use crate::proto::orderbook_aggregator_server::{OrderbookAggregatorServer, OrderbookAggregator};
use book_merger::exchange_tools::{BINANCE_WSS, BITSTAMP_WSS, Exchanges, parse_book, AggregatedBook, OrderBook, Summary};
use serde_json::json;
use num_traits::cast::ToPrimitive;
use futures::Stream;
use clap::{Arg, App};

mod proto {
  tonic::include_proto!("book_merger");
}

async fn connect_exchange(exchange: Exchanges, subscriber : Option<String>, tx :mpsc::Sender<OrderBook>) -> Result<(), Error> {
  let exchange_stream = exchange.clone().value();
  let url = Url::parse(&exchange_stream).expect("bad url string");
  let (ws_stream, _) = connect_async(url).await?;
  println!("connected {:?}", exchange);
  let (mut out_stream, input_stream) = ws_stream.split();
  if let Some(message) = subscriber {
    println!("subscribing...");
    match out_stream.send(Message::Text(message))
    .await {
      Ok(_) => println!("ok"),
      Err(e) => println!("{:?}", e)
    };
  }  
  let read_future = input_stream.for_each(|message| async {
    if let Ok(order_book) = parse_book(exchange.clone(), message.unwrap()) {
      let _ = tx.send(order_book).await;
    }
  });
  read_future.await;
  Ok::<(), Error>(())
}

#[derive(Default)]
pub struct BookStreamer {}

fn get_prop_levels(levels: &Vec<book_merger::exchange_tools::Level>) -> Vec<proto::Level> {
  levels.iter()
    .map(|l|
      proto::Level{
              exchange: l.exchange.clone(),
              price: l.price.to_f64().unwrap(),
              amount: l.amount.to_f64().unwrap(),
          })
      .collect()
}

impl From<Summary> for proto::Summary {
  fn from(summary: Summary) -> Self {
      let spread = summary.spread.to_f64().unwrap();
      let bids: Vec<proto::Level> = get_prop_levels(&summary.bids);
      let asks: Vec<proto::Level> = get_prop_levels(&summary.asks);
      proto::Summary{ spread, bids, asks }
  }
} 

#[tonic::async_trait]
impl OrderbookAggregator for BookStreamer {
  type BookSummaryStream = Pin<Box<dyn Stream<Item = Result<proto::Summary, Status>> + Send + 'static>>;
    async fn book_summary(
        &self,
        request: Request<proto::Empty>,
    ) -> Result<Response<Self::BookSummaryStream>, Status> {
      println!("Got a request from {:?}", request.remote_addr());
      let output = async_stream::try_stream! {
        let mut receiver = WATCHSTATE.read().await.1.clone();
        while let Ok(_) = receiver.changed().await {
          let aggregation = AGGREGATOR.read().await;
          let summary = aggregation.get_levels(10);
          yield proto::Summary::from(summary);
        }
      };
    Ok(Response::new(Box::pin(output) as Self::BookSummaryStream))
  } 
}

#[macro_use]
extern crate lazy_static;

lazy_static! {
  static ref AGGREGATOR: Arc<RwLock<AggregatedBook>> = Arc::new(RwLock::new(AggregatedBook::new()));
  static ref WATCHSTATE: Arc<RwLock<(watch::Sender<bool>, watch::Receiver<bool>)>> = Arc::new(RwLock::new(watch::channel(false)));
}

#[tokio::main]
async fn main() -> Result<(), Error> {
  let matches = App::new("book-merger")
    .version("0.0.1")
    .about("create combined orderbook for Binance and Bitstamp")
    .arg(Arg::new("currencies")
    .long("currencies")
    .required(false)
    .takes_value(true)
    .help("provides pair of currencies for orderbook data")
  )
  .get_matches();
  let mut currencies = "ethbtc";
   
  let frmt = json!({
    "event": "bts:subscribe",
    "data": {
      "channel": "order_book_{}"
    }
  }).to_string();
  
  if matches.is_present("currencies") {
    if let Some(val) = matches.value_of("currencies") {
      currencies = val; 
   }
  }
  let subscribe_ethbtc: String = frmt.replace("{}", currencies);
  let binace_wss_currency = BINANCE_WSS.replace("{}", currencies);
  let (tx, mut rx) = mpsc::channel(100);
  let tx2 = tx.clone();
  let _= join!(
    tokio::spawn(async move { connect_exchange(Exchanges::Bitstamp(BITSTAMP_WSS.to_owned()), Some(subscribe_ethbtc), tx).await }),
    tokio::spawn(async move { connect_exchange(Exchanges::Binance(binace_wss_currency), None, tx2).await }),
    tokio::spawn(async move { 
      while let Some(res) = rx.recv().await {
        let mut aggr = AGGREGATOR.write().await;
        aggr.update(res);
        match WATCHSTATE.read().await.0.send(true) {
          Err(e) => println!("{}", e),
          _ => {}
        }
      }
    }),
    tokio::spawn(async move { 
      let addr = "[::1]:50051".parse().unwrap();
      println!("Server listening on {}", addr);
      let _= Server::builder()
        .add_service(OrderbookAggregatorServer::new(BookStreamer::default()))
        .serve(addr)
        .await;
    })
);
  Ok(())
}
