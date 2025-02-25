use std::{
    collections::{HashMap, HashSet},
    time::{Duration, Instant},
};

use indicatif::{ParallelProgressIterator, ProgressIterator};
use itertools::Itertools;
use process_mining::ocel::linked_ocel::{IndexLinkedOCEL, LinkedOCELAccess};
use rayon::prelude::*;

use crate::{
    get_activity_object_involvements, get_object_to_object_involvements, perf, OCDeclareArc,
    OCDeclareArcLabel, OCDeclareArcType, OCDeclareNode, ObjectInvolvementCounts,
    ObjectTypeAssociation, EXIT_EVENT_PREFIX, INIT_EVENT_PREFIX,
};

const MAX_COUNT_OPT: Option<usize> = None; //Some(20);
pub fn discover(locel: &IndexLinkedOCEL, noise_thresh: f64) -> Vec<OCDeclareArc> {
    let now = Instant::now();
    let mut ret = Vec::new();
    // First type of discovery: How many events of a specific type per object of specified type?
    let act_ob_inv: HashMap<String, HashMap<String, ObjectInvolvementCounts>> =
        get_activity_object_involvements(locel);
    let ob_ob_inv: HashMap<String, HashMap<String, ObjectInvolvementCounts>> =
        get_object_to_object_involvements(locel);
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
                // let now = Instant::now();
                // let mut arcs = Vec::new();
                // let act1_oi = act_ob_inv.get(act1).unwrap();
                // let act1_ot_set: HashSet<_> = act1_oi.keys().collect();
                // for direction in &[OCDeclareArcType::EF, OCDeclareArcType::EFREV] {
                // for act2 in locel.get_ev_types() {
                // if act1.starts_with(INIT_EVENT_PREFIX)
                //     || act1.starts_with(EXIT_EVENT_PREFIX)
                //     || act2.starts_with(INIT_EVENT_PREFIX)
                //     || act2.starts_with(EXIT_EVENT_PREFIX)
                // {
                //     continue;
                // }
                // let act2_oi = act_ob_inv.get(act2).unwrap();
                // let act2_ot_set: HashSet<_> = act2_oi.keys().collect();
                let mut act_arcs = Vec::new();
                for (ot, is_multiple) in
                    get_direct_or_indirect_object_involvements(act1, act2, &act_ob_inv, &ob_ob_inv)
                {
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
                                    // act_arcs.push(each_label);
                                    // act_arcs.push(any_label);
                                    act_arcs.push(all_label);
                                } else {
                                    // Each should only be preferred if type is not resource-like  
                                    let sat = perf::get_for_all_evs_perf_thresh(
                                        act1,
                                        act2,
                                        &each_label,
                                        &direction,
                                        &(Some(1), Some(20)),
                                        locel,
                                        noise_thresh,
                                    );
                                    if sat {
                                        act_arcs.push(each_label);
                                    } else {
                                        act_arcs.push(any_label);
                                    }
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
                // if now.elapsed().as_secs_f32() > 2.0 {
                //     println!("{:?}",act_arcs);
                // println!(
                //     "Before combining for {} -> {} [Took {:.2?}]",
                //     act1,
                //     act2,
                //     now.elapsed()
                // );
                // }
                // let now = Instant::now();
                let mut changed = true;
                let mut old = Vec::new();
                while changed {
                    // let mut to_remove = HashSet::new();
                    // let mut to_add = HashSet::new();
                    // println!("New iteration: |act_arcs| = {}", act_arcs.len());
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

                            // if !act_arcs.contains(&new_arc_label) {
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
                                return Some(new_arc_label);
                            } else {
                                return None;
                            }
                        })
                        .collect();

                    changed = !new_res.is_empty();
                    old.extend(act_arcs.into_iter());
                    act_arcs = new_res.into_iter().collect();

                    // for arc1_i in 0..act_arcs.len() {
                    //     for arc2_i in (arc1_i + 1)..act_arcs.len() {
                    //         let arc1 = &act_arcs[arc1_i];
                    //         let arc2 = &act_arcs[arc2_i];
                    //         if arc1.is_dominated_by(arc2) || arc2.is_dominated_by(arc1) {
                    //             continue;
                    //         }
                    //         // let this_is_it = arc1.each.len() == 1
                    //         //     && arc1.each.contains(&ObjectTypeAssociation::Simple {
                    //         //         object_type: "orders".to_string(),
                    //         //     })
                    //         //     && arc2.each.len() == 1
                    //         //     && arc2.any.contains(&ObjectTypeAssociation::O2O {
                    //         //         first: "customers".to_string(),
                    //         //         second: "employees".to_string(),
                    //         //         reversed: false,
                    //         //     });
                    //         let new_arc_label = arc1.combine(arc2);

                    //         // if !act_arcs.contains(&new_arc_label) {
                    //         // let n = Instant::now();
                    //         let sat = perf::get_for_all_evs_perf_thresh(
                    //             act1,
                    //             act2,
                    //             &new_arc_label,
                    //             &direction,
                    //             &counts,
                    //             locel,
                    //             noise_thresh,
                    //         );
                    //         // if this_is_it {
                    //         //     println!(
                    //         //         "Trying to combine {:?} and {:?} yielded {:?}",
                    //         //         arc1, arc2, sat
                    //         //     );
                    //         // }

                    //         // let score = perf::get_for_all_evs_perf(
                    //         //     act1,
                    //         //     act2,
                    //         //     &new_arc_label,
                    //         //     &direction,
                    //         //     &counts,
                    //         //     locel,);
                    //         // println!("Trying to combine {:?} and {:?} into {:?}, sat?: {sat}, score: {score:.2}",arc1,arc2,new_arc_label);
                    //         if sat {
                    //             // It IS a viable candidate!
                    //             to_add.insert(new_arc_label);
                    //             to_remove.insert(arc1_i);
                    //             to_remove.insert(arc2_i);
                    //         }
                    //         // }
                    //     }
                    // }
                    // changed = !to_add.is_empty();
                    // act_arcs = act_arcs
                    //     .into_iter()
                    //     .enumerate()
                    //     .filter_map(|(index, arc)| {
                    //         if to_remove.contains(&index) {
                    //             None
                    //         } else {
                    //             Some(arc)
                    //         }
                    //     })
                    //     // .filter(|a| to_add.iter().any(|b| a.is_dominated_by(b)))
                    //     .chain(to_add.clone())
                    //     .collect();
                }
                act_arcs = old;
                // println!(
                //     "Combining for {} -> {} [Took {:.2?}]",
                //     act1,
                //     act2,
                //     now.elapsed()
                // );
                // let now = Instant::now();
                // arcs.extend(
                let v = act_arcs
                    .clone()
                    // .into_iter()
                    // .iter()
                    .into_par_iter()
                    .filter(move |arc1| {
                        !act_arcs.iter().any(|arc2| {
                            *arc1 != *arc2 && arc1.is_dominated_by(arc2)
                            // && !arc2.is_dominated_by(&arc1)
                        })
                    })
                    // .into_iter()
                    .flat_map(move |label| {
                        let arc = OCDeclareArc {
                            from: OCDeclareNode::new_act(act1.clone()),
                            to: OCDeclareNode::new_act(act2.clone()),
                            arc_type: OCDeclareArcType::ASS,
                            label: label,
                            counts: (Some(1), MAX_COUNT_OPT),
                        };
                        // vec![arc]
                        get_stricter_arrows_for_as(arc, noise_thresh, locel)
                        // if arc.get_for_all_evs_perf(locel) <= noise_thresh {
                        //     Some(arc)
                        // } else {
                        //     arc.counts.1 = MAX_COUNT_OPT;
                        //     if arc.get_for_all_evs_perf(locel) <= noise_thresh {
                        //         arc.counts.1 = None;
                        //         Some(arc)
                        //     } else {
                        //         None
                        //     }
                        // }
                    });
                // .collect_vec();

                // println!(
                //     "Creating v for {} -> {}, |act_arcs|={}, |v|={} [Took {:.2?}]",
                //     act1,
                //     act2,
                //     act_arcs.len(),
                //     v.len(),
                //     now.elapsed()
                // );
                // println!("Finishing {} -> {} took {:.2?}", act1, act2, now.elapsed());
                // println!("{} -> {} took {:.2?}",act1,act2,now.elapsed());
                v
                // );

                // if now.elapsed().as_secs_f32() > 2.0 {
                // println!("After combining: [{:.2?}]",now.elapsed());
                // }
                // }
                // }

                // arcs
            }),
    );
    // Also test if the discovered (EF / EF-rev) constraints hold if we set min = max = 1
    // Also include directly-follows constraints:
    // Collect all EF constraints that hold (i.e., also the dominated/superseeded ones)
    // For each of them check if DF also holds
    // Collect DF constraints, remove dominated ones
    // Remove EF constraints dominated by DF constraints (i.e., those where the sets are the same)

    // Fourth type of discovery: NOT Eventually Follows
    // There we also have monotonicity properties, but different ones!
    // In particular, A --T-//-> B implies A --T,T'--//-> B and so on

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
    if ret.is_empty() {
        if a.from != a.to {
            a.arc_type = OCDeclareArcType::ASS;
            // if a.get_for_all_evs_perf(locel) <= noise_thresh {
            ret.push(a);
        }
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
) -> Vec<(ObjectTypeAssociation, bool)> {
    let act1_obs: HashSet<_> = act_ob_involvement.get(act1).unwrap().keys().collect();
    let act2_obs: HashSet<_> = act_ob_involvement.get(act2).unwrap().keys().collect();
    return act1_obs
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
        .chain(act1_obs.iter().flat_map(|ot| {
            obj_obj_involvement
                .get(*ot)
                .into_iter()
                .flat_map(|ots2| {
                    ots2.iter()
                        .filter(|(ot2, _)| act2_obs.contains(ot2))
                        // .filter(|(ot2, _)| *ot == "customers" && *ot2 == "employees")
                        .map(|(ot2, oi)| {
                            (
                                ot,
                                ot2,
                                oi.max > 1
                                    || act_ob_involvement.get(act1).unwrap().get(*ot).unwrap().max
                                        > 1,
                            )
                        })
                })
                .map(|(ot1, ot2, multiple)| (ObjectTypeAssociation::new_o2o(*ot1, ot2), multiple))
                .collect_vec()
        }))
        // // TODO: Fix multiplicity?! Probably better to construct reverse o2o relation counts directly!
        // .chain(act2_obs.iter().flat_map(|ot| {
        //     obj_obj_involvement
        //         .get(*ot)
        //         .into_iter()
        //         .flat_map(|ots2| {
        //             ots2.iter()
        //                 .filter(|(ot2, _)| act1_obs.contains(ot2))
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
        //         .map(|(ot1, ot2, multiple)| (ObjectTypeAssociation::new_o2o_rev(ot2, &ot1), multiple))
        //         .collect_vec()
        // }))
        .collect();
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use process_mining::{import_ocel_xml_file, ocel::linked_ocel::IndexLinkedOCEL};

    use crate::{
        discovery::get_direct_or_indirect_object_involvements, get_activity_object_involvements,
        get_object_to_object_involvements, ObjectInvolvementCounts,
    };

    use super::discover;

    #[test]
    fn test_obj_involvements() {
        let ocel = import_ocel_xml_file("/home/aarkue/dow/ocel/order-management.xml");
        let locel: IndexLinkedOCEL = ocel.into();
        let act_ob_inv: HashMap<String, HashMap<String, ObjectInvolvementCounts>> =
            get_activity_object_involvements(&locel);
        let ob_ob_inv: HashMap<String, HashMap<String, ObjectInvolvementCounts>> =
            get_object_to_object_involvements(&locel);
        let res = get_direct_or_indirect_object_involvements(
            "place order",
            "pick item",
            &act_ob_inv,
            &ob_ob_inv,
        );
        println!("{:?}", res);
    }
}
