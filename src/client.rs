use clap::Parser;
//use proto::orderbook_aggregator_client::OrderbookAggregatorClient;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal_macros::dec;

use crate::proto::greeter_client::GreeterClient;
use crate::proto::HelloRequest;

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
    let mut client = GreeterClient::connect("http://[::1]:50051").await?;

    let request = tonic::Request::new(HelloRequest {
        name: "Tonic".into(),
    });

    let response = client.say_hello(request).await?;

    println!("RESPONSE={:?}", response);

    Ok(())
}
