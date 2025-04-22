use std::{collections::HashMap, fs::File, sync::Arc};

use process_mining::{ocel::linked_ocel::{IndexLinkedOCEL, LinkedOCELAccess}, petri_net::petri_net_struct::{ArcType, Place, PlaceID, TransitionID}, PetriNet};
use serde::{Deserialize, Serialize};

use crate::{OCDeclareArc, OCDeclareArcType, ObjectInvolvementCounts};

pub fn reduce_oc_arcs(arcs: Vec<OCDeclareArc>) -> Vec<OCDeclareArc> {
    let mut ret = arcs.clone();
    for a in &arcs {
        
            // ret.retain(|b| {
            //     let remove = a.from == b.to && a.to == b.from && a.arc_type!= b.arc_type  && b.label.is_dominated_by(&a.label) && !a.label.is_dominated_by(&b.label);
            //     !remove
            // });
            //         ret.retain(|b| {
            //     let remove = a.from == b.to && a.to == b.from && a.arc_type != b.arc_type && a.arc_type == OCDeclareArcType::EF  && b.label.is_dominated_by(&a.label);
            //     !remove
            // });
        
        for b in &arcs {
            // if a.from.as_str() != "place order" && a.to.as_str() != "confirm order" && b.to.as_str() != "create package" {
            //     continue;
            // }
            if a.from != a.to && b.from == a.to && a.from != b.to && a.arc_type == b.arc_type {
                // Remove a.from -> b.to (IF object involvement + arc type match)
                // let a_dominates_b = b.label.is_dominated_by(&a.label);
                // let b_dominates_a = a.label.is_dominated_by(&b.label);
                // if a.label == b.label {
                    ret.retain(|c| {
                        let remove = c.from == a.from
                        && c.to == b.to
                        && c.arc_type == a.arc_type
                        && (c.label.is_dominated_by(&a.label) && c.label.is_dominated_by(&b.label));
                        let is_strictly_dominated = c.label.any.iter().all(|any_label| {
                            let x = if c.arc_type == OCDeclareArcType::EF { !b.label.any.iter().any(|l| l == any_label) } else { !a.label.any.iter().any(|l| l == any_label) };
                            x
                         });
                        // if  c.to.as_str() == "send package" && c.from.as_str() == "payment reminder" && a.from.as_str() == "package delivered" && !is_strictly_dominated {
                        //     println!("[{}{}{}] NOT Removing {:?} because of {:?} and {:?}",c.from.as_str(),if c.arc_type == OCDeclareArcType::EF { "->"} else { "<-"},c.to.as_str(),c.label,a.label,b.label);
                        // }
                        !remove || !is_strictly_dominated
                    })
                // }
            }

        }
    }

    ret
}

