use std::io;

fn main() -> io::Result<()> {
  tonic_build::configure()
  .build_server(false)
  .build_client(true)
  .type_attribute("Summary", "#[derive(serde::Deserialize, serde::Serialize)]")
  .type_attribute("Level", "#[derive(serde::Deserialize, serde::Serialize)]")
  .compile(&["../proto/orderbook.proto"], &["../proto"])
}
  
