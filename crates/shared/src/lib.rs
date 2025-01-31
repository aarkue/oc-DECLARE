use std::{borrow::Cow, collections::HashSet, hash::Hash};

use itertools::{Itertools, MultiProduct};
pub use process_mining;
use process_mining::ocel::{
    linked_ocel::{EventID, LinkedOCEL, ObjectID},
    ocel_struct::OCELEvent,
};
use serde::{Deserialize, Serialize};

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
impl<'a> OCDeclareArc {
    pub fn get_for_all_evs(&self, linked_ocel: &LinkedOCEL<'_>) -> (usize,usize,Vec<(usize, Vec<ViolationInfo>)>) {
        let inner_res: Vec<_> = linked_ocel
            .events_per_type
            .get(Into::<Cow<_>>::into(&self.from).as_str())
            .unwrap()
            .par_iter()
            // iter()
            .map(|ev| self.get_for_ev(ev, linked_ocel))
            .collect();
        let total_situations = inner_res.iter().map(|e| e.0).sum();
        let total_violations = inner_res.iter().map(|e| e.1.len()).sum();
        (total_situations,total_violations,inner_res)
    }
    pub fn get_for_ev(
        &self,
        ev: &'a OCELEvent,
        linked_ocel: &LinkedOCEL<'_>,
    ) -> (usize,Vec<ViolationInfo>) {
        let res = self.label
            .get_bindings(ev, linked_ocel)
            .map(|binding| {
                let binding = binding.collect_vec();
                let to = Into::<Cow<_>>::into(&self.to);
                let evs = get_evs_with_objs(
                    &binding,
                    linked_ocel,
                    to.as_str(),
                ).into_iter().filter(|ev2| {
                    match self.arc_type {
                        OCDeclareArcType::ASS => true,
                        OCDeclareArcType::EF => ev.time < linked_ocel.events.get(ev2).unwrap().time,
                        OCDeclareArcType::EFREV => ev.time > linked_ocel.events.get(ev2).unwrap().time,
                    }
                }).collect_vec();
                let count = evs.len();

                if self.counts.0.is_some_and(|n_min| count < n_min) {
                    return Some(ViolationInfo::TooFew {
                        source_ev: ev.id.clone(),
                        matching_evs: evs
                            .into_iter()
                            .map(|e| linked_ocel.events.get(&e).unwrap().id.clone())
                            .collect(),
                        all_obs: binding
                            .iter()
                            .flat_map(|b| match b {
                                // SetFilter::Any(items) => todo!(),
                                SetFilter::All(items) => Some(
                                    items
                                        .iter()
                                        .map(|o| linked_ocel.objects.get(o).unwrap().id.clone()),
                                ),
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
                            .map(|e| linked_ocel.events.get(&e).unwrap().id.clone())
                            .collect(),
                        all_obs: binding
                            .iter()
                            .flat_map(|b| match b {
                                // SetFilter::Any(items) => todo!(),
                                SetFilter::All(items) => Some(
                                    items
                                        .iter()
                                        .map(|o| linked_ocel.objects.get(o).unwrap().id.clone()),
                                ),
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
        return (res.len(),res.into_iter().flatten().collect())
    }
}

fn get_evs_with_objs<'a>(
    objs: &Vec<SetFilter<ObjectID<'_>>>,
    linked_ocel: &'a LinkedOCEL<'_>,
    etype: &'a str,
) -> Vec<EventID<'a>> {
    let mut initial: Vec<EventID> = match &objs[0] {
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
                        .e2o_rel_rev
                        .get(o)
                        .unwrap()
                        .iter()
                        .map(|e| e.1)
                        .filter(|e| e.event_type == *etype)
                })
                .map(|e| (&e.id).into())
                .collect()
        }
        SetFilter::All(items) => {
            if items.len() == 0 {
                return Vec::new();
            }
            // items.iter().flat_map(|o| linked_ocel.e2o_rel_rev.get(o).unwrap().iter().map(|e| e.1).filter(|e| e.event_type == *etype))
            linked_ocel
                .e2o_rel_rev
                .get(&items[0])
                .unwrap()
                .into_iter()
                .filter(|(_, e)| {
                    e.event_type == etype
                        && items.iter().skip(1).all(|o| {
                            e.relationships
                                .iter()
                                .any(|r| Into::<ObjectID>::into(&r.object_id) == *o)
                        })
                })
                .map(|x| (&x.1.id).into())
                .collect_vec()
        }
    };
    // println!("Initial is of size: {}",initial.len());
    for o in objs.iter() {
        initial.retain(|e| {
            let obs = linked_ocel
                .e2o_rel
                .get(e)
                .unwrap()
                .iter()
                .map(|o| ObjectID::from(&o.1.id))
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

    pub fn get_for_ev<'a, T: Into<EventID<'a>>>(
        &self,
        ev: T,
        linked_ocel: &'a LinkedOCEL,
    ) -> Vec<ObjectID<'a>> {
        match self {
            ObjectTypeAssociation::Simple { object_type } => linked_ocel
                .get_ev_rels(ev)
                .unwrap()
                .iter()
                .map(|x| x.1)
                .filter(|o| o.object_type == *object_type)
                .map(|o| ObjectID::from(&o.id))
                .collect(),
            ObjectTypeAssociation::O2O {
                first,
                second,
                reversed,
            } => linked_ocel
                .get_ev_rels(ev)
                .unwrap()
                .iter()
                .map(|x| x.1)
                .filter(|o| o.object_type == *first)
                .flat_map(|o| {
                    if !reversed {
                        linked_ocel
                            .o2o_rel
                            .get(&Into::<ObjectID>::into(&o.id))
                            .unwrap()
                            .iter()
                            .map(|rel| rel.1)
                            .filter(|o2| o2.object_type == *second)
                            .collect_vec()
                    } else {
                        linked_ocel
                            .o2o_rel_rev
                            .get(&Into::<ObjectID>::into(&o.id))
                            .unwrap()
                            .iter()
                            .map(|rel| rel.1)
                            .filter(|o2| o2.object_type == *second)
                            .collect_vec()
                    }
                })
                .map(|o| ObjectID::from(&o.id))
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

impl<'a> OCDeclareArcLabel {
    pub fn get_bindings(
        &'a self,
        ev: &'a OCELEvent,
        linked_ocel: &'a LinkedOCEL,
    ) -> impl Iterator<Item = impl Iterator<Item = SetFilter<ObjectID<'a>>>> {
        self.each
            .iter()
            .map(|otass| otass.get_for_ev(&ev.id, linked_ocel))
            .multi_cartesian_product()
            .map(|product| {
                self.all
                    .iter()
                    .map(|otass| SetFilter::All(otass.get_for_ev(&ev.id, linked_ocel)))
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
                            .map(|otass| SetFilter::Any(otass.get_for_ev(&ev.id, linked_ocel))),
                    )
                // .collect_vec()
            })
        // .collect_vec()
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use process_mining::{import_ocel_json_from_path, ocel::linked_ocel::OwnedLinkedOcel};

    use super::*;

    #[test]
    fn it_works() {
        let ocel =
            import_ocel_json_from_path("/home/aarkue/dow/ocel/order-management.json").unwrap();
        // let ocel = import_ocel_json_from_path(
        //     "/home/aarkue/dow/ocel/bpic2017-o2o-workflow-qualifier.json",
        // )
        // .unwrap();
        let linked_ocel: OwnedLinkedOcel = ocel.into();
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
        let all_res = x.get_for_all_evs(&linked_ocel.linked_ocel);
        println!("Took {:?}", now.elapsed());
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
