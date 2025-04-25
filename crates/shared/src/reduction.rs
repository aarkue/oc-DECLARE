use std::{collections::HashMap, fs::File, hash::Hash, sync::Arc};

use process_mining::{
    ocel::linked_ocel::{IndexLinkedOCEL, LinkedOCELAccess},
    petri_net::petri_net_struct::{ArcType, Place, PlaceID, TransitionID},
    PetriNet,
};
use serde::{Deserialize, Serialize};

use crate::{
    OCDeclareArc, OCDeclareArcType, ObjectInvolvementCounts, EXIT_EVENT_PREFIX, INIT_EVENT_PREFIX,
};

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
                        let x = if c.arc_type == OCDeclareArcType::EF {
                            !b.label.any.iter().any(|l| l == any_label)
                        } else {
                            !a.label.any.iter().any(|l| l == any_label)
                        };
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

pub fn oc_pn_prefilter(
    name: &str,
    arcs: Vec<OCDeclareArc>,
    locel: &IndexLinkedOCEL,
    noise_thresh: f64,
    oi: &HashMap<String, HashMap<String, ObjectInvolvementCounts>>,
) {
    let mut arc_map: HashMap<String, Vec<OCDeclareArc>> = HashMap::new();
    arcs.into_iter()
        .for_each(|arc| arc_map.entry(arc.to.0.clone()).or_default().push(arc));
    //    Object Type, First Transition, multiple in?, Second transition, multiple out?
    let mut places: Vec<Vec<(String, Vec<(String, bool)>, Vec<(String, bool)>)>> = Vec::new();
    for a in arc_map.values().flatten() {
        for b in arc_map.get(&a.from.0).iter().flat_map(|x| x.iter()) {
            if a.from == b.to
                && a.to == b.from
                && a.arc_type == OCDeclareArcType::EF
                && b.arc_type == OCDeclareArcType::EFREV
            {
                if a.from.as_str().starts_with(INIT_EVENT_PREFIX)
                    && a.to.as_str().starts_with(EXIT_EVENT_PREFIX)
                {
                    continue;
                }
                // Next, build compatible label
                // And test if compatible label holds also with c_min=c_max=1
                let common_label = a.label.intersect(&b.label);

                let new_arc_1 = OCDeclareArc {
                    from: a.from.clone(),
                    to: a.to.clone(),
                    // arc_type: OCDeclareArcType::EF,
                    arc_type: OCDeclareArcType::CHAINEF,
                    label: common_label.clone(),
                    counts: (Some(1), Some(1)),
                };
                let new_arc_2 = OCDeclareArc {
                    from: b.from.clone(),
                    to: b.to.clone(),
                    // arc_type: OCDeclareArcType::EFREV,
                    arc_type: OCDeclareArcType::CHAINEFREV,
                    label: common_label.clone(),
                    counts: (Some(1), Some(1)),
                };

                // if a.from.as_str() == "place order" && a.to.as_str() == "pick item" {
                //     println!("{}",new_arc_1.as_template_string());
                //     println!("{}",new_arc_2.as_template_string());
                // }
                // println!("Testing {}=>{} with label {:?}",new_arc_1.from.as_str(),new_arc_1.to.as_str(),new_arc_1.label);
                if new_arc_1.get_for_all_evs_perf_thresh(locel, noise_thresh)
                    && new_arc_2.get_for_all_evs_perf_thresh(locel, noise_thresh)
                {
                    let mut new_places = Vec::new();
                    new_arc_1.label.all.iter().for_each(|oa| match oa {
                        crate::ObjectTypeAssociation::Simple { object_type } => {
                            new_places.push((
                                object_type.to_string(),
                                vec![(a.from.0.to_string(), true)],
                                vec![(a.to.0.to_string(), true)],
                            ));
                        }
                        _ => {}
                    });
                    new_arc_1.label.each.iter().for_each(|oa| match oa {
                        crate::ObjectTypeAssociation::Simple { object_type } => {
                            let first_multiple = oi
                                .get(a.from.as_str())
                                .unwrap()
                                .get(object_type)
                                .unwrap()
                                .max
                                > 1;
                            let second_multiple =
                                oi.get(a.to.as_str()).unwrap().get(object_type).unwrap().max > 1;
                            new_places.push((
                                object_type.to_string(),
                                vec![(a.from.0.to_string(), first_multiple)],
                                vec![(a.to.0.to_string(), second_multiple)],
                            ));
                        }
                        _ => {}
                    });
                    println!("PASSED: {}", new_arc_1.as_template_string());
                    // Test for optional C
                    for c1 in arc_map.get(&a.from.0).iter().flat_map(|x| x.iter()) {
                        for c2 in arc_map.get(&a.to.0).iter().flat_map(|x| x.iter()) {
                            if c1.arc_type == OCDeclareArcType::EFREV
                                && c2.arc_type == OCDeclareArcType::EF
                                && c1.from.as_str() == c2.from.as_str()
                            {
                                let d = arc_map
                                    .get(c1.from.as_str())
                                    .iter()
                                    .flat_map(|a| a.iter())
                                    .find(|arc| arc.from == a.from);
                                if d.is_some() {
                                    continue;
                                }
                                let c_common_label = c1.label.intersect(&c2.label);
                                if c_common_label.is_dominated_by(&common_label) {
                                    // C is an optional loop candidate :)
                                    for (ot, in_arcs, out_arcs) in &mut new_places {
                                        let all = c_common_label.all.iter().find(|l| match l {
                                            crate::ObjectTypeAssociation::Simple {
                                                object_type,
                                            } => object_type == ot,
                                            _ => false,
                                        });
                                        if let Some(x) = all {
                                            in_arcs.push((c1.from.0.clone(), true));
                                            out_arcs.push((c1.from.0.clone(), true));
                                        }

                                        let each = c_common_label.each.iter().find(|l| match l {
                                            crate::ObjectTypeAssociation::Simple {
                                                object_type,
                                            } => object_type == ot,
                                            _ => false,
                                        });
                                        if let Some(x) = each {
                                            // TODO: Correct!
                                            in_arcs.push((c1.from.0.clone(), false));
                                            out_arcs.push((c1.from.0.clone(), false));
                                        }
                                    }
                                    println!(
                                        "{} is a candidate for {}\n{:?} and {:?}",
                                        c1.from.as_str(),
                                        new_arc_1.as_template_string(),
                                        c1,
                                        c2
                                    );
                                }
                            }
                        }
                    }
                    places.push(new_places);
                }
            }
        }
    }
    // println!("Places: {:#?}", places);
    println!("#Places: {}", places.iter().map(|p| p.len()).sum::<usize>());
    let mut net = PetriNet::new();
    let trans_map: HashMap<&str, TransitionID> = locel
        .get_ev_types()
        .map(|et| (et, net.add_transition(Some(et.to_string()), None)))
        .collect();
    let mut place_object_type = HashMap::new();
    let mut place_in_out_mult: HashMap<
        PlaceID,
        (HashMap<TransitionID, bool>, HashMap<TransitionID, bool>),
    > = HashMap::new();
    let mut place_groups: HashMap<PlaceID, usize> = HashMap::new();
    for (group_index, place_group) in places.iter().enumerate() {
        for (ot, froms, tos) in place_group {
            let place_id = net.add_place(None);
            place_groups.insert(place_id.clone(), group_index);
            for (from, from_multi) in froms {
                let from = trans_map.get(from.as_str()).unwrap();
                net.add_arc(
                    ArcType::transition_to_place(*from, place_id),
                    Some(if *from_multi { 10 } else { 1 }),
                );
            }
            for (to, to_multi) in tos {
                let to = trans_map.get(to.as_str()).unwrap();
                net.add_arc(
                    ArcType::place_to_transition(place_id, *to),
                    Some(if *to_multi { 10 } else { 1 }),
                );
            }
            place_object_type.insert(place_id.clone(), ot.to_string());
            place_in_out_mult.insert(
                place_id,
                (
                    froms
                        .iter()
                        .map(|(from, multi)| {
                            (trans_map.get(from.as_str()).unwrap().clone(), *multi)
                        })
                        .collect(),
                    tos.iter()
                        .map(|(to, multi)| (trans_map.get(to.as_str()).unwrap().clone(), *multi))
                        .collect(),
                ),
            );
        }
    }
    #[derive(Debug, Clone, Deserialize, Serialize)]
    struct OCPetriNet {
        petri_net: PetriNet,
        place_object_type: HashMap<PlaceID, String>,
        // place_in_out_mult: HashMap<PlaceID, (bool, bool)>,
        place_in_out_mult:
            HashMap<PlaceID, (HashMap<TransitionID, bool>, HashMap<TransitionID, bool>)>,
        place_groups: HashMap<PlaceID, usize>,
    }
    // net.export_pnml("./oc-pn-logistics.pnml").unwrap();
    let oc_pn = OCPetriNet {
        petri_net: net,
        place_object_type,
        place_in_out_mult,
        place_groups,
    };
    serde_json::to_writer_pretty(
        File::create(format!("./oc-pn-chain-{}.json", name)).unwrap(),
        &oc_pn,
    )
    .unwrap();
}

