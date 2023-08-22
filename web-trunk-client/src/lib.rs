use yew::prelude::*;
use gloo::timers::callback::Interval;
use std::rc::Rc;
pub mod components;
use wasm_bindgen::JsValue;

use crate::components::agent::proto::Level;

use crate::components::{ 
  Worker, 
  WorkerInput, 
  WorkerOutput, 
  WorkerRequest,
  OrderTableView,
  SpreadView
};

use yew_agent::Bridged;
pub enum Msg {
  AgentReady(WorkerOutput),
  Redraw,
  WorkerMsg(WorkerOutput),
}
pub struct OrderWeb {
  _update_orderbook_interval: Interval,
  ask_table: Vec<Vec<NodeRef>>,
  bid_table: Vec<Vec<NodeRef>>,
  spread: NodeRef,
  label: NodeRef,
 }

enum OrderBookSide {
  Ask,
  Bid,
}

impl OrderWeb {
  fn update_spread(&self, spread: f64) ->  Result<(), JsValue> {
    let selement = self.spread.cast::<web_sys::Element>().unwrap();
    selement.set_text_content(Some(&spread.to_string()));
    Ok(())
  }
    
  fn update_table(&self, levels: &Vec<Level>, side: OrderBookSide) -> Result<(), JsValue> {
    let table = match side {
        OrderBookSide::Ask => &self.ask_table,
        OrderBookSide::Bid => &self.bid_table,
    };
    let mut row_idx = 0;
    for level in levels {
      let properties = vec![level.price.to_string(), level.amount.to_string(), level.exchange.clone()]; 
      for column_idx in 0..3 {
        let td = (table[row_idx])[column_idx].cast::<web_sys::Element>().unwrap();
        td.set_text_content(Some(&properties[column_idx]));
      } 
      row_idx +=1;
    }
      Ok(())
    }
  }

  impl Component for OrderWeb {

  type Message = Msg;
  type Properties = ();

  fn create(ctx: &Context<Self>) -> Self {
    log::info!("Creating Yew-Plotters View");
    let mut ask_table = Vec::new(); 
    let mut bid_table = Vec::new();
    for _ in 0..20 {
      let vec1 = vec![NodeRef::default(), NodeRef::default(), NodeRef::default()];
      let vec2 = vec![NodeRef::default(), NodeRef::default(), NodeRef::default()];
      ask_table.push(vec1);
      bid_table.push(vec2);
    }
    let spread = NodeRef::default();
    let label = NodeRef::default();
    let update_interval_cb = {
      let link = ctx.link().clone();
      move |e| link.send_message(Self::Message::AgentReady(e))
    };
    let _update_orderbook_interval = {
      let mut worker = Worker::bridge(Rc::new(update_interval_cb));
      Interval::new(10, move ||
        worker.send(WorkerInput {
          req: WorkerRequest::GetGRPC,
        })
      )
    };
    OrderWeb {
      spread,
      label,
      ask_table,
      bid_table,
      _update_orderbook_interval,
    }       
  }

  fn view(&self, _ctx: &Context<Self>) -> Html {   
    html! {
      <dev>
        <SpreadView id = {"spread_label"}  input_ref = {self.label.clone()} value = {"Spread: "}/>
        <SpreadView id = {"spread_value"}  input_ref = {self.spread.clone()} value = {"0"}/>
        <dev class="orderInfo">
        <OrderTableView id = {"Bid"} cells = {self.bid_table.clone()} headers = {
          vec![
            "price".to_owned(),
            "amount".to_owned(),
            "exchange".to_owned() 
          ]} />
        <OrderTableView id = {"Ask"} cells = {self.ask_table.clone()} headers = {
          vec![
            "price".to_owned(),
            "amount".to_owned(),
            "exchange".to_owned() 
          ]}/>
        </dev>
      </dev>
    }
  }
  
  fn changed(&mut self, _ctx: &Context<Self>, _props_old: &Self::Properties) -> bool {
    false
  }

  fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
    match msg {
      Self::Message::AgentReady(response) => {
         //log::info!("agent response: {:?}", response);
         if let Some(latest) = response.book.last() {
          let _ = self.update_table(&latest.asks, OrderBookSide::Ask);
          let _= self.update_table(&latest.bids, OrderBookSide::Bid);
          let _ = self.update_spread(latest.spread);
         }         
        true
    },
    _ => false,
  }
 }
}
