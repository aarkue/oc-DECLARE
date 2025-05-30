use std::{
    collections::{HashMap, HashSet},
    fs::File,
    time::Instant,
};

use indicatif::ParallelProgressIterator;
use itertools::Itertools;
use process_mining::{
    import_ocel_json_from_path, import_ocel_xml_file,
    ocel::linked_ocel::{IndexLinkedOCEL, LinkedOCELAccess},
    petri_net::petri_net_struct::{ArcType, PlaceID, TransitionID},
    PetriNet,
};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    discovery::get_direct_or_indirect_object_involvements, get_activity_object_involvements,
    get_object_to_object_involvements, get_rev_object_to_object_involvements, perf,
    preprocess_ocel, OCDeclareArc, OCDeclareArcLabel, OCDeclareArcType, OCDeclareNode,
    ObjectInvolvementCounts, ObjectTypeAssociation, EXIT_EVENT_PREFIX, INIT_EVENT_PREFIX,
};

pub fn mine_associations(
    locel: &IndexLinkedOCEL,
    noise_thresh: f64,
) -> Vec<(String, String, OCDeclareArcLabel)> {
    let mut ret = Vec::new();
    let act_ob_inv: HashMap<String, HashMap<String, ObjectInvolvementCounts>> =
        get_activity_object_involvements(locel);
    let ob_ob_inv: HashMap<String, HashMap<String, ObjectInvolvementCounts>> =
        get_object_to_object_involvements(locel);
    let ob_ob_rev_inv = get_rev_object_to_object_involvements(locel);
    // Third type of discovery: Eventually-follows
    let direction = OCDeclareArcType::ASS;
    let any_counts = (Some(1), None);
    let other_counts = (Some(1), Some(20));
    ret.par_extend(
        locel
            .events_per_type
            .keys()
            .cartesian_product(locel.events_per_type.keys())
            .par_bridge()
            .progress_count(locel.events_per_type.len() as u64 * locel.events_per_type.len() as u64)
            // .filter(|(act1, act2)| {
            //     if act1.starts_with(INIT_EVENT_PREFIX)
            //         || act1.starts_with(EXIT_EVENT_PREFIX)
            //         || act2.starts_with(INIT_EVENT_PREFIX)
            //         || act2.starts_with(EXIT_EVENT_PREFIX)
            //     {
            //         return false;
            //     }
            //     true
            // })
            .flat_map(|(act1, act2)| {
                let mut act_arcs = Vec::new();
                let obj_invs = get_direct_or_indirect_object_involvements(
                    act1,
                    act2,
                    &act_ob_inv,
                    &ob_ob_inv,
                    &ob_ob_rev_inv,
                    crate::discovery::O2OMode::None,
                );
                // let obj_invs_cloned = obj_invs.clone();
                for (ot, is_multiple) in obj_invs {
                    // ANY?
                    let each_label = OCDeclareArcLabel {
                        each: vec![ot],
                        any: vec![],
                        all: vec![],
                    };
                    let sat = perf::get_for_all_evs_perf_thresh(
                        act1,
                        act2,
                        &each_label,
                        &direction,
                        &any_counts,
                        locel,
                        noise_thresh,
                    );
                    if sat {
                        // It IS a viable candidate!
                        // Also test All:
                        if is_multiple {
                            let all_label = OCDeclareArcLabel {
                                each: vec![],
                                any: vec![],
                                all: each_label.each.clone(),
                            };
                            let sat = perf::get_for_all_evs_perf_thresh(
                                act1,
                                act2,
                                &all_label,
                                &direction,
                                &other_counts,
                                locel,
                                noise_thresh,
                            );
                            if sat {
                                // All is also valid!
                                act_arcs.push(all_label);
                                act_arcs.push(each_label);
                                // act_arcs.push(any_label);
                            } else {
                                act_arcs.push(each_label);
                                // act_arcs.push(any_label);
                            }
                        } else {
                            act_arcs.push(OCDeclareArcLabel {
                                each: vec![],
                                any: vec![],
                                all: each_label.each,
                            });
                        }
                    }
                }
                let mut changed = true;
                let mut old: HashSet<_> = act_arcs.iter().cloned().collect();
                let mut iteration = 1;
                while changed {
                    let x = 0..act_arcs.len();
                    let new_res: HashSet<_> = x
                        .flat_map(|arc1_i| {
                            ((arc1_i + 1)..act_arcs.len()).map(move |arc2_i| (arc1_i, arc2_i))
                        })
                        .par_bridge()
                        .filter_map(|(arc1_i, arc2_i)| {
                            let arc1 = &act_arcs[arc1_i];
                            let arc2 = &act_arcs[arc2_i];
                            if arc1.is_dominated_by(arc2) || arc2.is_dominated_by(arc1) {
                                return None;
                            }
                            if !arc1.each.is_empty() || !arc2.each.is_empty() {
                                // In this approach, we do not combine multiple each/all constructs.
                                // However, ALL can have multiple object types
                                return None;
                            }
                            let new_arc_label = arc1.combine(arc2);
                            let new_n = new_arc_label.all.len()
                                + new_arc_label.any.len()
                                + new_arc_label.each.len();
                            if new_n != iteration + 1 {
                                return None;
                            }
                            let sat = perf::get_for_all_evs_perf_thresh(
                                act1,
                                act2,
                                &new_arc_label,
                                &direction,
                                &other_counts,
                                locel,
                                noise_thresh,
                            );
                            if sat {
                                Some(new_arc_label)
                            } else {
                                None
                            }
                        })
                        .collect();

                    changed = !new_res.is_empty();
                    old.retain(|a: &OCDeclareArcLabel| {
                        !new_res.iter().any(|a2| a != a2 && a.is_dominated_by(a2))
                    });
                    old.extend(new_res.clone().into_iter());
                    act_arcs = new_res
                        .iter()
                        .filter(|a| !new_res.iter().any(|a2| *a != a2 && a.is_dominated_by(a2)))
                        .cloned()
                        .collect();
                    iteration += 1;
                }
                let v = old
                    .clone()
                    .into_par_iter()
                    .filter(move |arc1| {
                        !old.iter()
                            .any(|arc2| *arc1 != *arc2 && arc1.is_dominated_by(arc2))
                    })
                    .map(move |label| (act1.clone(), act2.clone(), label));
                v
            }),
    );
    ret
}

