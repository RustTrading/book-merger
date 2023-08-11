use futures_util::{StreamExt, SinkExt};
use proto::{Empty, Summary};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message, tungstenite::error::Error};
use futures::join;
use tokio::sync::mpsc;
use url::Url;
use tonic::{transport::Server, Request, Response, Status};
use crate::proto::*;
use crate::proto::orderbook_aggregator_server::{OrderbookAggregatorServer, OrderbookAggregator};
use book_merger::exchange_tools::{BINANCE_WSS_ETHBTC_20, BITSTAMP_WSS, Exchanges, parse_book, AggregatedBook, OrderBook};
use serde_json::json;

mod proto {
  tonic::include_proto!("book_merger");
}

async fn connect_exchange(exchange: Exchanges, subscriber : Option<String>, tx :mpsc::Sender<OrderBook>) -> Result<(), Error> {
  let exchange_stream = exchange.value();
  let url = Url::parse(exchange_stream).expect("bad url string");
  let (ws_stream, _) = connect_async(url).await?;
  println!("connected");
  let (mut out_stream, input_stream) = ws_stream.split();
  if let Some(message) = subscriber {
    println!("subscribing...");
    match out_stream.send(Message::Text(message))
    .await {
      Ok(res) => println!("{:?}", res),
      Err(e) => println!("{:?}", e)
    };
  }  
  let read_future = input_stream.for_each(|message| async {
    if let Ok(order_book) = parse_book(exchange, message.unwrap()) {
      let _ = tx.send(order_book).await;
      //println!("{:?}, update_id: {:?}, order_book: {:?}", exchange, update_id, order_book);
    }
  });
  read_future.await;
  Ok::<(), Error>(())
}

#[derive(Default)]
pub struct BookStreamer {}

#[tonic::async_trait]
impl OrderbookAggregator for BookStreamer {
    async fn book_summary(
        &self,
        request: Request<proto::Empty>,
    ) -> Result<Response<proto::Summary>, Status> {
        println!("Got a request from {:?}", request.remote_addr());
        let sum = proto::Summary {
          asks: vec![ Level { price: 10.0, amount: 20.0, exchange: String::from("test")}],
          bids: vec![ Level { price: 9.0, amount: 20.0, exchange: String::from("test")}],
          spread: 1.0,
      };
        Ok(Response::new(sum))
    }
}

#[tokio::main]
async fn main() -> Result<(), Error>{
  let subscribe_ethbtc: String = json!({
    "event": "bts:subscribe",
    "data": {
      "channel": "order_book_ethbtc"
    }
  }
  ).to_string();

  let _unsubscribe_eth_btc: String = json!({
    "event": "bts:unsubscribe",
    "data": {
    "channel": "order_book_ethbtc"
  }
  }
  ).to_string();
  let mut aggregated = AggregatedBook::new();
  let (tx, mut rx) = mpsc::channel(100);
  let tx2 = tx.clone();
  let _= join!(
    tokio::spawn(async move { connect_exchange(Exchanges::Bitstamp(BITSTAMP_WSS), Some(subscribe_ethbtc), tx).await }),
    tokio::spawn(async move { connect_exchange(Exchanges::Binance(BINANCE_WSS_ETHBTC_20), None, tx2).await }),
    tokio::spawn(async move { 
      while let Some(res) = rx.recv().await {
        aggregated.update(res);
        let summary = aggregated.get_levels(10);
        println!("{:?}", summary);
      }
    }),
    tokio::spawn(async move { 
      let addr = "[::1]:50051".parse().unwrap();
      let greeter = BookStreamer::default();
      println!("Server listening on {}", addr);
      Server::builder()
          .add_service(OrderbookAggregatorServer::new(greeter))
          .serve(addr)
          .await;
    })
);
  Ok(())
}
