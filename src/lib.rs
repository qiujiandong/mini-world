use creep_manager::CreepMgr;
use std::cell::RefCell;
use wasm_bindgen::prelude::*;

mod creep_manager;
mod creep_target;
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

        creep_mgrs.push(CreepMgr::new("upgrader-0")); // work + carry + move
        creep_mgrs.push(CreepMgr::new("builder-5")); // work + carry + move
        creep_mgrs.push(CreepMgr::new("builder-4")); // work + carry + move
        creep_mgrs.push(CreepMgr::new("builder-3")); // work + carry + move
        creep_mgrs.push(CreepMgr::new("builder-2")); // work + carry + move
        creep_mgrs.push(CreepMgr::new("builder-1")); // work + carry + move
        creep_mgrs.push(CreepMgr::new("builder-0")); // work + carry + move

        // creep_mgrs.push(CreepMgr::new("carrier-0")); // carry + move
        // creep_mgrs.push(CreepMgr::new("miner-0")); // work + move
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
}