pub fn incoporate_control_flow_and_ensure_fitness(
    locel: &IndexLinkedOCEL,
    noise_thresh: f64,
    candidates: Vec<(String, String, OCDeclareArcLabel)>,
) -> Vec<(String, String, OCDeclareArcLabel)> {
    let mut ret: Vec<(String, String, OCDeclareArcLabel)> = Vec::default();
    for (a1, b1, l1) in &candidates {
        for (a2, b2, l2) in &candidates {
            if a1 == b2 && b1 == a2 {
                let common_label = l1.intersect(&l2);
                if !common_label.each.is_empty() || !common_label.all.is_empty() {
                    if perf::get_for_all_evs_perf_thresh(
                        &a1,
                        &b1,
                        &common_label,
                        &OCDeclareArcType::EF,
                        &(Some(1), None),
                        locel,
                        noise_thresh,
                    ) && perf::get_for_all_evs_perf_thresh(
                        &a2,
                        &b2,
                        &common_label,
                        &OCDeclareArcType::EFREV,
                        &(Some(1), None),
                        locel,
                        noise_thresh,
                    ) {
                        // Candidate passes as EF/EP conditions hold!
                        // Next step: Check for fitness
                        // For that we use alternating constructs
                        if perf::get_for_all_evs_perf_thresh(
                            &a1,
                            &b1,
                            &common_label,
                            &OCDeclareArcType::ALTEF,
                            &(Some(1), Some(1)),
                            locel,
                            noise_thresh,
                        ) && perf::get_for_all_evs_perf_thresh(
                            &a2,
                            &b2,
                            &common_label,
                            &OCDeclareArcType::ALTEFREV,
                            &(Some(1), Some(1)),
                            locel,
                            noise_thresh,
                        ) {
                            ret.push((a1.clone(), b1.clone(), common_label))
                        } else {
                            // println!("Filtered out by fitness check: {a1}->{b1} for {:?}",common_label.as_template_string());
                        }
                    }
                }
            }
        }
    }
    ret
}

