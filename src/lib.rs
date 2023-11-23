use js_sys::JsString;
use log::*;
use screeps::raw_memory;
use screeps::*;
use serde::{Deserialize, Serialize};
use serde_json;
use wasm_bindgen::prelude::*;

mod logging;

#[derive(Serialize, Deserialize, Debug, Default)]
struct Info {
  name: String,
  age: u32,
}

// add wasm_bindgen to any function you would like to expose for call from js
#[wasm_bindgen]
pub fn setup() {
  logging::setup_logging(logging::Debug);
}

// to use a reserved name as a function name, use `js_name`:
#[wasm_bindgen(js_name = loop)]
pub fn game_loop() {
  let active_segs: [u8; 4] = [0, 1, 2, 3];
  raw_memory::set_active_segments(&active_segs);

  let last_info = raw_memory::get().as_string().unwrap();
  let info_old: Info = serde_json::from_str(&last_info).unwrap_or_default();
  debug!("last info: {:?}", info_old);

  let info = Info {
    name: String::from("hello"),
    age: game::time(),
  };

  let serialized = JsString::from(serde_json::to_string(&info).unwrap());
  raw_memory::set(&serialized);
}
