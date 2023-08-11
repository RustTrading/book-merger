use crate::proto::orderbook_aggregator_client::OrderbookAggregatorClient;

mod proto {
  tonic::include_proto!("book_merger");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = OrderbookAggregatorClient::connect("http://[::1]:50051").await?;

    let request = tonic::Request::new(proto::Empty {});

    let mut response = client.book_summary(request).await?.into_inner();
    while let Some(res) = response.message().await? {
       println!("{:?}", res);
    }
    Ok(())
}