pub fn perform_transitive_reduction(
    candidates: Vec<(String, String, OCDeclareArcLabel)>,
) -> Vec<(String, String, OCDeclareArcLabel)> {
    let mut ret: Vec<(String, String, OCDeclareArcLabel)> = candidates.clone();
    for (a1, b1, l1) in &candidates {
        for (a2, b2, l2) in &candidates {
            if b1 == a2 {
                // So we have a1 -l1> b1 -l2> b2
                // Remove all a1 -l3> b2, where l3 <= l1  and l3 <= l2
                ret.retain(|(a3, b3, l3)| {
                    let remove = a3 == a1
                        && b3 == b2
                        && (l3.is_dominated_by(&l1) && l3.is_dominated_by(&l2));
                    !remove
                })
            }
        }
    }

    ret
}

fn add_optional_steps(
    locel: &IndexLinkedOCEL,
    noise_thresh: f64,
    candidates: Vec<(String, String, OCDeclareArcLabel)>,
) -> Vec<(String, String, OCDeclareArcLabel, Vec<String>)> {
    candidates
        .into_par_iter()
        .map(|(a1, b1, l1)| {
            let optional_acts = locel
                .get_ev_types()
                .filter(|op_act| {
                    perf::get_for_all_evs_perf_thresh(
                        &op_act,
                        &a1,
                        &l1,
                        &OCDeclareArcType::EFREV,
                        &(Some(1), None),
                        locel,
                        noise_thresh,
                    ) && perf::get_for_all_evs_perf_thresh(
                        &op_act,
                        &b1,
                        &l1,
                        &OCDeclareArcType::EF,
                        &(Some(1), None),
                        locel,
                        noise_thresh,
                    ) && !perf::get_for_all_evs_perf_thresh(
                        &a1,
                        &op_act,
                        &l1,
                        &OCDeclareArcType::EF,
                        &(Some(1), None),
                        locel,
                        noise_thresh,
                    )
                })
                .map(|op_act| op_act.to_string())
                .collect();
            (a1, b1, l1, optional_acts)
        })
        .collect()
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct OCPetriNet {
    petri_net: PetriNet,
    place_object_type: HashMap<PlaceID, String>,
    // place_in_out_mult: HashMap<PlaceID, (bool, bool)>,
    place_in_out_mult: HashMap<PlaceID, (HashMap<TransitionID, bool>, HashMap<TransitionID, bool>)>,
    place_groups: HashMap<PlaceID, usize>,
}

fn perform_sync_group_discovery(
    locel: &IndexLinkedOCEL,
    noise_thresh: f64,
) -> (
    Vec<(String, String, OCDeclareArcLabel, Vec<String>)>,
    OCPetriNet,
) {
    let ass = mine_associations(locel, noise_thresh);
    let fit = incoporate_control_flow_and_ensure_fitness(locel, noise_thresh, ass);
    let red = perform_transitive_reduction(fit);
    let opt = add_optional_steps(locel, noise_thresh, red);

    // println!("{:#?}", opt);
    // println!("{}", opt.len());

    // Construct Synchronized Place Groups and Places in Petri Nets
    
    let act_ot_involvements = get_activity_object_involvements(locel);

    let mut net = PetriNet::new();
    let trans_map: HashMap<&str, TransitionID> = locel
        .get_ev_types()
        .filter(|et| !et.starts_with(INIT_EVENT_PREFIX) && !et.starts_with(EXIT_EVENT_PREFIX))
        .map(|et| (et, net.add_transition(Some(et.to_string()), None)))
        .collect();
    let mut place_object_type = HashMap::new();
    let mut place_in_out_mult: HashMap<
        PlaceID,
        (HashMap<TransitionID, bool>, HashMap<TransitionID, bool>),
    > = HashMap::new();
    let mut place_groups: HashMap<PlaceID, usize> = HashMap::new();
    let mut place_group_index = 0;
    for (a, b, l1, opts) in &opt {
        if !l1.all.is_empty() {
            assert!(l1.each.is_empty());
            for oa in &l1.all {
                if let ObjectTypeAssociation::Simple { object_type } = oa {
                    if a.starts_with(INIT_EVENT_PREFIX) && b.starts_with(EXIT_EVENT_PREFIX) {
                        continue;
                    }
                    let place_id = net.add_place(None);
                    place_object_type.insert(place_id.clone(), object_type.to_string());
                    if !a.starts_with(INIT_EVENT_PREFIX) {
                        let a_trans = trans_map.get(a.as_str()).unwrap();
                        net.add_arc(
                            ArcType::transition_to_place(*a_trans, place_id),
                            act_ot_involvements
                                .get(a)
                                .unwrap()
                                .get(object_type)
                                .map(|w| if w.max > 1 || w.min < 1 { 10 } else { 1 }),
                        );
                    }
                    if !b.starts_with(EXIT_EVENT_PREFIX) {
                        let b_trans = trans_map.get(b.as_str()).unwrap();

                        net.add_arc(
                            ArcType::place_to_transition(place_id, *b_trans),
                            act_ot_involvements
                                .get(b)
                                .unwrap()
                                .get(object_type)
                                .map(|w| if w.max > 1 || w.min < 1 { 10 } else { 1 }),
                        );
                    }

                    if !a.starts_with(INIT_EVENT_PREFIX) && !b.starts_with(EXIT_EVENT_PREFIX) {
                        place_groups.insert(place_id.clone(), place_group_index);
                        for opt_act in opts {
                            if !opt_act.starts_with(INIT_EVENT_PREFIX)
                                && !opt_act.starts_with(EXIT_EVENT_PREFIX)
                            {
                                let opt_trans = trans_map.get(opt_act.as_str()).unwrap();
                                net.add_arc(
                                    ArcType::transition_to_place(*opt_trans, place_id),
                                    act_ot_involvements
                                        .get(opt_act)
                                        .unwrap()
                                        .get(object_type)
                                        .map(|w| if w.max > 1 || w.min < 1 { 10 } else { 1 }),
                                );
                                net.add_arc(
                                    ArcType::place_to_transition(place_id, *opt_trans),
                                    act_ot_involvements
                                        .get(opt_act)
                                        .unwrap()
                                        .get(object_type)
                                        .map(|w| if w.max > 1 || w.min < 1 { 10 } else { 1 }),
                                );
                            }
                        }
                    }
                }
            }
            place_group_index += 1;
        } else if !l1.each.is_empty() {
            assert!(l1.each.len() == 1);
            if let Some(ObjectTypeAssociation::Simple { object_type }) = l1.each.first() {
                if a.starts_with(INIT_EVENT_PREFIX) && b.starts_with(EXIT_EVENT_PREFIX) {
                    continue;
                }
                let place_id = net.add_place(None);
                // place_groups.insert(place_id.clone(), place_group_index);
                place_object_type.insert(place_id.clone(), object_type.to_string());
                if !a.starts_with(INIT_EVENT_PREFIX) {
                    let a_trans = trans_map.get(a.as_str()).unwrap();

                    net.add_arc(
                        ArcType::transition_to_place(*a_trans, place_id),
                        act_ot_involvements
                            .get(a)
                            .unwrap()
                            .get(object_type)
                            .map(|w| if w.max > 1 || w.min < 1 { 10 } else { 1 }),
                    );
                }

                if !b.starts_with(EXIT_EVENT_PREFIX) {
                    let b_trans = trans_map.get(b.as_str()).unwrap();
                    net.add_arc(
                        ArcType::place_to_transition(place_id, *b_trans),
                        act_ot_involvements
                            .get(b)
                            .unwrap()
                            .get(object_type)
                            .map(|w| if w.max > 1 || w.min < 1 { 10 } else { 1 }),
                    );
                }
                if !a.starts_with(INIT_EVENT_PREFIX) && !b.starts_with(EXIT_EVENT_PREFIX) {
                    for opt_act in opts {
                        if !opt_act.starts_with(INIT_EVENT_PREFIX)
                            && !opt_act.starts_with(EXIT_EVENT_PREFIX)
                        {
                            let opt_trans = trans_map.get(opt_act.as_str()).unwrap();
                            net.add_arc(
                                ArcType::transition_to_place(*opt_trans, place_id),
                                act_ot_involvements
                                    .get(opt_act)
                                    .unwrap()
                                    .get(object_type)
                                    .map(|w| if w.max > 1 || w.min < 1 { 10 } else { 1 }),
                            );
                            net.add_arc(
                                ArcType::place_to_transition(place_id, *opt_trans),
                                act_ot_involvements
                                    .get(opt_act)
                                    .unwrap()
                                    .get(object_type)
                                    .map(|w| if w.max > 1 || w.min < 1 { 10 } else { 1 }),
                            );
                        }
                    }
                }
            }
        }
    }

    let oc_pn = OCPetriNet {
        petri_net: net,
        place_object_type,
        place_in_out_mult,
        place_groups,
    };
    (opt, oc_pn)
}

#[test]
fn sync_group_discovery_eval_table() {
    let datasets = [
        ("Order", "/home/aarkue/dow/ocel/order-management.json"),
        ("P2P", "/home/aarkue/dow/ocel/ocel2-p2p.json"),
        ("Logistics", "/home/aarkue/dow/ocel/ContainerLogistics.json"),
        ("Hinge", "/home/aarkue/dow/ocel/socel2_hinge.xml"),
    ];
    // let noise_thresholds = [0.00, 0.05, 0.10, 0.15, 0.20, 0.25, 0.30];
    let noise_thresholds = [0.00, 0.10, 0.20, 0.30];

    for (name, path) in datasets {
        let ocel = if path.ends_with(".json") {
            import_ocel_json_from_path(path).unwrap()
        } else {
            import_ocel_xml_file(path)
        };
        let locel = preprocess_ocel(ocel);
        // let locel = IndexLinkedOCEL::from(ocel);
        print!(
            "{name} & {} & {} & {} & {} ",
            locel.get_ev_types().count(),
            locel.get_all_evs().count(),
            locel.get_ob_types().count(),
            locel.get_all_obs().count()
        );
        for noise_thresh in noise_thresholds {
            let now = Instant::now();
            let (candidates, oc_sync_pn) = perform_sync_group_discovery(&locel, noise_thresh);
            let duration = now.elapsed();
            let num_places = oc_sync_pn.petri_net.places.len();
            let num_place_groups = oc_sync_pn.place_groups.values().unique().count();
            print!(
                "& {:.2}s & {num_places} & {num_place_groups} ",
                duration.as_secs_f64()
            );
            // println!("{name} with {noise_thresh} yielded {num_places} places with {num_place_groups} groups in  {:?}", duration.as_secs_f64());
            serde_json::to_writer_pretty(
                File::create(format!("./ocpn-place-groups_{name}-{noise_thresh}.json")).unwrap(),
                &oc_sync_pn,
            )
            .unwrap();
        }
        println!("\\\\");
    }
    // let ocel = import_ocel_json_from_path("/home/aarkue/dow/ocel/order-management.json").unwrap();
    // let ocel: process_mining::OCEL = import_ocel_json_from_path("/home/aarkue/dow/ocel/ocel2-p2p.json").unwrap();
    // let ocel = import_ocel_json_from_path("/home/aarkue/dow/ocel/ContainerLogistics.json").unwrap();
    // let ocel = import_ocel_json_from_path("/home/aarkue/dow/ocel/bpic2017-o2o-workflow-qualifier-index-no-ev-attrs-sm.json").unwrap();
    // let ocel = import_ocel_xml_file("/home/aarkue/dow/ocel/socel2_hinge.xml");
    // let ocel = import_ocel_xml_file("/home/aarkue/dow/ocel_inventory_management.xml");
}
