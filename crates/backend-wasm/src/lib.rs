mod utils;

use shared::{process_mining::{import_ocel_json_from_slice, ocel::{self, linked_ocel::OwnedLinkedOcel}}, OCDeclareArc};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, backend-wasm!");
}


#[wasm_bindgen]
pub fn get_edge_violation_percentage(ocel_json: String, edge_json: String) -> f32 {
    let ocel = import_ocel_json_from_slice(ocel_json.as_bytes()).unwrap();
    let edge: OCDeclareArc = serde_json::from_str(&edge_json).unwrap();
    let locel: OwnedLinkedOcel = ocel.into();
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
}