#[cfg(test)]
mod test {
    use process_mining::{
        import_ocel_json_from_path, import_ocel_xml_file, ocel::linked_ocel::IndexLinkedOCEL,
    };

    use crate::{
        discovery::discover, get_activity_object_involvements, preprocess_ocel,
        reduction::oc_pn_prefilter,
    };

    #[test]
    fn test_reduction() {
        // let ocel = import_ocel_json_from_path("/home/aarkue/dow/hpc-ocel-2025-04-01.json").unwrap();
        // let ocel =  import_ocel_json_from_path("/home/aarkue/dow/ocel/order-management.json").unwrap();
        // let ocel = import_ocel_json_from_path("/home/aarkue/dow/pm4Bundestag_20250406_periode_20_including_promulgation_proclamation.json").unwrap();
        // let ocel: process_mining::OCEL = import_ocel_json_from_path("/home/aarkue/dow/ocel/ocel2-p2p.json").unwrap();
        // let ocel = import_ocel_json_from_path("/home/aarkue/dow/ocel/ContainerLogistics.json").unwrap();
        // let ocel = import_ocel_json_from_path("/home/aarkue/dow/ocel/age_of_empires_ocel2_10_match_filter.json").unwrap();
        // let ocel = import_ocel_json_from_path("/home/aarkue/dow/ocel/bpic2017-o2o-workflow-qualifier-index-no-ev-attrs-sm.json").unwrap();
        // let ocel = import_ocel_xml_file("/home/aarkue/dow/ocel/lrm/01_o2c(2).xml");
        let ocel = import_ocel_xml_file("/home/aarkue/dow/ocel/socel2_hinge.xml");
        // let ocel = import_ocel_xml_file("/home/aarkue/dow/ocel/age_of_empires_ocel2.xml");
        // let ocel = import_ocel_xml_file("/home/aarkue/dow/ocel_v3-fixed.xml");
        let locel = preprocess_ocel(ocel);
        // let locel = IndexLinkedOCEL::from(ocel);
        let noise_thresh = 0.1;
        let res = discover(&locel, noise_thresh, crate::discovery::O2OMode::None);
        let act_ob_inv = get_activity_object_involvements(&locel);
        let new_ret = oc_pn_prefilter(
            &format!("pre-processed-hinge-{noise_thresh}"),
            res,
            &locel,
            noise_thresh,
            &act_ob_inv,
        );
        // println!("Discovered {} constraints", res.len());
        // let results_file =
        //     std::fs::File::create(format!("reduced-order-0.2noise.json")).unwrap();
        // serde_json::to_writer_pretty(results_file, &res).unwrap();
    }
}