pub fn oc_pn_prefilter(arcs: Vec<OCDeclareArc>, locel: &IndexLinkedOCEL, noise_thresh: f64, oi: &HashMap<String, HashMap<String, ObjectInvolvementCounts>>) -> Vec<OCDeclareArc> {
    //    Object Type, First Transition, multiple in?, Second transition, multiple out?
    let mut places: Vec<(String,String,bool,String,bool)> = Vec::new();
    for a in &arcs {
        for b in &arcs {
            if a.from == b.to && a.to == b.from && a.arc_type == OCDeclareArcType::EF && b.arc_type == OCDeclareArcType::EFREV {
                // Next, build compatible label
                // And test if compatible label holds also with c_min=c_max=1
                let common_label = a.label.intersect(&b.label);

                let new_arc_1 = OCDeclareArc {
                    from: a.from.clone(),
                    to: a.to.clone(),
                    arc_type: OCDeclareArcType::EF,
                    label: common_label.clone(),
                    counts: (Some(1),Some(1)),
                };
                let new_arc_2 = OCDeclareArc {
                    from: b.from.clone(),
                    to: b.to.clone(),
                    arc_type: OCDeclareArcType::EFREV,
                    label: common_label,
                    counts: (Some(1),Some(1)),
                };

                // if a.from.as_str() == "place order" && a.to.as_str() == "pick item" {
                //     println!("{}",new_arc_1.as_template_string());
                //     println!("{}",new_arc_2.as_template_string());
                // }
                // println!("Testing {}=>{} with label {:?}",new_arc_1.from.as_str(),new_arc_1.to.as_str(),new_arc_1.label);
                if new_arc_1.get_for_all_evs_perf_thresh(locel, noise_thresh) && new_arc_2.get_for_all_evs_perf_thresh(locel, noise_thresh) {
                    new_arc_1.label.all.iter().for_each(|oa| match oa {
                        crate::ObjectTypeAssociation::Simple { object_type }   => {
                            places.push((object_type.to_string(),a.from.0.to_string(),true,a.to.0.to_string(), true));
                        },
                        _ => {}
                    });
                    new_arc_1.label.each.iter().for_each(|oa| match oa {
                        crate::ObjectTypeAssociation::Simple { object_type }   => {
                            let first_multiple = oi.get(a.from.as_str()).unwrap().get(object_type).unwrap().max > 1;
                            let second_multiple = oi.get(a.to.as_str()).unwrap().get(object_type).unwrap().max > 1;
                            places.push((object_type.to_string(),a.from.0.to_string(),first_multiple,a.to.0.to_string(), second_multiple));
                        },
                        _ => {}
                    });
                    println!("PASSED: {}",new_arc_1.as_template_string());
                }
                
            }
        }
    }
    println!("Places: {:#?}",places);
    println!("#Places: {}",places.len());
    let mut net = PetriNet::new();
    let trans_map: HashMap<&str,TransitionID> = locel.get_ev_types().map(|et| (et,net.add_transition(Some(et.to_string()), None))).collect();
    let mut place_object_type = HashMap::new();
    let mut place_in_out_mult = HashMap::new();
    for (ot,from,from_mult,to,to_mult) in places {
        let from = trans_map.get(from.as_str()).unwrap();
        let to = trans_map.get(to.as_str()).unwrap();
        let place_id = net.add_place(None);
        place_object_type.insert(place_id.clone(), ot.to_string());
        place_in_out_mult.insert(place_id, (from_mult,to_mult));
        net.add_arc(ArcType::transition_to_place(*from, place_id), Some(if from_mult  {10} else { 1 }));
        net.add_arc(ArcType::place_to_transition(place_id,*to), Some(if to_mult  {10} else { 1 }));
    }
    #[derive(Debug,Clone, Deserialize, Serialize)]
    struct OCPetriNet {
        petri_net: PetriNet,
        place_object_type: HashMap<PlaceID,String>,
        place_in_out_mult: HashMap<PlaceID,(bool,bool)>
    }
    net.export_pnml("./oc-pn.pnml").unwrap();
    let oc_pn = OCPetriNet {
        petri_net: net,
        place_object_type,
        place_in_out_mult,
    };
    serde_json::to_writer_pretty(File::create("./logistics-oc-pn.json").unwrap(), &oc_pn).unwrap();
    arcs
}

#[cfg(test)]
mod test {
    use process_mining::{import_ocel_json_from_path, import_ocel_xml_file, ocel::linked_ocel::IndexLinkedOCEL};

    use crate::discovery::discover;


    #[test]
    fn test_reduction() {
        // let ocel = import_ocel_json_from_path("/home/aarkue/dow/hpc-ocel-2025-04-01.json").unwrap();
        // let ocel = import_ocel_json_from_path("/home/aarkue/dow/ocel/order-management.json").unwrap();
        // let ocel = import_ocel_json_from_path("/home/aarkue/dow/pm4Bundestag_20250406_periode_20_including_promulgation_proclamation.json").unwrap();
        // let ocel = import_ocel_json_from_path("/home/aarkue/dow/ocel/ocel2-p2p.json").unwrap();
        let ocel = import_ocel_json_from_path("/home/aarkue/dow/ocel/ContainerLogistics.json").unwrap();
        // let ocel = import_ocel_json_from_path("/home/aarkue/dow/ocel/age_of_empires_ocel2_10_match_filter.json").unwrap();
        // let ocel = import_ocel_json_from_path("/home/aarkue/dow/ocel/bpic2017-o2o-workflow-qualifier-index-no-ev-attrs-sm.json").unwrap();
        // let ocel = import_ocel_xml_file("/home/aarkue/dow/ocel/lrm/01_o2c(2).xml");
        // let ocel = import_ocel_xml_file("/home/aarkue/dow/ocel/socel2_hinge.xml");
        // let ocel = import_ocel_xml_file("/home/aarkue/dow/ocel/age_of_empires_ocel2.xml");
        // let ocel = import_ocel_xml_file("/home/aarkue/dow/ocel_v3-fixed.xml");
        let locel = IndexLinkedOCEL::from(ocel);
        let res = discover(&locel, 0.2, crate::discovery::O2OMode::None);
        println!("Discovered {} constraints",res.len());
        // let results_file =
        //     std::fs::File::create(format!("reduced-order-0.2noise.json")).unwrap();
        // serde_json::to_writer_pretty(results_file, &res).unwrap();
    }
}