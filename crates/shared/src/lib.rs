pub mod discovery;
pub mod reduction;
pub mod sync_group_discovery;

use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
    usize,
};

use chrono::Duration;
use itertools::Itertools;
pub use process_mining;
use process_mining::{
    ocel::{
        linked_ocel::{
            index_linked_ocel::{EventIndex, ObjectIndex},
            IndexLinkedOCEL, LinkedOCELAccess,
        },
        ocel_struct::{OCELEvent, OCELRelationship, OCELType},
    },
    OCEL,
};

use serde::{Deserialize, Serialize};
const INIT_EVENT_PREFIX: &str = "<init>";
const EXIT_EVENT_PREFIX: &str = "<exit>";
pub fn preprocess_ocel(ocel: OCEL) -> IndexLinkedOCEL {
    let locel: IndexLinkedOCEL = ocel.into();
    let new_evs = locel
        .get_all_obs_ref()
        .flat_map(|obi| {
            let ob = locel.get_ob(obi);
            let iter = locel
                .get_e2o_rev(obi)
                .map(|(_q, e)| locel.get_ev(e).time)
                .sorted();
            let first_ev = iter.clone().next();
            let first_ev_time = first_ev.unwrap_or_default() - Duration::nanoseconds(1);
            let last_ev = iter.clone().last();
            let last_ev_time = last_ev.unwrap_or_default() + Duration::nanoseconds(1);
            
            vec![
                OCELEvent {
                    id: format!("{}_{}_{}", INIT_EVENT_PREFIX, ob.object_type, ob.id),
                    event_type: format!("{} {}", INIT_EVENT_PREFIX, ob.object_type),
                    time: first_ev_time,
                    attributes: Vec::default(),
                    relationships: vec![OCELRelationship {
                        object_id: ob.id.clone(),
                        qualifier: String::from("init"),
                    }],
                },
                OCELEvent {
                    id: format!("{}_{}_{}", EXIT_EVENT_PREFIX, ob.object_type, ob.id),
                    event_type: format!("{} {}", EXIT_EVENT_PREFIX, ob.object_type),
                    time: last_ev_time,
                    attributes: Vec::default(),
                    relationships: vec![OCELRelationship {
                        object_id: ob.id.clone(),
                        qualifier: String::from("exit"),
                    }],
                },
            ]
        })
        .collect_vec();
    let mut ocel = locel.into_inner();
    ocel.event_types
        .extend(ocel.object_types.iter().flat_map(|ot| {
            vec![
                OCELType {
                    name: format!("{} {}", INIT_EVENT_PREFIX, ot.name),
                    attributes: Vec::default(),
                },
                OCELType {
                    name: format!("{} {}", EXIT_EVENT_PREFIX, ot.name),
                    attributes: Vec::default(),
                },
            ]
        }));
    ocel.events.extend(new_evs);
    ocel.into()
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
struct OCDeclareNode(String);

impl<'a> From<&'a OCDeclareNode> for &'a String {
    fn from(val: &'a OCDeclareNode) -> Self {
        &val.0
    }
}

