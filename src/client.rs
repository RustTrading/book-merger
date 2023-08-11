use clap::Parser;
//use proto::orderbook_aggregator_client::OrderbookAggregatorClient;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal_macros::dec;

use crate::proto::orderbook_aggregator_client::OrderbookAggregatorClient;
use crate::proto::{Summary, Level};

mod proto {
  tonic::include_proto!("book_merger");
}

/// Connects to the gRPC server and streams the orderbook summary.
#[derive(Parser)]
struct Cli {
  #[clap(short, long, help = "(Optional) Port number of the gRPC server. Default: 50051")]
  port: Option<usize>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = OrderbookAggregatorClient::connect("http://[::1]:50051").await?;

    let request = tonic::Request::new(proto::Empty {});

    let response = client.book_summary(request).await?;

    println!("RESPONSE={:?}", response);

    Ok(())
}
