
use yew::virtual_dom::{VTag, VText};
use yew::prelude::*;

#[derive(Clone, PartialEq, Properties)]
pub struct OrderTableProps {
  pub id: String,
  pub headers: Vec<String>,
}

#[derive(Clone, PartialEq, Properties)]
pub struct SpreadViewProps {
  pub id: String,
  pub value: String,
}

#[function_component(SpreadView)]
pub fn spread_view(props: &SpreadViewProps) -> Html {
  let SpreadViewProps {
    id,
    value
  } = props;
   html! {
    <label id = {id.to_owned()}>
    { value }
    </label>
   }
}

#[function_component(OrderTableView)]
pub fn order_table(props: &OrderTableProps) -> Html {
  let OrderTableProps {
    id,
    headers, 
  } = props;
  let mut table= VTag::new("table");
  table.add_attribute("id", id.to_owned());
  let mut thead= VTag::new("thead");
  let tbody= VTag::new("tbody");
  let mut tr = VTag::new("tr");
  let mut th = VTag::new("th");
  th.add_attribute("colspan", "3");
  let thtext = VText::new(id.to_owned());
  th.add_child(thtext.into());
  tr.add_child(th.into());
  thead.add_child(tr.into());
  let mut vtr= VTag::new("tr");
  for header in headers {
    let mut subheader= VTag::new("th");
    let vtext = VText::new(header.clone());
    subheader.add_child(vtext.into());
    vtr.add_child(subheader.into());
  }
  thead.add_child(vtr.into());
  table.add_child(thead.into());
  table.add_child(tbody.into());
  table.into() 
}
