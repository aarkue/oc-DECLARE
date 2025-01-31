mod utils;
pub use wasm_bindgen_rayon::init_thread_pool;

use std::sync::RwLock;


use shared::{
    process_mining::{
        import_ocel_json_from_slice, import_ocel_xml_slice, ocel::linked_ocel::OwnedLinkedOcel
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
pub fn load_ocel_json(ocel_json: &[u8]) {
    let ocel = import_ocel_json_from_slice(ocel_json).unwrap();
    let locel: OwnedLinkedOcel = ocel.into();
    // unsafe {
        *WASM_MEMORY_THINGY.write().unwrap() = Some(locel);
    // }
}



#[wasm_bindgen]
pub fn load_ocel_xml(ocel_xml: &[u8]) {
    let ocel = import_ocel_xml_slice(ocel_xml);
    let locel: OwnedLinkedOcel = ocel.into();
    // unsafe {
        *WASM_MEMORY_THINGY.write().unwrap() = Some(locel);
    }



#[wasm_bindgen]
pub fn unload_ocel() {
    // unsafe {
        WASM_MEMORY_THINGY.write().unwrap().take();
    // }
}

#[wasm_bindgen]
pub fn get_edge_violation_percentage(edge_json: String) -> String {
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
        // let all_res_flat: Vec<_> = all_res.into_iter().flatten().collect();

        // let count: usize = all_res.iter().flatten().sum();

        // let at_least_one: usize = all_res_flat.iter().filter(|r| **r >= 1).count();

        // println!("Len: {}", all_res.len());
        // println!("Count: {count}");
        // println!("At least one: {}", at_least_one);
        // println!(
        //     "Violation percentage: {:.2}%",
        //     1)
        // return ocel.objects.len()
        // alert(&format!("{}",at_least_one));
        // alert(&format!("{}",all_res_flat.len()));
        return serde_json::to_string(&all_res).unwrap();
        // 100.0 * (1.0 - (at_least_one as f64 / all_res_flat.len() as f64))
    } else {
        String::from("Failed")
    }
}
