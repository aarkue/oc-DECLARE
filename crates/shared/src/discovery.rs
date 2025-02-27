use std::collections::{HashMap, HashSet};

use indicatif::{ParallelProgressIterator, ProgressIterator};
use itertools::Itertools;
use process_mining::ocel::linked_ocel::IndexLinkedOCEL;
use rayon::prelude::*;

use crate::{
    get_activity_object_involvements, get_object_to_object_involvements,
    get_rev_object_to_object_involvements, perf, OCDeclareArc, OCDeclareArcLabel, OCDeclareArcType,
    OCDeclareNode, ObjectInvolvementCounts, ObjectTypeAssociation, EXIT_EVENT_PREFIX,
    INIT_EVENT_PREFIX,
};

const MAX_COUNT_OPT: Option<usize> = None; //Some(20);
pub fn discover(locel: &IndexLinkedOCEL, noise_thresh: f64) -> Vec<OCDeclareArc> {
    let mut ret = Vec::new();
    let act_ob_inv: HashMap<String, HashMap<String, ObjectInvolvementCounts>> =
        get_activity_object_involvements(locel);
    let ob_ob_inv: HashMap<String, HashMap<String, ObjectInvolvementCounts>> =
        get_object_to_object_involvements(locel);
    let ob_ob_rev_inv = get_rev_object_to_object_involvements(locel);

    // First type of discovery: How many events of a specific type per object of specified type?
    // for ot in locel.get_ob_types() {
    //     // Only consider activities generally involved with objects of a type
    //     let mut ev_types_per_ob: HashMap<&str, Vec<usize>> = act_ob_inv
    //         .iter()
    //         .filter_map(|(act_name, ob_inv)| {
    //             if act_name.starts_with(INIT_EVENT_PREFIX)
    //                 || act_name.starts_with(EXIT_EVENT_PREFIX)
    //             {
    //                 return None;
    //             }
    //             if let Some(oi) = ob_inv.get(ot) {
    //                 return Some((act_name.as_str(), Vec::new()));
    //             }
    //             None
    //         })
    //         .collect();
    //     for ob in locel.get_obs_of_type(ot) {
    //         let ev_types = locel
    //             .get_e2o_rev(ob)
    //             .map(|(_q, e)| &locel.get_ev(e).event_type)
    //             .collect_vec();
    //         ev_types_per_ob.iter_mut().for_each(|(et, counts)| {
    //             counts.push(ev_types.iter().filter(|et2| *et2 == et).count());
    //         });
    //     }
    //     // Now decide on bounds
    //     for (act, counts) in ev_types_per_ob {
    //         // Start with mean
    //         let mean = counts.iter().sum::<usize>() as f64 / counts.len() as f64;
    //         if mean >= 20.0 {
    //             // Probably not interesting (i.e., resource related, grows with log)
    //             continue;
    //         }
    //         let mut n_min = mean.round() as usize;
    //         let mut n_max = n_min;
    //         let min_fitting_len = (counts.len() as f64 * (1.0 - noise_thresh)).ceil() as usize;
    //         while counts
    //             .iter()
    //             .filter(|c| c >= &&n_min && c <= &&n_max)
    //             .count()
    //             < min_fitting_len
    //         {
    //             n_min = if n_min <= 0 { n_min } else { n_min - 1 };
    //             n_max += 1;
    //         }
    //         if n_min == 0 {
    //             // Oftentimes this is just infrequent behavior
    //             continue;
    //         }
    //         if n_max >= 20 {
    //             // Probably not interesting (i.e., resource related, grows with log)
    //             continue;
    //         }
    //         // Got bounds!
    //         // println!("[{ot}] {act}: {n_min} - {n_max} (starting from {mean})");
    //         ret.push(OCDeclareArc {
    //             from: OCDeclareNode::new_ob_init(ot),
    //             to: OCDeclareNode::new_act(act),
    //             arc_type: OCDeclareArcType::ASS,
    //             label: OCDeclareArcLabel {
    //                 each: Vec::default(),
    //                 any: vec![ObjectTypeAssociation::new_simple(ot)],
    //                 all: Vec::default(),
    //             },
    //             counts: (Some(n_min), Some(n_max)),
    //         });
    //     }
    // }

    // Second type of discovery: How many objects of object type per event of specified activity/event type?
    // TODO

    // Third type of discovery: Eventually-follows
    let direction = OCDeclareArcType::ASS;
    let counts = (Some(1), None);
    ret.par_extend(
        locel
            .events_per_type
            .keys()
            .cartesian_product(locel.events_per_type.keys())
            .par_bridge()
            .progress_count(locel.events_per_type.len() as u64 * locel.events_per_type.len() as u64)
            .filter(|(act1, act2)| {
                // return *act1 == "place order" && *act2 == "confirm order";
                if act1.starts_with(INIT_EVENT_PREFIX)
                    || act1.starts_with(EXIT_EVENT_PREFIX)
                    || act2.starts_with(INIT_EVENT_PREFIX)
                    || act2.starts_with(EXIT_EVENT_PREFIX)
                {
                    return false;
                }
                true
            })
            .flat_map(|(act1, act2)| {
                let mut act_arcs = Vec::new();
                let obj_invs = get_direct_or_indirect_object_involvements(
                    act1,
                    act2,
                    &act_ob_inv,
                    &ob_ob_inv,
                    &ob_ob_rev_inv,
                );
                for (ot, is_multiple) in obj_invs {
                    // ANY?
                    let any_label = OCDeclareArcLabel {
                        each: vec![],
                        any: vec![ot],
                        all: vec![],
                    };
                    let sat = perf::get_for_all_evs_perf_thresh(
                        act1,
                        act2,
                        &any_label,
                        &direction,
                        &counts,
                        locel,
                        noise_thresh,
                    );
                    if sat {
                        // It IS a viable candidate!
                        // Also test Each/All:
                        if is_multiple {
                            let each_label = OCDeclareArcLabel {
                                each: any_label.any.clone(),
                                any: vec![],
                                all: vec![],
                            };
                            // Otherwise, do not need to bother with differentiating Each/All!
                            let sat = perf::get_for_all_evs_perf_thresh(
                                act1,
                                act2,
                                &each_label,
                                &direction,
                                &counts,
                                locel,
                                noise_thresh,
                            );
                            if sat {
                                // Each is also valid!
                                // Next, test ALL:
                                let all_label = OCDeclareArcLabel {
                                    each: vec![],
                                    any: vec![],
                                    all: any_label.any.clone(),
                                };
                                let sat = perf::get_for_all_evs_perf_thresh(
                                    act1,
                                    act2,
                                    &all_label,
                                    &direction,
                                    &counts,
                                    locel,
                                    noise_thresh,
                                );
                                if sat {
                                    // All is also valid!
                                    act_arcs.push(all_label);
                                    act_arcs.push(each_label);
                                    act_arcs.push(any_label);
                                } else {
                                    act_arcs.push(each_label);
                                    act_arcs.push(any_label);
                                }
                            } else {
                                act_arcs.push(any_label);
                            }
                        } else {
                            act_arcs.push(OCDeclareArcLabel {
                                each: any_label.any,
                                any: vec![],
                                all: vec![],
                            });
                        }
                    }
                }
                let mut changed = true;
                let mut old: HashSet<_> = act_arcs
                    .iter()
                    .cloned()
                    .collect();
                let mut iteration = 1;
                while changed {
                    println!("{}->{}, |act_arcs|={}",act1,act2,act_arcs.len());
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
                                &counts,
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
                    // .into_iter()
                    .into_par_iter()
                    .filter(move |arc1| {
                        !old.iter()
                            .any(|arc2| *arc1 != *arc2 && arc1.is_dominated_by(arc2))
                    })
                    // .into_iter()
                    .flat_map(move |label| {
                        let mut arc = OCDeclareArc {
                            from: OCDeclareNode::new_act(act1.clone()),
                            to: OCDeclareNode::new_act(act2.clone()),
                            arc_type: OCDeclareArcType::ASS,
                            label,
                            counts: (Some(1), Some(20)),
                        };
                        if arc.get_for_all_evs_perf_thresh(locel, noise_thresh) {
                            arc.counts.1 = None;
                        get_stricter_arrows_for_as(arc, noise_thresh, locel)
                        } else {
                            vec![]
                        }
                    });
                v
            }),
    );

    ret
}

fn get_stricter_arrows_for_as(
    mut a: OCDeclareArc,
    noise_thresh: f64,
    locel: &IndexLinkedOCEL,
) -> Vec<OCDeclareArc> {
    let mut ret: Vec<OCDeclareArc> = Vec::new();
    {
        // Test EF
        a.arc_type = OCDeclareArcType::EF;
        if a.get_for_all_evs_perf_thresh(locel, noise_thresh) {
            // Test DF
            a.arc_type = OCDeclareArcType::DF;
            // let df_viol_frac = a.get_for_all_evs_perf(locel);
            if a.get_for_all_evs_perf_thresh(locel, noise_thresh) {
                ret.push(a.clone());
            } else {
                a.arc_type = OCDeclareArcType::EF;
                ret.push(a.clone());
            }
        }
    }
    {
        // Test EP
        a.arc_type = OCDeclareArcType::EFREV;
        // let ep_viol_frac = a.get_for_all_evs_perf(locel);
        if a.get_for_all_evs_perf_thresh(locel, noise_thresh) {
            // Test DFREV
            a.arc_type = OCDeclareArcType::DFREV;
            // let dp_viol_frac = a.get_for_all_evs_perf(locel);
            if a.get_for_all_evs_perf_thresh(locel, noise_thresh) {
                ret.push(a.clone());
            } else {
                a.arc_type = OCDeclareArcType::EFREV;
                ret.push(a.clone());
            }
        }
    }
    if ret.is_empty() && a.from != a.to {
        a.arc_type = OCDeclareArcType::ASS;
        // if a.get_for_all_evs_perf_thresh(locel, noise_thresh) {
            ret.push(a);
        // }
    }
    ret
}

/// Returns an iterator over different object type associations
/// in particular each item (X,b) consists of an ObjectTypeAssociation X and a flag b, indicating if multiple objects are sometimes involved in the source (or through the O2O)
fn get_direct_or_indirect_object_involvements<'a>(
    act1: &'a str,
    act2: &'a str,
    act_ob_involvement: &'a HashMap<String, HashMap<String, ObjectInvolvementCounts>>,
    obj_obj_involvement: &'a HashMap<String, HashMap<String, ObjectInvolvementCounts>>,
    rev_obj_obj_involvement: &'a HashMap<String, HashMap<String, ObjectInvolvementCounts>>,
) -> Vec<(ObjectTypeAssociation, bool)> {
    let act1_obs: HashSet<_> = act_ob_involvement.get(act1).unwrap().keys().collect();
    let act2_obs: HashSet<_> = act_ob_involvement.get(act2).unwrap().keys().collect();
    act1_obs
        .iter()
        .filter(|ot| act2_obs.contains(*ot))
        .map(|ot| {
            (
                ObjectTypeAssociation::Simple {
                    object_type: ot.to_string(),
                },
                act_ob_involvement.get(act1).unwrap().get(*ot).unwrap().max > 1,
            )
        })
        // .chain(act1_obs.iter().flat_map(|ot| {
        //     obj_obj_involvement
        //         .get(*ot)
        //         .into_iter()
        //         .flat_map(|ots2| {
        //             ots2.iter()
        //                 .filter(|(ot2, _)| act2_obs.contains(ot2))
        //                 // .filter(|(ot2, _)| *ot == "customers" && *ot2 == "employees")
        //                 .map(|(ot2, oi)| {
        //                     (
        //                         ot,
        //                         ot2,
        //                         oi.max > 1
        //                             || act_ob_involvement.get(act1).unwrap().get(*ot).unwrap().max
        //                                 > 1,
        //                     )
        //                 })
        //         })
        //         .map(|(ot1, ot2, multiple)| (ObjectTypeAssociation::new_o2o(*ot1, ot2), multiple))
        //         .collect_vec()
        // }))
        // .chain(act1_obs.iter().flat_map(|ot| {
        //     rev_obj_obj_involvement
        //         .get(*ot)
        //         .into_iter()
        //         .flat_map(|ots2| {
        //             ots2.iter()
        //                 .filter(|(ot2, _)| act2_obs.contains(ot2))
        //                 .map(|(ot2, oi)| {
        //                     (
        //                         ot,
        //                         ot2,
        //                         oi.max > 1
        //                             || act_ob_involvement.get(act1).unwrap().get(*ot).unwrap().max
        //                                 > 1,
        //                     )
        //                 })
        //         })
        //         .map(|(ot1, ot2, multiple)| {
        //             (ObjectTypeAssociation::new_o2o_rev(*ot1, ot2), multiple)
        //         })
        //         .collect_vec()
        // }))
        .collect()
}

