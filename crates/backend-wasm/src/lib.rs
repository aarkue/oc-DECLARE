mod utils;

use std::sync::RwLock;

use shared::{
    process_mining::{
        import_ocel_json_from_slice,
        ocel::linked_ocel::OwnedLinkedOcel,
    },
    OCDeclareArc,
};
use wasm_bindgen::prelude::*;

static WASM_MEMORY_THINGY: RwLock<Option<OwnedLinkedOcel>> = RwLock::new(Option::None);

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, backend-wasm!");
}

#[wasm_bindgen]
pub fn load_ocel(ocel_json: String) {
    let ocel = import_ocel_json_from_slice(ocel_json.as_bytes()).unwrap();
    let locel: OwnedLinkedOcel = ocel.into();
    // unsafe {
        *WASM_MEMORY_THINGY.write().unwrap() = Some(locel);
    // }
}

#[wasm_bindgen]
pub fn unload_ocel() {
    // unsafe {
        WASM_MEMORY_THINGY.write().unwrap().take();
    // }
}

#[wasm_bindgen]
pub fn get_edge_violation_percentage(edge_json: String) -> f32 {
    // let locel: OwnedLinkedOcel =  unsafe {
    //     *Box::from_raw(ocel_pointer as *mut OwnedLinkedOcel)
    // };

    let locel_guard =
    //  unsafe {
         WASM_MEMORY_THINGY.read().unwrap();
        //  };
    if let Some(locel) = locel_guard.as_ref() {
        let edge: OCDeclareArc = serde_json::from_str(&edge_json).unwrap();
        let all_res = edge.get_for_all_evs(&locel.linked_ocel);

        // let count: usize = all_res.iter().flatten().sum();

        let at_least_one: usize = all_res.iter().flatten().filter(|r| **r >= 1).count();

        // println!("Len: {}", all_res.len());
        // println!("Count: {count}");
        // println!("At least one: {}", at_least_one);
        // println!(
        //     "Violation percentage: {:.2}%",
        //     1)
        // return ocel.objects.len()
        100.0 * (1.0 - (at_least_one as f32 / all_res.len() as f32))
    } else {
        -1.0
    }
}
