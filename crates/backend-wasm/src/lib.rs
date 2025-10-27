mod utils;
pub use wasm_bindgen_rayon::init_thread_pool;

use std::sync::RwLock;

use shared::{
    OCDeclareArc, OCDeclareDiscoveryOptions, discover_behavior_constraints, get_activity_object_involvements, preprocess_ocel, process_mining::{
        import_ocel_json_from_slice, import_ocel_xml_slice, ocel::linked_ocel::IndexLinkedOCEL,
    }, reduction::reduce_oc_arcs
};
use wasm_bindgen::prelude::*;

static WASM_MEMORY_THINGY: RwLock<Option<IndexLinkedOCEL>> = RwLock::new(Option::None);

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
    let locel: IndexLinkedOCEL = preprocess_ocel(ocel);
    // unsafe {
    *WASM_MEMORY_THINGY.write().unwrap() = Some(locel);
    // }
}

#[wasm_bindgen]
pub fn load_ocel_xml(ocel_xml: &[u8]) -> usize {
    let ocel = import_ocel_xml_slice(ocel_xml);
    let num_objs: usize = ocel.objects.len();
    let locel: IndexLinkedOCEL = preprocess_ocel(ocel);
    // unsafe {
    *WASM_MEMORY_THINGY.write().unwrap() = Some(locel);
    num_objs
}

#[wasm_bindgen]
pub fn unload_ocel() {
    // unsafe {
    WASM_MEMORY_THINGY.write().unwrap().take();
    // }
}

#[wasm_bindgen]
pub fn get_edge_violation_percentage(edge_json: String) -> String {
    let locel_guard = WASM_MEMORY_THINGY.read().unwrap();
    if let Some(locel) = locel_guard.as_ref() {
        let edge: OCDeclareArc = serde_json::from_str(&edge_json).unwrap();
        let all_res = edge.get_for_all_evs_perf(locel);

        serde_json::to_string(&all_res).unwrap()
    } else {
        String::from("Failed")
    }
}

#[wasm_bindgen]
pub fn get_edge_violation_percentage_perf(edge_json: String) -> Result<f64, String> {
    let locel_guard = WASM_MEMORY_THINGY.read().unwrap();
    if let Some(locel) = locel_guard.as_ref() {
        let edge: OCDeclareArc = serde_json::from_str(&edge_json).unwrap();
        let viol_frac = edge.get_for_all_evs_perf(locel);

        Ok(viol_frac)
    } else {
        Err(String::from("Failed"))
    }
}

#[wasm_bindgen]
pub fn get_edge_as_template_text(edge_json: String) -> Result<String, String> {
    let edge: OCDeclareArc = serde_json::from_str(&edge_json).map_err(|e| e.to_string())?;
    Ok(edge.as_template_string())
}

#[wasm_bindgen]
pub fn get_all_edge_violation_percentage(edge_json: String) -> Result<Vec<String>, String> {
    let locel_guard = WASM_MEMORY_THINGY.read().unwrap();
    if let Some(locel) = locel_guard.as_ref() {
        let edge_json: Vec<OCDeclareArc> = serde_json::from_str(&edge_json).unwrap();
        let res = edge_json
            .iter()
            .map(|edge| {
                //    let edge: OCDeclareArc = serde_json::from_str(json).unwrap();
                let all_res = edge.get_for_all_evs_perf(locel);
                serde_json::to_string(&all_res).unwrap()
            })
            .collect();
        Ok(res)
        // let edge: OCDeclareArc = serde_json::from_str(&edge_json).unwrap();

        // return serde_json::to_string(&all_res).unwrap();
    } else {
        Err(String::from("Failed"))
    }
}

#[wasm_bindgen]
pub fn get_all_edge_violation_percentage_perf(edge_json: String) -> Result<Vec<f64>, String> {
    let locel_guard = WASM_MEMORY_THINGY.read().unwrap();
    if let Some(locel) = locel_guard.as_ref() {
        let edge_json: Vec<OCDeclareArc> = serde_json::from_str(&edge_json).unwrap();
        let res = edge_json
            .iter()
            .map(|edge| {
                //    let edge: OCDeclareArc = serde_json::from_str(json).unwrap();
                let viol_frac = edge.get_for_all_evs_perf(locel);
                viol_frac
            })
            .collect();
        Ok(res)
        // let edge: OCDeclareArc = serde_json::from_str(&edge_json).unwrap();

        // return serde_json::to_string(&all_res).unwrap();
    } else {
        Err(String::from("Failed"))
    }
}

#[wasm_bindgen]
pub fn get_ot_act_involvements() -> String {
    let locel_guard = WASM_MEMORY_THINGY.read().unwrap();
    if let Some(locel) = locel_guard.as_ref() {
        let ot_act_involvement = get_activity_object_involvements(locel);
        serde_json::to_string(&ot_act_involvement).unwrap()
    } else {
        String::from("Failed")
    }
}

#[wasm_bindgen]
pub fn discover_oc_declare_constraints(noise_thresh: f64) -> Result<String, String> {
    let locel_guard = WASM_MEMORY_THINGY.read().unwrap();
    if let Some(locel) = locel_guard.as_ref() {
        let mut options =  OCDeclareDiscoveryOptions::default();
        options.noise_threshold = noise_thresh;
        // options.counts_for_generation.1 = Some(20);
        options.counts_for_filter.1 = Some(5);
        let discovered_arcs = discover_behavior_constraints(locel,options);
        let reduced = reduce_oc_arcs(&discovered_arcs);
        Ok(serde_json::to_string(&reduced).unwrap())
    } else {
        Err(String::from("Failed"))
    }
}