fn test_for_resource(l: &OCDeclareArcLabel, obj_inv: &Vec<(ObjectTypeAssociation, bool)>) -> bool {
    let out1: HashSet<_> = get_out_types(&l.all)
        .chain(get_out_types(&l.any))
        .chain(get_out_types(&l.each))
        .collect();
    let link: HashSet<_> = obj_inv
        .iter()
        .map(|(oi, _)| match oi {
            ObjectTypeAssociation::Simple { object_type } => object_type,
            ObjectTypeAssociation::O2O {
                first,
                second,
                reversed,
            } => second,
        })
        .collect();
    let resource_types: HashSet<_> = ["products", "employees", "customers"].into_iter().collect();
    let non_resource_possible = link.iter().any(|t| !resource_types.contains(t.as_str()));
    if non_resource_possible {
        let non_resource_in_arc = out1.iter().any(|t| !resource_types.contains(t.as_str()));
        return non_resource_in_arc;
    }
    true
}
// fn check_compatability(l1: &OCDeclareArcLabel, l2: &OCDeclareArcLabel) -> bool {
//     let out1: HashSet<_> = get_out_types(&l1.all).chain(get_out_types(&l1.any)).chain(get_out_types(&l1.each)).collect();
//     let out2: HashSet<_> = get_out_types(&l2.all).chain(get_out_types(&l2.any)).chain(get_out_types(&l2.each)).collect();
//     out1.is_disjoint(&out2)
// }