impl OCDeclareNode {
    pub fn new<T: Into<String>>(act: T) -> Self {
        Self(act.into())
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
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

impl OCDeclareArc {
    pub fn clone_with_arc_type(&self, arc_type: OCDeclareArcType) -> Self {
        let mut ret = self.clone();
        ret.arc_type = arc_type;
        ret
    }

    pub fn as_template_string(&self) -> String {
        format!(
            "{}({}, {}, {},{},{})",
            self.arc_type.get_name(),
            self.from.0,
            self.to.0,
            self.label.as_template_string(),
            self.counts.0.unwrap_or_default(),
            self.counts
                .1
                .map(|x| x.to_string())
                .unwrap_or(String::from("∞"))
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "type")]
pub enum ViolationInfo {
    TooMany {
        source_ev: String,
        matching_evs: Vec<String>,
        all_obs: Vec<String>,
        any_obs: Vec<Vec<String>>,
        count: usize,
    },
    TooFew {
        source_ev: String,
        matching_evs: Vec<String>,
        all_obs: Vec<String>,
        any_obs: Vec<Vec<String>>,
        count: usize,
    },
}
use ts_rs::TS;
impl OCDeclareArc {
    pub fn get_for_all_evs(
        &self,
        linked_ocel: &IndexLinkedOCEL,
    ) -> (usize, usize, Vec<(usize, Vec<ViolationInfo>)>) {
        let inner_res: Vec<_> = linked_ocel
            .get_evs_of_type(self.from.as_str())
            // .get(Into::<Cow<_>>::into(&self.from).as_str())
            // .unwrap()
            // .par_bridge()
            // iter()
            .map(|ev| self.get_for_ev(ev, linked_ocel))
            .collect();
        let total_situations = inner_res.iter().map(|e| e.0).sum();
        let total_violations = inner_res.iter().map(|e| e.1.len()).sum();
        (total_situations, total_violations, inner_res)
    }

    pub fn get_for_all_evs_perf(&self, linked_ocel: &IndexLinkedOCEL) -> f64 {
        perf::get_for_all_evs_perf(
            self.from.as_str(),
            self.to.as_str(),
            &self.label,
            &self.arc_type,
            &self.counts,
            linked_ocel,
        )
    }

    pub fn get_for_all_evs_perf_thresh(
        &self,
        linked_ocel: &IndexLinkedOCEL,
        noise_thresh: f64,
    ) -> bool {
        perf::get_for_all_evs_perf_thresh(
            self.from.as_str(),
            self.to.as_str(),
            &self.label,
            &self.arc_type,
            &self.counts,
            linked_ocel,
            noise_thresh,
        )
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
                let evs = get_evs_with_objs(&binding, linked_ocel, self.to.as_str())
                    .into_iter()
                    .filter(|ev2| {
                        let ev2 = linked_ocel.get_ev(ev2);
                        match self.arc_type {
                            OCDeclareArcType::ASS => true,
                            OCDeclareArcType::EF => ev.time < ev2.time,
                            OCDeclareArcType::EFREV => ev.time > ev2.time,
                            _ => todo!("Not implemented yet for non-perf version!"),
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
                        any_obs: binding
                            .iter()
                            .filter_map(|b| match b {
                                // SetFilter::Any(items) => todo!(),
                                SetFilter::Any(items) => Some(
                                    items
                                        .iter()
                                        .map(|o| linked_ocel.get_ob(o).id.clone())
                                        .collect_vec(),
                                ),
                                _ => None,
                            })
                            .collect_vec(),
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
                        any_obs: binding
                            .iter()
                            .filter_map(|b| match b {
                                // SetFilter::Any(items) => todo!(),
                                SetFilter::Any(items) => Some(
                                    items
                                        .iter()
                                        .map(|o| linked_ocel.get_ob(o).id.clone())
                                        .collect(),
                                ),
                                _ => None,
                            })
                            .collect(),
                        count,
                    });
                }
                None

                // (binding,count)

                // binding.len()
            })
            .collect_vec();
        // let num_viol_bindings = res.iter().filter(|o| o.is_some()).count();
        // let num_sat_bindings = res.len() - num_viol_bindings;
        (res.len(), res.into_iter().flatten().collect())
    }
}

fn get_evs_with_objs<'a>(
    objs: &Vec<SetFilter<ObjectIndex>>,
    linked_ocel: &'a IndexLinkedOCEL,
    etype: &'a str,
) -> Vec<EventIndex> {
    let mut initial: Vec<EventIndex> = match &objs[0] {
        SetFilter::Any(items) => {
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
                            let ev = linked_ocel.get_ev(e);
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
            if items.is_empty() {
                return Vec::new();
            }
            linked_ocel
                .get_e2o_rev(&items[0])
                .filter_map(|(_, e)| {
                    let ev = linked_ocel.get_ev(e);
                    if ev.event_type == etype
                        && items
                            .iter()
                            .skip(1)
                            .all(|o| linked_ocel.get_e2o_set(e).contains(o))
                    {
                        Some(*e)
                    } else {
                        None
                    }
                })
                .collect_vec()
        }
    };
    for o in objs.iter() {
        initial.retain(|e| {
            let obs = linked_ocel.get_e2o_set(e);
            o.check(obs)
        });
    }
    initial
}

