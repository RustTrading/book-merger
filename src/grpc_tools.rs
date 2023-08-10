use crate::error::Error;
//use crate::orderbook::{self, OutTick};
//use crate::orderly::OutTickPair;
use crate::exchange_tools::{Summary, Level};
use futures::Stream;
use log::info;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal_macros::dec;
use std::pin::Pin;
//use std::sync::Arc;
//use tokio::sync::RwLock;
use tonic::{transport::Server, Request, Response, Status};

//pub mod proto {
    //tonic::include_proto!("orderbook");
//}
/* 
pub struct OrderBookService {
    //out_ticks: Arc<RwLock<OutTickPair>>,
}

impl OrderBookService {
    pub fn new(/*out_ticks: Arc<RwLock<OutTickPair>>*/) -> Self {
        OrderBookService { /*out_ticks*/ }
    }

    pub async fn serve(self, port: usize) -> Result<(), Error>{
        let addr = format!("[::1]:{}", port);
        let addr = addr.parse()?;

        info!("Serving grpc at {}", addr);

        Server::builder()
            .add_service(proto::orderbook_aggregator_server::OrderbookAggregatorServer::new(self))
            .serve(addr)
            .await?;

        Ok(())
    }
}

impl From<Summary> for proto::Summary {
  fn from(sum: Summary) -> Self {
    let spread = sum.spread.to_f64().unwrap();
    let bids: Vec<proto::Level> = to_levels(&sum.bids);
    let asks: Vec<proto::Level> = to_levels(&sum.asks);

    proto::Summary{ spread, bids, asks }
  }
}

fn to_levels(levels: &Vec<Level>) -> Vec<proto::Level> {
  levels.iter()
  .map(|l|
            proto::Level{
                exchange: l.exchange,
                price: l.price.to_f64().unwrap(),
                amount: l.amount.to_f64().unwrap(),
            })
        .collect()
}
 
#[tonic::async_trait]
impl proto::orderbook_aggregator_server::OrderbookAggregator for OrderBookService {
    async fn check(
        &self,
        request: Request<proto::Empty>,
    ) -> Result<Response<proto::Summary>, Status> {
        //info!("Got a request: {:?}", request);

        let _req = request.into_inner();

        //let out_tick = self.out_tick().await;

        let reply = proto::Summary::from(Summary { spread: dec!(0), asks: vec![], bids: vec![]});

        Ok(Response::new(reply))
    }

    type BookSummaryStream =
        Pin<Box<dyn Stream<Item = Result<proto::Summary, Status>> + Send + 'static>>;

    async fn book_summary(
        &self,
        request: Request<proto::Empty>,
    ) -> Result<Response<Self::BookSummaryStream>, Status> {
        //info!("Got a request: {:?}", request);

        let _req = request.into_inner();
        //let output = async_stream::try_stream!(vec![Ok(Summary { spread: dec!(0), asks: vec![], bids: vec![]})]);
        /* let mut rx_out_ticks = self.out_ticks.read().await.1.clone();
         */
        let output = async_stream::try_stream! {
          // yield the current value
          yield proto::Summary::from(Summary { spread: dec!(0), asks: vec![], bids: vec![]});
        };
        
        Ok(Response::new(Response::new(Box::pin(output) as Self::BookSummaryStream)))
    }
}
*/
