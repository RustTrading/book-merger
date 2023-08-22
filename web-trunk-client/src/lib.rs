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
 }

 fn update_spread(spread: f64) ->  Result<(), JsValue> {
  let document = web_sys::window().unwrap().document().unwrap();
  let spread_field = document.get_element_by_id("spread_value").unwrap();
  spread_field.first_child().unwrap().set_text_content(Some(&spread.to_string()));
  Ok(())
 }

 fn update_table(table_id: &str, levels: &Vec<Level>) -> Result<(), JsValue> {
  let document = web_sys::window().unwrap().document().unwrap();
  let table = document.get_element_by_id(table_id).unwrap();
  
  let tbody = table.last_element_child().unwrap();
  let rows = tbody.children();
  
  //log::info!("rows: {}", rows.length());

  let mut row_idx = 0;

  let level_num = levels.len() as u32;
  let current_table_len = tbody.child_element_count();
    
  //log::info!("level_num: {}, current_child: {}", level_num, current_table_len);

  if level_num < current_table_len {
    let extra_children = tbody.child_nodes();
    for idx in level_num..current_table_len {
      tbody.remove_child(&extra_children.get(idx).unwrap())?;
    }
  }
  for level in levels {
    let properties = vec![level.price.to_string(), level.amount.to_string(), level.exchange.clone()]; 
    if row_idx < current_table_len {
      if let Some(row) = rows.get_with_index(row_idx) {
        let items = row.children();
        for idx in 0..items.length() {
          let td = items.get_with_index(idx).unwrap();
          td.set_text_content(Some(&properties[idx as usize]));
        }
        row_idx +=1;
        continue;
      }
    } 
    let row = document.create_element("tr").unwrap();
    for data in &properties {
      let item = document.create_element("td").unwrap();
      let value= document.create_text_node(data);
      item.append_child(&value)?;
      row.append_child(&item)?;
    }
    tbody.append_child(&row)?;
  }
  Ok(())
 }

  impl Component for OrderWeb {

  type Message = Msg;
  type Properties = ();

  fn create(ctx: &Context<Self>) -> Self {
    log::info!("Creating Yew-Plotters View");
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
      _update_orderbook_interval,
    }       
  }

  fn view(&self, _ctx: &Context<Self>) -> Html {   
    
    html! {
      <dev>
        <SpreadView id = {"spread_label"}  value = {"Spread: "}/>
        <SpreadView id = {"spread_value"}  value = {"0"}/>
        <dev class="orderInfo">
        <OrderTableView id = {"Bid"} headers = {
          vec![
            "price".to_owned(),
            "amount".to_owned(),
            "exchange".to_owned() 
          ]} />
        <OrderTableView id = {"Ask"} headers = {
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
          let _ = update_table("Ask", &latest.asks);
          let _= update_table("Bid", &latest.bids);
          let _ = update_spread(latest.spread);
         }         
        true
    },
    _ => false,
  }
 }
}
