use std::{collections::HashSet, hash::Hash};

use itertools::{Itertools, MultiProduct};
use process_mining::ocel::{
    linked_ocel::{EventID, LinkedOCEL, ObjectID},
    ocel_struct::OCELEvent,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
enum OCDeclareNode {
    Activity(String),
    ObjectInit(String),
    ObjectEnd(String),
}

impl<'a> Into<&'a str> for &'a OCDeclareNode {
    fn into(self) -> &'a str {
        match self {
            OCDeclareNode::Activity(a) => a.as_str(),
            OCDeclareNode::ObjectInit(_) => todo!(),
            OCDeclareNode::ObjectEnd(_) => todo!(),
        }
    }
}

impl OCDeclareNode {
    pub fn new_act<T: Into<String>>(act: T) -> Self {
        Self::Activity(act.into())
    }

    pub fn new_ob_init<T: Into<String>>(ob_type: T) -> Self {
        Self::ObjectInit(ob_type.into())
    }
    pub fn new_ob_end<T: Into<String>>(ob_type: T) -> Self {
        Self::ObjectEnd(ob_type.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct OCDeclareArc {
    from: OCDeclareNode,
    to: OCDeclareNode,
    arc_type: OCDeclareArcType,
    label: OCDeclareArcLabel,
}
// use rayon::prelude::*;
impl<'a> OCDeclareArc {
    pub fn get_for_all_evs(&self,linked_ocel:&LinkedOCEL<'_>) -> Vec<Vec<usize>> {
        linked_ocel.events_per_type.get(Into::<&str>::into(&self.from)).unwrap().
        // par_iter()
        iter()
        .map(|ev| {
            self.get_for_ev(ev, linked_ocel)
        }).collect()
    }
    pub fn get_for_ev(&self, ev: &'a OCELEvent, linked_ocel:&LinkedOCEL<'_> ) -> Vec<usize> {
        self.label.get_bindings(ev, linked_ocel).map(|binding| {
            let binding = binding.collect_vec();
            // Now get the number of events fulfilling this binding criteria
            // linked_ocel.events_per_type.get(Into::<&str>::into(&self.to)).unwrap().iter().filter(|e| {
            //     let obs: HashSet<ObjectID> = linked_ocel.get_ev_rels(&e.id).unwrap().iter().map(|x| (&x.1.id).into()).collect();
            //     // println!("{:?}",obs);
            //     binding.iter().all(|b| b.check(&obs))
            // false
            // }).count()
            get_evs_with_objs(&binding,linked_ocel,Into::<&str>::into(&self.to)).len()
            // binding.len()
        }).collect_vec()
    }
}

fn get_evs_with_objs<'a>(objs: &Vec<SetFilter<ObjectID<'_>>>,  linked_ocel:&'a LinkedOCEL<'_>, etype: &'a str) -> Vec<EventID<'a>> {
    let mut initial: Vec<EventID> = match &objs[0] {
        SetFilter::Any(_items) => linked_ocel.events_per_type.get(etype).unwrap().iter().map(|e| (&e.id).into()).collect(),
        SetFilter::All(items) => linked_ocel.e2o_rel_rev.get(&items[0]).unwrap().into_iter().filter(|(_,e)| items.iter().skip(1).all(|o| e.relationships.iter().any(|r| Into::<ObjectID>::into(&r.object_id) == *o))).map(|x| (&x.1.id).into()).collect_vec(),
    };
    for o in objs.iter() {
        initial.retain(|e| {
            let obs = linked_ocel.e2o_rel.get(e).unwrap().iter().map(|o| ObjectID::from(&o.1.id)).collect();
            o.check(&obs)
        });
    }
    initial
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
enum OCDeclareArcType {
    ASS,
    EF,
    EFREV,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
enum ObjectTypeAssociation {
    Simple(String),
    O2O {
        first: String,
        second: String,
        reversed: bool,
    },
}

impl ObjectTypeAssociation {
    pub fn new_simple<T: Into<String>>(ot: T) -> Self {
        Self::Simple(ot.into())
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
            ObjectTypeAssociation::Simple(ot) => linked_ocel
                .get_ev_rels(ev)
                .unwrap()
                .iter()
                .map(|x| x.1)
                .filter(|o| o.object_type == *ot)
                .map(|o| ObjectID::from(&o.id))
                .collect(),
            ObjectTypeAssociation::O2O {
                first,
                second,
                reversed,
            } => todo!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
struct OCDeclareArcLabel {
    each: Vec<ObjectTypeAssociation>,
    any: Vec<ObjectTypeAssociation>,
    all: Vec<ObjectTypeAssociation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
                    .chain(vec![SetFilter::All(product)].into_iter())
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
        let linked_ocel: OwnedLinkedOcel = ocel.into();
        let x = OCDeclareArc {
            from: OCDeclareNode::new_act("place order"),
            to: OCDeclareNode::new_act("payment reminder"),
            arc_type: OCDeclareArcType::EF,
            label: OCDeclareArcLabel {
                each: vec![ObjectTypeAssociation::new_simple("orders")],
                ..Default::default()
            },
        };
        // let ev = linked_ocel.ocel_ref().events.iter().filter(|e| e.event_type == "place order").next().unwrap();
        // let res = x.get_for_ev(ev, &linked_ocel.linked_ocel);
        // println!("{:?}",res);


        let now = Instant::now();
        let all_res = x.get_for_all_evs(&linked_ocel.linked_ocel);
        println!("Took {:?}", now.elapsed());
        println!("{:?}",all_res.iter().take(10).collect_vec());

        // for y in x.label.get_bindings(linked_ocel.ocel_ref().events.iter().filter(|e| e.event_type == "confirm order").next().unwrap(), &linked_ocel.linked_ocel) {
        //     for yy in y {
        //         println!("{:?}",yy);
        //     }
        // }
        println!("{:?}", x);
    }
}