fn get_evs_with_objs_perf<'a>(
    objs: &'a Vec<SetFilter<ObjectIndex>>,
    linked_ocel: &'a IndexLinkedOCEL,
    etype: &'a str,
) -> impl Iterator<Item = EventIndex> + use<'a> {
    let initial: Box<dyn Iterator<Item = EventIndex>> = match &objs[0] {
        SetFilter::Any(items) => Box::new(
            items
                .iter()
                .flat_map(|o| {
                    linked_ocel
                        .e2o_rev_et
                        .get(etype)
                        .unwrap()
                        .get(o)
                        .into_iter()
                        .flatten()
                        .copied()
                })
                .collect::<HashSet<_>>()
                .into_iter(),
        ),
        SetFilter::All(items) => {
            if items.is_empty() {
                Box::new(Vec::new().into_iter())
            } else {
                Box::new(
                    linked_ocel
                        .e2o_rev_et
                        .get(etype)
                        .unwrap()
                        .get(&items[0])
                        .into_iter()
                        .flatten()
                        .filter(|e| {
                            items
                                .iter()
                                .skip(1)
                                .all(|o| linked_ocel.get_e2o_set(e).contains(o))
                        })
                        .copied(),
                )
            }
        }
    };
    initial.filter(|e| {
        for o in objs.iter() {
            let obs = linked_ocel.get_e2o_set(e);
            if !o.check(obs) {
                return false;
            }
        }
        true
    })
}

fn get_df_or_dp_event_perf<'a>(
    objs: &'a Vec<SetFilter<ObjectIndex>>,
    linked_ocel: &'a IndexLinkedOCEL,
    reference_event_index: &'a EventIndex,
    reference_event: &'a OCELEvent,
    following: bool,
) -> Option<&'a EventIndex> {
    let initial: Box<dyn Iterator<Item = &EventIndex>> = match &objs[0] {
        SetFilter::Any(items) => Box::new(
            items
                .iter()
                .flat_map(|o| {
                    linked_ocel.get_e2o_rev(o).map(|(_q, e)| e).filter(|e| {
                        if following {
                            e > &reference_event_index
                        } else {
                            e < &reference_event_index
                        }
                    })
                })
                .collect::<HashSet<_>>()
                .into_iter(),
        ),
        SetFilter::All(items) => {
            if items.is_empty() {
                Box::new(Vec::new().into_iter())
            } else {
                Box::new(
                    linked_ocel.get_e2o_rev(&items[0]).map(|e| e.1).filter(|e| {
                        items
                            .iter()
                            .skip(1)
                            .all(|o| linked_ocel.get_e2o_set(e).contains(o))
                    }), // .copied()
                )
            }
        }
    };
    let x = initial.filter(|e| {
        if following
            && (e < &reference_event_index || reference_event.time >= linked_ocel.get_ev(e).time)
        {
            return false;
        }
        if !following
            && (e > &reference_event_index || reference_event.time <= linked_ocel.get_ev(e).time)
        {
            return false;
        }
        for o in objs.iter() {
            let obs = linked_ocel.get_e2o_set(e);
            if !o.check(obs) {
                return false;
            }
        }
        true
    });
    match following {
        true => x.min(),
        false => x.max(),
    }
}