fn get_out_types<'a>(ras: &'a Vec<ObjectTypeAssociation>) -> impl Iterator<Item = &'a String> {
    ras.iter().map(|oas| match oas {
        ObjectTypeAssociation::Simple { object_type } => object_type,
        ObjectTypeAssociation::O2O {
            first,
            second,
            reversed,
        } => second,
    })
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use process_mining::{import_ocel_xml_file, ocel::linked_ocel::IndexLinkedOCEL};

    use crate::{
        discovery::get_direct_or_indirect_object_involvements, get_activity_object_involvements,
        get_object_to_object_involvements, get_rev_object_to_object_involvements,
        ObjectInvolvementCounts,
    };

    #[test]
    fn test_obj_involvements() {
        let ocel = import_ocel_xml_file("/home/aarkue/dow/ocel/order-management.xml");
        let locel: IndexLinkedOCEL = ocel.into();
        let act_ob_inv: HashMap<String, HashMap<String, ObjectInvolvementCounts>> =
            get_activity_object_involvements(&locel);
        let ob_ob_inv: HashMap<String, HashMap<String, ObjectInvolvementCounts>> =
            get_object_to_object_involvements(&locel);
        let ob_ob_inv_rev: HashMap<String, HashMap<String, ObjectInvolvementCounts>> =
            get_rev_object_to_object_involvements(&locel);
        let res = get_direct_or_indirect_object_involvements(
            "place order",
            "pick item",
            &act_ob_inv,
            &ob_ob_inv,
            &ob_ob_inv_rev,
        );
        println!("{:?}", res);
    }
}
