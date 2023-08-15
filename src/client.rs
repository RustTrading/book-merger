use proto::orderbook_aggregator_client::OrderbookAggregatorClient;
pub mod error {
  use tokio_tungstenite::tungstenite;

#[derive(Debug)]
pub enum Error {
  BadConnection(tungstenite::Error),
  BadData(serde_json::Error),
  IoError(std::io::Error),
  ServerError(tonic::transport::Error),
  BadAddr(std::net::AddrParseError),
  JoinError(tokio::task::JoinError),
  NotImplemented(),
  Status(tonic::Status),
}

impl From<tonic::Status> for Error {
  fn from(e: tonic::Status) -> Self {
    Self::Status(e)
  }
}

impl From<tungstenite::Error> for Error {
  fn from(e: tungstenite::Error) -> Self {
    Self::BadConnection(e)
  }
}

impl From<serde_json::Error> for Error {
  fn from(e: serde_json::Error) -> Self {
    Self::BadData(e)
  }
}

impl From<std::io::Error> for Error {
  fn from(e: std::io::Error) -> Self {
    Self::IoError(e)
  }
}

impl From<tonic::transport::Error> for Error {
  fn from(e: tonic::transport::Error) -> Self {
    Self::ServerError(e)
  }
}

impl From<std::net::AddrParseError> for Error {
  fn from(e: std::net::AddrParseError) -> Self {
    Self::BadAddr(e)
  }
}
}

mod proto {
  tonic::include_proto!("book_merger");
}

pub async fn grpc_client() -> Result<(), error::Error> {
  let mut client = OrderbookAggregatorClient::connect("http://[::1]:50051").await?;
  let request = tonic::Request::new(proto::Empty {});
  let mut response = client.book_summary(request).await?.into_inner();
  while let Some(res) = response.message().await? {
     println!("{:?}", res);
  }
  Ok::<(), error::Error>(())
}

#[tokio::main]
async fn main() -> Result<(), error::Error> {
  grpc_client().await
}
