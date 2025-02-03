#![feature(precise_capturing)]

use std::{borrow::Cow, collections::HashSet, hash::Hash, time::UNIX_EPOCH};

use itertools::{Itertools, MultiProduct};
pub use process_mining;
use process_mining::{
    export_ocel_json_path, export_ocel_xml_path, ocel::{
        linked_ocel::{index_linked_ocel::{EventIndex, ObjectIndex}, IndexLinkedOCEL, LinkedOCELAccess},
        ocel_struct::{OCELEvent, OCELRelationship, OCELType},
    }, OCEL
};

use serde::{Deserialize, Serialize};
const INIT_EVENT_PREFIX: &str = "<init>";
pub fn preprocess_ocel(mut ocel: OCEL) -> IndexLinkedOCEL {
    ocel.event_types
        .extend(ocel.object_types.iter().map(|ot| OCELType {
            name: format!("{} {}", INIT_EVENT_PREFIX, ot.name),
            attributes: Vec::default(),
        }));
    ocel.events.extend(
        ocel.objects
            .iter()
            .map(|ob| {
                // let first_ev = ocel
                //     .events
                //     .iter()
                //     .filter(|ev| ev.relationships.iter().any(|r| r.object_id == ob.id))
                //     .sorted_by_key(|ev| ev.time)
                //     .next();
                // let first_ev_time = first_ev.map(|ev| ev.time).unwrap_or_default();
                OCELEvent {
                    id: format!("{}_{}_{}", INIT_EVENT_PREFIX, ob.object_type,ob.id),
                    event_type: format!("{} {}", INIT_EVENT_PREFIX, ob.object_type),
                    time: Default::default(),
                    attributes: Vec::default(),
                    relationships: vec![OCELRelationship {
                        object_id: ob.id.clone(),
                        qualifier: String::from("init"),
                    }],
                }
            }).into_iter(),
    );
    ocel.into()
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "type")]
enum OCDeclareNode {
    Activity { activity: String },
    ObjectInit { object_type: String },
    ObjectEnd { object_type: String },
}

impl<'a> Into<Cow<'a, String>> for &'a OCDeclareNode {
    fn into(self) -> Cow<'a, String> {
        match self {
            OCDeclareNode::Activity { activity } => Cow::Borrowed(activity),
            OCDeclareNode::ObjectInit { object_type } => {
                Cow::Owned(format!("<init> {object_type}"))
            }
            OCDeclareNode::ObjectEnd { object_type } => Cow::Owned(format!("<exit> {object_type}")),
        }
    }
}

impl OCDeclareNode {
    pub fn new_act<T: Into<String>>(act: T) -> Self {
        Self::Activity {
            activity: act.into(),
        }
    }

