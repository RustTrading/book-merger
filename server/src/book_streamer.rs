use crate::connector::connect_exchange;
use crate::client::error::Error;
use crate::exchange_tools::{AggregatedBook, Exchange, Summary, Level};
use futures::try_join;
use num_traits::cast::ToPrimitive;
use proto::orderbook_aggregator_server::{OrderbookAggregatorServer, OrderbookAggregator};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock, watch};
use tokio_stream::wrappers::ReceiverStream;
use tonic_web::GrpcWebLayer;
use tonic::{transport::Server, Request, Response, Status};
use tower_http::cors::{Any, CorsLayer};

#[allow(non_snake_case)]
mod proto {
  tonic::include_proto!("book_merger");
}

pub struct BookStreamerTonik {
  pub aggregator: Arc<RwLock<AggregatedBook>>,
  pub watcher: Arc<RwLock<watch::Receiver<bool>>>,
}

pub struct BookStreamer {
  pub exchanges: Vec<(Exchange, Option<String>)>,
  pub aggregator: Arc<RwLock<AggregatedBook>>
}

impl BookStreamer {
  pub fn new(exchanges: Vec<(Exchange, Option<String>)>) -> Self {
    Self {
      exchanges,
      aggregator: Arc::new(RwLock::new(AggregatedBook::new())),
    }
  }
}

fn get_prop_levels(levels: &Vec<Level>) -> Vec<proto::Level> {
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
impl OrderbookAggregator for BookStreamerTonik {
  type BookSummaryStream = ReceiverStream<Result<proto::Summary, Status>>;
    async fn book_summary(
        &self,
        request: Request<proto::Empty>,
    ) -> Result<Response<Self::BookSummaryStream>, Status> {
      println!("Got a request from {:?}", request.remote_addr());
      let (tx, rx) = mpsc::channel::<Result<proto::Summary, Status>>(100);
      let agg = self.aggregator.clone();
      let watcher = self.watcher.clone();
      tokio::spawn(async move {
        while let Ok(_) = watcher.read().await.clone().changed().await {
          let aggregation = agg.read().await;
          let summary = aggregation.get_levels(10);
          let _ = tx.send(Ok(proto::Summary::from(summary))).await;
        }
      });
      Ok(Response::new(ReceiverStream::new(rx)))
  } 
}

use itertools::Itertools;

impl BookStreamer {
  pub async fn run(&mut self) -> Result<(), Error> {
    let (tx, mut rx) = mpsc::channel(100);
    let tx2 = tx.clone();
    let (exchange1, exchange2) = self.exchanges.clone().into_iter().collect_tuple().unwrap();
    let aggregator = self.aggregator.clone();
    let aggregator_ = self.aggregator.clone();
    let (tx_w, rx_w)= watch::channel(false);
    match try_join!(
      tokio::spawn(async move { connect_exchange(exchange1.clone().0, exchange1.1.clone(), tx).await }),
      tokio::spawn(async move { connect_exchange(exchange2.clone().0, exchange2.1.clone(), tx2).await }),
      tokio::spawn(async move { 
        while let Some(res) = rx.recv().await {
          let mut aggregator_guard = aggregator_.write().await;
          aggregator_guard.update(res);
          match tx_w.send(true) {
            Err(e) => println!("{}", e),
            _ => {}
          }
        }
    }),
    tokio::spawn(async move { 
      let addr = "[::1]:50051".parse().unwrap();
      println!("Server listening on {}", addr);
      Server::builder()
      .accept_http1(true)
      .layer(
          CorsLayer::new()
          .allow_headers(Any)
          .allow_origin(["http://localhost:8080".parse().unwrap()])
          .expose_headers(Any)
      )
      .layer(GrpcWebLayer::new())
      .add_service(OrderbookAggregatorServer::new(BookStreamerTonik { aggregator, watcher: Arc::new(RwLock::new(rx_w)) }))
      .serve(addr)
      .await
    })
  ) {
     Ok(_) => Ok(()),
     Err(v) => Err(Error::JoinError(v)),
  }
  }
}