use book_merger_web::{Plotters};

fn main() {
  wasm_logger::init(wasm_logger::Config::default());
  yew::Renderer::<Plotters>::new().render();
}