    pub fn new_ob_init<T: Into<String>>(ob_type: T) -> Self {
        Self::ObjectInit {
            object_type: ob_type.into(),
        }
    }
    pub fn new_ob_end<T: Into<String>>(ob_type: T) -> Self {
        Self::ObjectEnd {
            object_type: ob_type.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct OCDeclareArc {
    from: OCDeclareNode,
    to: OCDeclareNode,
    arc_type: OCDeclareArcType,
    label: OCDeclareArcLabel,
    /// First tuple element: min count (optional), Second: max count (optional)
    counts: (Option<usize>, Option<usize>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "type")]
pub enum ViolationInfo {
    TooMany {
        source_ev: String,
        matching_evs: Vec<String>,
        all_obs: Vec<String>,
        count: usize,
    },
    TooFew {
        source_ev: String,
        matching_evs: Vec<String>,
        all_obs: Vec<String>,
        count: usize,
    },
}
use rayon::prelude::*;
use ts_rs::TS;
impl OCDeclareArc {
    pub fn get_for_all_evs(
        &self,
        linked_ocel: &IndexLinkedOCEL,
    ) -> (usize, usize, Vec<(usize, Vec<ViolationInfo>)>) {
        let inner_res: Vec<_> = linked_ocel
            .get_evs_of_type(Into::<Cow<_>>::into(&self.from).as_str())
            // .get(Into::<Cow<_>>::into(&self.from).as_str())
            // .unwrap()
            .par_bridge()
            // iter()
            .map(|ev| self.get_for_ev(ev, linked_ocel))
            .collect();
        let total_situations = inner_res.iter().map(|e| e.0).sum();
        let total_violations = inner_res.iter().map(|e| e.1.len()).sum();
        (total_situations, total_violations, inner_res)
    }
    pub fn get_for_ev<'a>(
        &self,
        ev_index: &EventIndex,
        linked_ocel: &IndexLinkedOCEL,
    ) -> (usize, Vec<ViolationInfo>) {
        let ev = linked_ocel.get_ev(ev_index);
        let res = self
            .label
            .get_bindings(ev_index, linked_ocel)
            .map(|binding| {
                let binding = binding; //.collect_vec();
                let to = Into::<Cow<_>>::into(&self.to);
                let evs = get_evs_with_objs(&binding, linked_ocel, to.as_str())
                    .into_iter()
                    .filter(|ev2| {
                        let ev2 = linked_ocel.get_ev(ev2);
                        match self.arc_type {
                            OCDeclareArcType::ASS => true,
                            OCDeclareArcType::EF => ev.time < ev2.time,
                            OCDeclareArcType::EFREV => ev.time > ev2.time,
                        }
                    })
                    .collect_vec();
                let count = evs.len();

                if self.counts.0.is_some_and(|n_min| count < n_min) {
                    return Some(ViolationInfo::TooFew {
                        source_ev: ev.id.clone(),
                        matching_evs: evs
                            .into_iter()
                            .map(|e| linked_ocel.get_ev(&e).id.clone())
                            .collect(),
                        all_obs: binding
                            .iter()
                            .flat_map(|b| match b {
                                // SetFilter::Any(items) => todo!(),
                                SetFilter::All(items) => {
                                    Some(items.iter().map(|o| linked_ocel.get_ob(o).id.clone()))
                                }
                                _ => None,
                            })
                            .flatten()
                            .collect(),
                        count,
                    });
                }
                if self.counts.1.is_some_and(|n_max| count > n_max) {
                    return Some(ViolationInfo::TooMany {
                        source_ev: ev.id.clone(),
                        matching_evs: evs
                            .into_iter()
                            .map(|e| linked_ocel.get_ev(&e).id.clone())
                            .collect(),
                        all_obs: binding
                            .iter()
                            .flat_map(|b| match b {
                                // SetFilter::Any(items) => todo!(),
                                SetFilter::All(items) => {
                                    Some(items.iter().map(|o| linked_ocel.get_ob(o).id.clone()))
                                }
                                _ => None,
                            })
                            .flatten()
                            .collect(),
                        count,
                    });
                }
                return None;

                // (binding,count)

                // binding.len()
            })
            .collect_vec();
        // let num_viol_bindings = res.iter().filter(|o| o.is_some()).count();
        // let num_sat_bindings = res.len() - num_viol_bindings;
        return (res.len(), res.into_iter().flatten().collect());
    }
}

fn get_evs_with_objs<'a>(
    objs: &Vec<SetFilter<ObjectIndex>>,
    linked_ocel: &'a IndexLinkedOCEL,
    etype: &'a str,
) -> Vec<EventIndex> {
    let mut initial: Vec<EventIndex> = match &objs[0] {
        SetFilter::Any(items) =>
        //  linked_ocel
        //     .events_per_type
        //     .get(etype)
        //     .unwrap()
        //     .iter()
        {
            items
                .iter()
                .flat_map(|o| {
                    linked_ocel
                        .get_e2o_rev(o)
                        // .get(o)
                        // .unwrap()
                        // .iter()
                        .map(|e| e.1)
                        .filter_map(|e| {
                            let ev = linked_ocel.get_ev(&e);
                            if ev.event_type == *etype {
                                Some(*e)
                            } else {
                                None
                            }
                        })
                })
                // .map(|e| (&e.id).into())
                .collect()
        }
        SetFilter::All(items) => {
            if items.len() == 0 {
                return Vec::new();
            }
            // items.iter().flat_map(|o| linked_ocel.e2o_rel_rev.get(o).unwrap().iter().map(|e| e.1).filter(|e| e.event_type == *etype))
            linked_ocel
                .get_e2o_rev(&items[0])
                // .get(&items[0])
                // .unwrap()
                // .into_iter()
                .filter_map(|(_, e)| {
                    let ev = linked_ocel.get_ev(e);
                    if ev.event_type == etype
                        && items.iter().skip(1).all(|o| {
                            linked_ocel
                                .get_e2o(e)
                                // .iter()
                                .any(|(q, o_index)| o_index == o)
                        })
                    {
                        Some(*e)
                    } else {
                        None
                    }
                })
                // .map(|x| (&x.1.id).into())
                .collect_vec()
        }
    };
    // println!("Initial is of size: {}",initial.len());
    for o in objs.iter() {
        initial.retain(|e| {
            let obs = linked_ocel
                .get_e2o(e)
                // .get(e)
                // .unwrap()
                // .iter()
                .map(|o| *o.1)
                .collect();
            o.check(&obs)
        });
    }
    initial
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
// #[serde(tag = "type")]
enum OCDeclareArcType {
    ASS,
    EF,
    EFREV,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "type")]
enum ObjectTypeAssociation {
    Simple {
        object_type: String,
    },
    O2O {
        first: String,
        second: String,
        reversed: bool,
    },
}

impl ObjectTypeAssociation {
    pub fn new_simple<T: Into<String>>(ot: T) -> Self {
        Self::Simple {
            object_type: ot.into(),
        }
    }
    pub fn new_o2o<T: Into<String>>(ot1: T, ot2: T) -> Self {
        Self::O2O {
            first: ot1.into(),
            second: ot2.into(),
            reversed: false,
        }
    }
    pub fn new_o2o_rev<T: Into<String>>(ot1: T, ot2: T) -> Self {
        Self::O2O {
            first: ot1.into(),
            second: ot2.into(),
            reversed: true,
        }
    }