fn get_alternate_ef_ep_event_perf<'a>(
    objs: &'a Vec<SetFilter<ObjectIndex>>,
    linked_ocel: &'a IndexLinkedOCEL,
    reference_event_index: &'a EventIndex,
    reference_event: &'a OCELEvent,
    following: bool,
    alternate_ev_type: &str,
    to_ev_type: &str,
) -> usize {
    let initial: Box<dyn Iterator<Item = &EventIndex>> = match &objs[0] {
        SetFilter::Any(items) => Box::new(
            items
                .iter()
                .flat_map(|o| {
                    linked_ocel.get_e2o_rev(o).map(|(_q, e)| e).filter(|e| {
                        if following {
                            e > &reference_event_index
                        } else {
                            e < &reference_event_index
                        }
                    })
                })
                .collect::<HashSet<_>>()
                .into_iter(),
        ),
        SetFilter::All(items) => {
            if items.is_empty() {
                Box::new(Vec::new().into_iter())
            } else {
                Box::new(
                    linked_ocel.get_e2o_rev(&items[0]).map(|e| e.1).filter(|e| {
                        items
                            .iter()
                            .skip(1)
                            .all(|o| linked_ocel.get_e2o_set(e).contains(o))
                    }), // .copied()
                )
            }
        }
    };
    let mut x = initial.filter(|e| {
        if following
            && (e <= &reference_event_index) // || (*e != reference_event_index && reference_event.time >= linked_ocel.get_ev(e).time)
        {
            return false;
        }
        if !following
            && (e >= &reference_event_index) // || (*e != reference_event_index && reference_event.time <= linked_ocel.get_ev(e).time)
        {
            return false;
        }
        for o in objs.iter() {
            let obs = linked_ocel.get_e2o_set(e);
            if !o.check(obs) {
                return false;
            }
        }
        true
    }).sorted().collect_vec();
    if !following {
       x.reverse();
    }
    let mut count = 0;
    for e in x {
        let e = linked_ocel.get_ev(e);
        if e.event_type == *alternate_ev_type {
            break;
        }else{
           if e.event_type == to_ev_type {
            count += 1;
           }
        }
    }
    count
}


#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
// #[serde(tag = "type")]
pub enum OCDeclareArcType {
    ASS,
    EF,
    EFREV,
    DF,
    DFREV,
    ALTEF,
    ALTEFREV,
}

