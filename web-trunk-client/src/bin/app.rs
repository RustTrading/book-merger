use book_merger_web::OrderWeb;

fn main() {
  wasm_logger::init(wasm_logger::Config::default());
  yew::Renderer::<OrderWeb>::new().render();
}