    pub fn get_for_ev<'a>(
        &self,
        ev: &EventIndex,
        linked_ocel: &'a IndexLinkedOCEL,
    ) -> Vec<ObjectIndex> {
        match self {
            ObjectTypeAssociation::Simple { object_type } => linked_ocel
                .get_e2o(ev)
                .map(|x| x.1)
                .filter_map(|o| {
                    let ob = linked_ocel.get_ob(&o);
                    if ob.object_type == *object_type {
                        Some(*o)
                    } else {
                        None
                    }
                })
                .collect(),
            ObjectTypeAssociation::O2O {
                first,
                second,
                reversed,
            } => linked_ocel
                .get_e2o(ev)
                // .unwrap()
                // .iter()
                .map(|x| x.1)
                .filter(|o| linked_ocel.get_ob(&o).object_type == *first)
                .flat_map(|o| {
                    if !reversed {
                        linked_ocel
                            .get_o2o(&o)
                            // .get(&Into::<ObjectID>::into(&o.id))
                            // .unwrap()
                            // .iter()
                            .map(|rel| rel.1)
                            .filter(|o2| linked_ocel.get_ob(o2).object_type == *second)
                            .collect_vec()
                    } else {
                        linked_ocel
                            .get_o2o_rev(&o)
                            // .get(&Into::<ObjectID>::into(&o.id))
                            // .unwrap()
                            // .iter()
                            .map(|rel| rel.1)
                            .filter(|o2| linked_ocel.get_ob(o2).object_type == *second)
                            .collect_vec()
                    }
                })
                .map(|o| *o)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default, TS)]
#[ts(export)]
struct OCDeclareArcLabel {
    each: Vec<ObjectTypeAssociation>,
    any: Vec<ObjectTypeAssociation>,
    all: Vec<ObjectTypeAssociation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
enum SetFilter<T: Eq + Hash> {
    Any(Vec<T>),
    All(Vec<T>),
}

impl<T: Eq + Hash> SetFilter<T> {
    pub fn check(&self, s: &HashSet<T>) -> bool {
        match self {
            SetFilter::Any(items) => items.iter().any(|i| s.contains(i)),
            SetFilter::All(items) => items.iter().all(|i| s.contains(i)),
        }
    }
}

impl<'a, 'b> OCDeclareArcLabel {
    pub fn get_bindings(
        &'a self,
        ev: &'a EventIndex,
        linked_ocel: &'a IndexLinkedOCEL,
    ) -> impl Iterator<Item = Vec<SetFilter<ObjectIndex>>> + use<'a, 'b>
//  impl Iterator<Item = impl Iterator<Item = SetFilter<ObjectIndex>>>
    {
        self.each
            .iter()
            .map(|otass| otass.get_for_ev(ev, linked_ocel))
            .multi_cartesian_product()
            .map(|product| {
                self.all
                    .iter()
                    .map(|otass| SetFilter::All(otass.get_for_ev(ev, linked_ocel)))
                    .chain(
                        if product.is_empty() {
                            Vec::default()
                        } else {
                            vec![SetFilter::All(product)]
                        }
                        .into_iter(),
                    )
                    .chain(
                        self.any
                            .iter()
                            .map(|otass| SetFilter::Any(otass.get_for_ev(ev, linked_ocel))),
                    )
                    .collect_vec()
            })
        // .collect_vec()
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use process_mining::import_ocel_json_from_path;

    use super::*;

    #[test]
    fn it_works() {
        let ocel =
            import_ocel_json_from_path("/home/aarkue/dow/ocel/order-management.json").unwrap();
        // let ocel = import_ocel_json_from_path(
        //     "/home/aarkue/dow/ocel/bpic2017-o2o-workflow-qualifier.json",
        // )
        // .unwrap();
        let linked_ocel: IndexLinkedOCEL = preprocess_ocel(ocel);
        let x = OCDeclareArc {
            from: OCDeclareNode::new_act("item out of stock"),
            to: OCDeclareNode::new_act("reorder item"),
            arc_type: OCDeclareArcType::EF,
            label: OCDeclareArcLabel {
                each: vec![ObjectTypeAssociation::new_simple("items")],
                // all: vec![ObjectTypeAssociation::new_simple("orders")],
                any: vec![ObjectTypeAssociation::new_simple("employees")],
                ..Default::default()
            },
            counts: (Some(1), None),
        };

        let x: OCDeclareArc = serde_json::from_str(r#"{"from":{"type":"Activity","activity":"payment reminder"},"to":{"type":"Activity","activity":"item out of stock"},"arc_type":"EFREV","counts":[0,0],"label":{"each":[{"type":"O2O","first":"orders","second":"items","reversed":false}],"any":[],"all":[]}}"#).unwrap();

        // let x = OCDeclareArc {
        //     from: OCDeclareNode::new_act("A_Accepted"),
        //     to: OCDeclareNode::new_act("O_Created"),
        //     arc_type: OCDeclareArcType::EF,
        //     label: OCDeclareArcLabel {
        //         any: vec![ObjectTypeAssociation::new_o2o("Application", "Offer"),ObjectTypeAssociation::new_simple("Case_R")],
        //         ..Default::default()
        //     },
        // };
        let now = Instant::now();
        let (total, violated, all_res) = x.get_for_all_evs(&linked_ocel);
        println!("Took {:?}", now.elapsed());
        println!(
            "{violated} / {total}:  {:.?}",
            violated as f64 / total as f64
        );
        // println!("{:?}", all_res.iter().take(10).collect_vec());

        // let count: usize = all_res.iter().flatten().sum();

        // let at_least_one: usize = all_res.iter().flatten().filter(|r| **r >= 1).count();

        // println!("Len: {}", all_res.len());
        // println!("Count: {count}");
        // println!("At least one: {}", at_least_one);
        // println!(
        //     "Violation percentage: {:.2}%",
        //     100.0 * (1.0 - (at_least_one as f32 / all_res.len() as f32))
        // )
        // println!("{}", serde_json::to_string_pretty(&x).unwrap());

        // println!("{:?}", x);
    }
}
