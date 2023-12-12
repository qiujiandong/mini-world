use creep_manager::CreepMgr;
use link_manager::*;
use std::cell::RefCell;
use wasm_bindgen::prelude::*;

mod creep_composition;
mod creep_manager;
mod creep_target;
mod link_manager;
mod logging;

thread_local! {
  static CREEP_ARRAY: RefCell<Vec<CreepMgr>> = RefCell::new(Vec::new());
}

// add wasm_bindgen to any function you would like to expose for call from js
#[wasm_bindgen]
pub fn setup() {
    logging::setup_logging(logging::Debug);

    CREEP_ARRAY.with(|creep_array| {
        let creep_mgrs = &mut creep_array.borrow_mut();

        creep_mgrs.push(CreepMgr::new("transfer-0"));
        creep_mgrs.push(CreepMgr::new("carrier-0"));
        creep_mgrs.push(CreepMgr::new("miner-1"));
        creep_mgrs.push(CreepMgr::new("miner-0"));
        creep_mgrs.push(CreepMgr::new("builder-1"));
        creep_mgrs.push(CreepMgr::new("builder-0"));
        creep_mgrs.push(CreepMgr::new("upgrader-0"));
    })
}

// to use a reserved name as a function name, use `js_name`:
#[wasm_bindgen(js_name = loop)]
pub fn game_loop() {
    CREEP_ARRAY.with(|creep_array| {
        let creep_mgrs = &mut creep_array.borrow_mut();
        for creep_mgr in creep_mgrs.iter_mut() {
            creep_mgr.run();
        }
    });
    link_tx_from_mine();
}