impl OCDeclareArcType {
    pub fn get_name(&self) -> &'static str {
        match self {
            OCDeclareArcType::ASS => "AS",
            OCDeclareArcType::EF => "EF",
            OCDeclareArcType::EFREV => "EP",
            OCDeclareArcType::DF => "DF",
            OCDeclareArcType::DFREV => "DP",
            OCDeclareArcType::ALTEF => "AF",
            OCDeclareArcType::ALTEFREV => "AP",
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord, TS)]
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

    pub fn as_template_string(&self) -> String {
        match self {
            ObjectTypeAssociation::Simple { object_type } => object_type.clone(),
            ObjectTypeAssociation::O2O {
                first,
                second,
                reversed,
            } => format!("{}{}{}", first, if !reversed { ">" } else { "<" }, second),
        }
    }

    pub fn get_for_ev(&self, ev: &EventIndex, linked_ocel: &IndexLinkedOCEL) -> Vec<ObjectIndex> {
        match self {
            ObjectTypeAssociation::Simple { object_type } => linked_ocel
                .get_e2o_set(ev)
                // .map(|x| x.1)
                .iter()
                .filter_map(|o| {
                    let ob = linked_ocel.get_ob(o);
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
                .get_e2o_set(ev)
                // .unwrap()
                .iter()
                // .map(|x| x.1)
                .filter(|o| linked_ocel.get_ob(o).object_type == *first)
                .flat_map(|o| {
                    if !reversed {
                        linked_ocel
                            .get_o2o(o)
                            // .get(&Into::<ObjectID>::into(&o.id))
                            // .unwrap()
                            // .iter()
                            .map(|rel| rel.1)
                            .filter(|o2| linked_ocel.get_ob(o2).object_type == *second)
                            .collect_vec()
                    } else {
                        linked_ocel
                            .get_o2o_rev(o)
                            // .get(&Into::<ObjectID>::into(&o.id))
                            // .unwrap()
                            // .iter()
                            .map(|rel| rel.1)
                            .filter(|o2| linked_ocel.get_ob(o2).object_type == *second)
                            .collect_vec()
                    }
                })
                .copied()
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default, Hash, TS)]
#[ts(export)]
pub struct OCDeclareArcLabel {
    each: Vec<ObjectTypeAssociation>,
    any: Vec<ObjectTypeAssociation>,
    all: Vec<ObjectTypeAssociation>,
}

impl OCDeclareArcLabel {
    pub fn as_template_string(&self) -> String {
        let mut ret = String::new();
        if !self.each.is_empty() {
            ret.push_str(&format!(
                "Each({})",
                self.each.iter().map(|ot| ot.as_template_string()).join(",")
            ));
        }
        if !self.all.is_empty() {
            if !self.each.is_empty() {
                ret.push_str(", ");
            }
            ret.push_str(&format!(
                "All({})",
                self.all.iter().map(|ot| ot.as_template_string()).join(",")
            ));
        }
        if !self.any.is_empty() {
            if !self.each.is_empty() || !self.any.is_empty() {
                ret.push_str(", ");
            }
            ret.push_str(&format!(
                "Any({})",
                self.any.iter().map(|ot| ot.as_template_string()).join(",")
            ));
        }
        ret
    }
}

// fn get_out_types<'a>(ras: &'a HashSet<ObjectTypeAssociation>) -> impl Iterator<Item = &'a String> {
//     ras.iter().filter_map(|oas| match oas {
//         ObjectTypeAssociation::Simple { object_type } => Some(object_type),
//         ObjectTypeAssociation::O2O {
//             first,
//             second,
//             reversed,
//         } => None,
//     })
// }
impl OCDeclareArcLabel {
    pub fn combine(&self, other: &Self) -> Self {
        let all = self
            .all
            .iter()
            .chain(other.all.iter())
            .cloned()
            .collect::<HashSet<_>>();
        let each = self
            .each
            .iter()
            .chain(other.each.iter())
            .filter(|e| !all.contains(e))
            .cloned()
            .collect::<HashSet<_>>();
        let any = self
            .any
            .iter()
            .chain(other.any.iter())
            .filter(|e| !all.contains(e) && !each.contains(e))
            .cloned()
            .collect::<HashSet<_>>();
        Self {
            each: each.into_iter().sorted().collect(),
            all: all.into_iter().sorted().collect(),
            any: any.into_iter().sorted().collect(),
        }
    }

    pub fn intersect(&self, other: &Self) -> Self {
        let all: Vec<ObjectTypeAssociation> = self
            .all
            .iter()
            .filter(|oi| other.all.contains(&oi))
            .cloned()
            .collect();
        let each: Vec<ObjectTypeAssociation> = self
            .each
            .iter()
            .chain(self.all.iter())
            .filter(|oi| !all.contains(oi))
            .filter(|oi| other.all.contains(&oi) || other.each.contains(&oi))
            .cloned()
            .collect();
        let any: Vec<ObjectTypeAssociation> = self
            .any
            .iter()
            .chain(self.each.iter())
            .chain(self.all.iter())
            .filter(|oi| !all.contains(oi) && !each.contains(oi))
            .filter(|oi| {
                other.all.contains(&oi) || other.each.contains(&oi) || other.any.contains(&oi)
            })
            .cloned()
            .collect();
        Self { each, all, any }
    }

    pub fn is_dominated_by(&self, other: &Self) -> bool {
        let all_all = self.all.iter().all(|a| other.all.contains(a));
        if !all_all {
            return false;
        }
        let all_each = self
            .each
            .iter()
            .all(|a| other.each.contains(a) || other.all.contains(a));
        if !all_each {
            return false;
        }
        let all_any = self
            .any
            .iter()
            .all(|a| other.any.contains(a) || other.each.contains(a) || other.all.contains(a));
        all_any
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SetFilter<T: Eq + Hash> {
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

impl<'b> OCDeclareArcLabel {
    pub fn get_bindings<'a>(
        &'a self,
        ev: &'a EventIndex,
        linked_ocel: &'a IndexLinkedOCEL,
    ) -> impl Iterator<Item = Vec<SetFilter<ObjectIndex>>> + use<'a, 'b> {
        self.each
            .iter()
            .map(|otass| otass.get_for_ev(ev, linked_ocel))
            .multi_cartesian_product()
            .map(|product| {
                self.all
                    .iter()
                    .map(|otass| SetFilter::All(otass.get_for_ev(ev, linked_ocel)))
                    .chain(if product.is_empty() {
                        Vec::default()
                    } else {
                        vec![SetFilter::All(product)]
                    })
                    .chain(
                        self.any
                            .iter()
                            .sorted_by_cached_key(|ot| match ot {
                                ObjectTypeAssociation::Simple { object_type } => {
                                    -(linked_ocel.get_obs_of_type(object_type).count() as i32)
                                }
                                ObjectTypeAssociation::O2O { second, .. } => {
                                    -(linked_ocel.get_obs_of_type(second).count() as i32)
                                }
                            })
                            .map(|otass| {
                                let x = otass.get_for_ev(ev, linked_ocel);
                                if x.len() == 1 {
                                    SetFilter::All(x)
                                } else {
                                    SetFilter::Any(x)
                                }
                            }),
                    )
                    .collect_vec()
            })
        // .collect_vec()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ObjectInvolvementCounts {
    min: usize,
    max: usize,
    // mean: usize,
}
impl Default for ObjectInvolvementCounts {
    fn default() -> Self {
        Self {
            min: usize::MAX,
            max: Default::default(),
        }
    }
}

pub fn get_activity_object_involvements(
    locel: &IndexLinkedOCEL,
) -> HashMap<String, HashMap<String, ObjectInvolvementCounts>> {
    locel
        .get_ev_types()
        .map(|et| {
            let mut nums_of_objects_per_type: HashMap<String, ObjectInvolvementCounts> = locel
                .get_ob_types()
                .map(|ot| (ot.to_string(), ObjectInvolvementCounts::default()))
                .collect();
            for ev in locel.get_evs_of_type(et) {
                let mut num_of_objects_for_ev: HashMap<&str, usize> = HashMap::new();
                for (_q, oi) in locel.get_e2o(ev) {
                    let o = locel.get_ob(oi);
                    *num_of_objects_for_ev.entry(&o.object_type).or_default() += 1;
                }
                for (ot, count) in num_of_objects_for_ev {
                    let num_ob_per_type = nums_of_objects_per_type.get_mut(ot).unwrap();

                    if count < num_ob_per_type.min {
                        num_ob_per_type.min = count
                    }
                    if count > num_ob_per_type.max {
                        num_ob_per_type.max = count;
                    }
                }
            }
            (
                et.to_string(),
                nums_of_objects_per_type
                    .into_iter()
                    .filter(|(_x, y)| y.max > 0)
                    .collect(),
            )
            // (nums_of_objects_per_type
        })
        .collect()
}

pub fn get_object_to_object_involvements(
    locel: &IndexLinkedOCEL,
) -> HashMap<String, HashMap<String, ObjectInvolvementCounts>> {
    locel
        .get_ob_types()
        .map(|ot| {
            let mut nums_of_objects_per_type: HashMap<String, ObjectInvolvementCounts> = locel
                .get_ob_types()
                .map(|ot| (ot.to_string(), ObjectInvolvementCounts::default()))
                .collect();
            for ob in locel.get_obs_of_type(ot) {
                let mut num_of_objects_for_ob: HashMap<&str, usize> = HashMap::new();
                for (_q, oi) in locel.get_o2o(ob) {
                    let o = locel.get_ob(oi);
                    *num_of_objects_for_ob.entry(&o.object_type).or_default() += 1;
                }
                for (ot, count) in num_of_objects_for_ob {
                    let num_ob_per_type = nums_of_objects_per_type.get_mut(ot).unwrap();

                    if count < num_ob_per_type.min {
                        num_ob_per_type.min = count
                    }
                    if count > num_ob_per_type.max {
                        num_ob_per_type.max = count;
                    }
                }
            }
            (
                ot.to_string(),
                nums_of_objects_per_type
                    .into_iter()
                    .filter(|(_x, y)| y.max > 0)
                    .collect(),
            )
            // (nums_of_objects_per_type
        })
        .collect()
}

pub fn get_rev_object_to_object_involvements(
    locel: &IndexLinkedOCEL,
) -> HashMap<String, HashMap<String, ObjectInvolvementCounts>> {
    locel
        .get_ob_types()
        .map(|ot| {
            let mut nums_of_objects_per_type: HashMap<String, ObjectInvolvementCounts> = locel
                .get_ob_types()
                .map(|ot| (ot.to_string(), ObjectInvolvementCounts::default()))
                .collect();
            for ob in locel.get_obs_of_type(ot) {
                let mut num_of_objects_for_ob: HashMap<&str, usize> = HashMap::new();
                for (_q, oi) in locel.get_o2o_rev(ob) {
                    let o = locel.get_ob(oi);
                    *num_of_objects_for_ob.entry(&o.object_type).or_default() += 1;
                }
                for (ot, count) in num_of_objects_for_ob {
                    let num_ob_per_type = nums_of_objects_per_type.get_mut(ot).unwrap();

                    if count < num_ob_per_type.min {
                        num_ob_per_type.min = count
                    }
                    if count > num_ob_per_type.max {
                        num_ob_per_type.max = count;
                    }
                }
            }
            (
                ot.to_string(),
                nums_of_objects_per_type
                    .into_iter()
                    .filter(|(_x, y)| y.max > 0)
                    .collect(),
            )
            // (nums_of_objects_per_type
        })
        .collect()
}

pub mod perf {
    use std::sync::atomic::AtomicI32;

    use process_mining::ocel::linked_ocel::{
        index_linked_ocel::EventIndex, IndexLinkedOCEL, LinkedOCELAccess,
    };
    use rayon::prelude::*;

    use crate::{
        get_alternate_ef_ep_event_perf, get_df_or_dp_event_perf, get_evs_with_objs_perf, OCDeclareArcLabel, OCDeclareArcType
    };

    pub fn get_for_all_evs_perf(
        from_et: &str,
        to_et: &str,
        label: &OCDeclareArcLabel,
        arc_type: &OCDeclareArcType,
        counts: &(Option<usize>, Option<usize>),
        linked_ocel: &IndexLinkedOCEL,
    ) -> f64 {
        let evs = linked_ocel.events_per_type.get(from_et).unwrap();
        let ev_count = evs.len();
        let violated_evs_count = evs
            .into_par_iter()
            // .into_iter()
            .filter(|ev| get_for_ev_perf(ev, label, to_et, arc_type, counts, linked_ocel))
            .count();
        violated_evs_count as f64 / ev_count as f64
    }

    pub fn get_for_all_evs_perf_thresh(
        from_et: &str,
        to_et: &str,
        label: &OCDeclareArcLabel,
        arc_type: &OCDeclareArcType,
        counts: &(Option<usize>, Option<usize>),
        linked_ocel: &IndexLinkedOCEL,
        violation_thresh: f64,
    ) -> bool {
        let evs = linked_ocel.events_per_type.get(from_et).unwrap();
        let ev_count = evs.len();
        let min_s = (ev_count as f64 * (1.0 - violation_thresh)).ceil() as usize;
        let min_v = (ev_count as f64 * violation_thresh).floor() as usize + 1;
        // // Non-Atomic:
        // for ev in evs {
        //     let violated = get_for_ev_perf(ev, label, to_et, arc_type, counts, linked_ocel);
        //     if violated {
        //         min_v -= 1;
        //         if min_v == 0 {
        //             return false;
        //         }
        //     } else {
        //         min_s -= 1;
        //         if min_s == 0 {
        //             return true;
        //         }
        //     }
        // }
        // if min_s <= 0 {
        //     return true;
        // }
        // if min_v <= 0 {
        //     return false;
        // }

        // Atomic:
        let min_v_atomic = AtomicI32::new(min_v as i32);
        let min_s_atomic = AtomicI32::new(min_s as i32);
        evs.into_par_iter()
            .map(|ev| {
                let violated = get_for_ev_perf(ev, label, to_et, arc_type, counts, linked_ocel);
                if violated {
                    min_v_atomic.fetch_add(-1, std::sync::atomic::Ordering::Relaxed);
                } else {
                    min_s_atomic.fetch_add(-1, std::sync::atomic::Ordering::Relaxed);
                }
                ev
            })
            .take_any_while(|_x| {
                if min_s_atomic.load(std::sync::atomic::Ordering::Relaxed) <= 0 {
                    return false;
                }
                if min_v_atomic.load(std::sync::atomic::Ordering::Relaxed) <= 0 {
                    return false;
                }
                true
            })
            .count();
        let min_s_atomic = min_s_atomic.into_inner();
        let min_v_atomic = min_v_atomic.into_inner();
        // println!("{} and {}",min_s_atomic,min_v_atomic);
        if min_s_atomic <= 0 {
            return true;
        }
        if min_v_atomic <= 0 {
            return false;
        }

        unreachable!()

        // println!("{} and {} of {} (min_s: {}, min_v: {})",min_s_atomic,min_v_atomic,ev_count,min_s,min_v);
        // true

        // Previous:
        // let violated_evs_count =
        // evs
        //     .into_par_iter()
        //     // .into_iter()
        //     .filter(|ev| get_for_ev_perf(ev, label, to_et, arc_type, counts, linked_ocel))
        //     // .take_any(min_v)
        //     .take_any(min_s)
        //     .count();
        // violated_evs_count < min_v
        // // sat_evs_count >= min_s
    }

    /// Returns true if violated!
    pub fn get_for_ev_perf<'a>(
        ev_index: &EventIndex,
        label: &OCDeclareArcLabel,
        to_et: &str,
        arc_type: &OCDeclareArcType,
        counts: &(Option<usize>, Option<usize>),
        linked_ocel: &IndexLinkedOCEL,
    ) -> bool {
        let ev = linked_ocel.get_ev(ev_index);
        label.get_bindings(ev_index, linked_ocel).any(|binding| {
            let binding = binding;
            match arc_type {
                OCDeclareArcType::ASS | OCDeclareArcType::EF | OCDeclareArcType::EFREV => {
                    let target_ev_iterator = get_evs_with_objs_perf(&binding, linked_ocel, to_et)
                        .filter(|ev2| {
                            // let ev2 = linked_ocel.get_ev(ev2);
                            match arc_type {
                                OCDeclareArcType::EF => ev_index < ev2,
                                OCDeclareArcType::EFREV => ev_index > ev2,
                                OCDeclareArcType::ASS => true,
                                // OCDeclareArcType::EF => ev.time < ev2.time,
                                // OCDeclareArcType::EFREV => ev.time > ev2.time,
                                _ => unreachable!("DF should not go here."),
                            }
                        });
                    if counts.1.is_none() {
                        // Only take necessary
                        // ev_count.
                        if counts.0.unwrap_or_default()
                            > target_ev_iterator
                                .take(counts.0.unwrap_or_default())
                                .count()
                        {
                            // Violated!
                            return true;
                        }
                    } else if let Some(c) = counts.1 {
                        let count = target_ev_iterator.take(c + 1).count();
                        if c < count || count < counts.0.unwrap_or_default() {
                            // Violated
                            return true;
                        }
                    }
                    false
                }
                OCDeclareArcType::DF | OCDeclareArcType::DFREV => {
                    let df_ev = get_df_or_dp_event_perf(
                        &binding,
                        linked_ocel,
                        ev_index,
                        ev,
                        arc_type == &OCDeclareArcType::DF,
                    );
                    let count = if df_ev.is_some_and(|e| linked_ocel.get_ev(e).event_type == to_et)
                    {
                        1
                    } else {
                        0
                    };
                    if let Some(min_c) = counts.0 {
                        if count < min_c {
                            return true;
                        }
                    }
                    if let Some(max_c) = counts.1 {
                        if count > max_c {
                            return true;
                        }
                    }
                    false
                },
                OCDeclareArcType::ALTEF | OCDeclareArcType::ALTEFREV => {
                    let chain_counts = get_alternate_ef_ep_event_perf(
                        &binding,
                        linked_ocel,
                        ev_index,
                        ev,
                        arc_type == &OCDeclareArcType::ALTEF,
                        &ev.event_type,
                        to_et,
                        // if arc_type == &OCDeclareArcType::CHAINEFREV { &ev.event_type } else {to_et}
                    );
                    return !(counts.0.is_none_or(|c| chain_counts >= c) && counts.1.is_none_or(|c| chain_counts <= c))
                }
            }
        })
    }
}
