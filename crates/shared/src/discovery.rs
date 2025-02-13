use std::{
    collections::{HashMap, HashSet},
    time::Instant,
};

use itertools::Itertools;
use process_mining::{
    ocel::linked_ocel::{IndexLinkedOCEL, LinkedOCELAccess},
    petri_net::petri_net_struct::ArcType,
};
use rayon::iter::{ParallelBridge, ParallelIterator};

use crate::{
    get_activity_object_involvements, OCDeclareArc, OCDeclareArcLabel, OCDeclareArcType,
    OCDeclareNode, ObjectTypeAssociation, INIT_EVENT_PREFIX,
};

pub fn discover(locel: &IndexLinkedOCEL, noise_thresh: f64) -> Vec<OCDeclareArc> {
    let mut ret = Vec::new();
    // First type of discovery: How many events of a specific type per object of specified type?
    let act_ob_inv = get_activity_object_involvements(locel);
    for ot in locel.get_ob_types() {
        // Only consider activities generally involved with objects of a type
        let mut ev_types_per_ob: HashMap<&str, Vec<usize>> = act_ob_inv
            .iter()
            .filter_map(|(act_name, ob_inv)| {
                if act_name.starts_with(INIT_EVENT_PREFIX) {
                    return None;
                }
                if let Some(oi) = ob_inv.get(ot) {
                    return Some((act_name.as_str(), Vec::new()));
                }
                None
            })
            .collect();
        for ob in locel.get_obs_of_type(ot) {
            let ev_types = locel
                .get_e2o_rev(ob)
                .map(|(_q, e)| &locel.get_ev(e).event_type)
                .collect_vec();
            ev_types_per_ob.iter_mut().for_each(|(et, counts)| {
                counts.push(ev_types.iter().filter(|et2| *et2 == et).count());
            });
        }
        // Now decide on bounds
        for (act, counts) in ev_types_per_ob {
            // Start with mean
            let mean = counts.iter().sum::<usize>() as f64 / counts.len() as f64;
            if mean >= 20.0 {
                // Probably not interesting (i.e., resource related, grows with log)
                continue;
            }
            let mut n_min = mean.round() as usize;
            let mut n_max = n_min;
            let min_fitting_len = (counts.len() as f64 * (1.0 - noise_thresh)).ceil() as usize;
            while counts
                .iter()
                .filter(|c| c >= &&n_min && c <= &&n_max)
                .count()
                <= min_fitting_len
            {
                n_min = if n_min <= 0 { n_min } else { n_min - 1 };
                n_max = n_max + 1;
            }
            if n_min == 0 {
                // Oftentimes this is just infrequent behavior
                continue;
            }
            if n_max >= 20 {
                // Probably not interesting (i.e., resource related, grows with log)
                continue;
            }
            // Got bounds!
            // println!("[{ot}] {act}: {n_min} - {n_max} (starting from {mean})");
            ret.push(OCDeclareArc {
                from: OCDeclareNode::new_ob_init(ot),
                to: OCDeclareNode::new_act(act),
                arc_type: OCDeclareArcType::ASS,
                label: OCDeclareArcLabel {
                    each: Vec::default(),
                    any: vec![ObjectTypeAssociation::new_simple(ot)],
                    all: Vec::default(),
                },
                counts: (Some(n_min), Some(n_max)),
            });
        }
    }

    // Second type of discovery: How many objects of object type per event of specified activity/event type?

    // Third type of discovery: Eventually-follows
    ret.extend(
        locel
            .get_ev_types()
            // .par_bridge()
            .flat_map(|act1| {
                let mut arcs = Vec::new();
                let act1_oi = act_ob_inv.get(act1).unwrap();
                let act1_ot_set: HashSet<_> = act1_oi.keys().collect();
                for act2 in locel.get_ev_types() {
                    // Currently this is not supported, however: TODO: Also support self-loop arcs
                    if act1 == act2 {
                        continue;
                    }
                    let act2_ot_set: HashSet<_> = act_ob_inv.get(act2).unwrap().keys().collect();
                    let mut act_arcs = Vec::new();
                    for ot in act2_ot_set.intersection(&act1_ot_set) {
                        // if *ot == "orders" || *ot == "items" {

                        // ANY?
                        let label = OCDeclareArcLabel {
                            each: vec![],
                            any: vec![ObjectTypeAssociation::new_simple(*ot)],
                            all: vec![],
                        };
                        let any_arc = OCDeclareArc {
                            from: OCDeclareNode::new_act(act1),
                            to: OCDeclareNode::new_act(act2),
                            arc_type: OCDeclareArcType::EF,
                            label,
                            counts: (Some(1), None),
                        };
                        // let now = Instant::now();
                        let violation_frac = any_arc.get_for_all_evs_perf(locel);
                        // let d = now.elapsed();
                        // if d.as_secs_f64() >= 0.1 {
                        //     println!("{ot}: {act1} -> {act2} {:?}",d);
                        // }
                        // CAREFULL! Talk about violated events here! TODO
                        // Otherwise monotonicity does not hold the same directly.
                        // let violation_frac = tot_vio as f64 / tot_sit as f64;
                        if violation_frac <= noise_thresh {
                            // It IS a viable candidate!
                            // act_arcs.insert(any_arc.label.clone());
                            // Also test Each/All:
                            if let Some(oi) = act1_oi.get(ot.as_str()) {
                                let each_arc = OCDeclareArc {
                                    from: any_arc.from.clone(),
                                    to: any_arc.to.clone(),
                                    arc_type: any_arc.arc_type.clone(),
                                    label: OCDeclareArcLabel {
                                        each: any_arc.label.any.clone(),
                                        any: vec![],
                                        all: vec![],
                                    },
                                    counts: any_arc.counts,
                                };
                                if oi.max > 1 {
                                    // Otherwise, do not need to bother with differentiating Each/All!
                                    let violation_frac = each_arc.get_for_all_evs_perf(locel);
                                    if violation_frac <= noise_thresh {
                                        // Each is also valid!
                                        // Next, test ALL:
                                        let all_arc = OCDeclareArc {
                                            from: any_arc.from.clone(),
                                            to: any_arc.to.clone(),
                                            arc_type: any_arc.arc_type.clone(),
                                            label: OCDeclareArcLabel {
                                                each: vec![],
                                                any: vec![],
                                                all: any_arc.label.any.clone(),
                                            },
                                            counts: any_arc.counts,
                                        };
                                        let violation_frac = all_arc.get_for_all_evs_perf(locel);
                                        if violation_frac <= noise_thresh {
                                            // All is also valid!
                                            act_arcs.push(all_arc.label);
                                        } else {
                                            act_arcs.push(each_arc.label);
                                        }
                                    } else {
                                        act_arcs.push(any_arc.label);
                                    }
                                } else {
                                    act_arcs.push(each_arc.label);
                                }
                            }
                        }
                        // }
                    }
                    let mut changed = true;
                    while changed {
                        let mut to_remove = HashSet::new();
                        let mut to_add = HashSet::new();
                        
                        for arc1_i in 0..act_arcs.len() {
                            for arc2_i in (arc1_i+1)..act_arcs.len() {
                                let arc1 = &act_arcs[arc1_i];
                                let arc2 = &act_arcs[arc2_i];
                                // if arc1.is_dominated_by(arc2) || arc2.is_dominated_by(arc1){
                                //     continue;
                                // }
                                    let new_arc_label = arc1.combine(&arc2);
                                    // if !act_arcs.contains(&new_arc_label) {
                                        let new_arc = OCDeclareArc {
                                            from: OCDeclareNode::new_act(act1),
                                            to: OCDeclareNode::new_act(act2),
                                            arc_type: OCDeclareArcType::EF,
                                            label: new_arc_label,
                                            counts: (Some(1), None),
                                        };
                                        let violation_frac = new_arc.get_for_all_evs_perf(locel);
                                        if violation_frac <= noise_thresh {
                                            // println!("Combined into {:?}", new_arc);
                                            // It IS a viable candidate!
                                            to_add.insert(new_arc.label);
                                            to_remove.insert(arc1);
                                            to_remove.insert(arc2);
                                        }
                                    // }
                                }
                            }
                        changed = !to_add.is_empty();
                        act_arcs = act_arcs
                            .iter()
                            .filter(|arc| !to_remove.contains(arc))
                            .cloned()
                            .chain(to_add)
                            .collect();
                    }
                 
                    arcs.extend(act_arcs.iter().filter(|arc1| {
                        !act_arcs.iter().any(|arc2| {
                            *arc1 != arc2 && arc1.is_dominated_by(arc2) // && !arc2.is_dominated_by(&arc1)
                        })
                    }).into_iter().map(|label| OCDeclareArc {
                        from: OCDeclareNode::new_act(act1),
                        to: OCDeclareNode::new_act(act2),
                        arc_type: OCDeclareArcType::EF,
                        label: label.clone(),
                        counts: (Some(1), None),
                    }));
                }
                arcs
            })
            .collect::<Vec<_>>(),
    );

    // Fourth type of discovery: NOT Eventually Follows

    return ret;
}

#[cfg(test)]
mod test {
    use process_mining::{import_ocel_xml_file, ocel::linked_ocel::IndexLinkedOCEL};

    use super::discover;

    #[test]
    fn test_discovery_order_management() {
        let ocel = import_ocel_xml_file("/home/aarkue/dow/ocel/order-management.xml");
        let locel: IndexLinkedOCEL = ocel.into();
        let res = discover(&locel, 0.2);
        println!("{:?}", res);
    }
}
