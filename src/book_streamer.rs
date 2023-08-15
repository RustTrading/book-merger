use crate::{AGGREGATOR, WATCHSTATE};
use crate::connector::connect_exchange;
use crate::client::error::Error;
use crate::exchange_tools::{Exchange, Summary, Level};
use futures::{try_join, Stream};
use std::pin::Pin;
use tokio::sync::mpsc;
use tonic::{transport::Server, Request, Response, Status};
use num_traits::cast::ToPrimitive;
use proto::orderbook_aggregator_server::{OrderbookAggregatorServer, OrderbookAggregator};

#[allow(non_snake_case)]
mod proto {
  tonic::include_proto!("book_merger");
}

#[derive(Default)]
pub struct BookStreamerTonik {}

pub struct BookStreamer {
  pub exchanges: Vec<(Exchange, Option<String>)>,
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

use itertools::Itertools;

impl BookStreamer {
  pub async fn run(&mut self) -> Result<(), Error> {
    let (tx, mut rx) = mpsc::channel(100);
    let tx2 = tx.clone();
    let (exchange1, exchange2) = self.exchanges.clone().into_iter().collect_tuple().unwrap();
    match try_join!(
      tokio::spawn(async move { connect_exchange(exchange1.clone().0, exchange1.1.clone(), tx).await }),
      tokio::spawn(async move { connect_exchange(exchange2.clone().0, exchange2.1.clone(), tx2).await }),
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
        .add_service(OrderbookAggregatorServer::new(BookStreamerTonik {}))
        .serve(addr)
        .await;
    })
  ) {
     Ok(_) => Ok(()),
     Err(v) => Err(Error::JoinError(v)),
  }
  }
}