use crate::{OCDeclareArc, OCDeclareArcType};

pub fn perform_transitive_reduction(
    candidates: &Vec<OCDeclareArc>,
) -> Vec<OCDeclareArc> {
    let mut ret: Vec<OCDeclareArc> = candidates.clone();
    for e1 in candidates {
        for e2 in candidates {
            if  e1.to == e2.from {
                // So we have a1 -l1> b1 -l2> b2
                // Remove all a1 -l3> b2, where l3 <= l1  and l3 <= l2
                ret.retain(|e3| {
                    let remove = e3.arc_type.is_dominated_by_or_eq(&e1.arc_type) && e3.arc_type.is_dominated_by_or_eq(&e2.arc_type) && e3.from == e1.from
                        && e3.to == e2.to
                        && (e3.label.is_dominated_by(&e1.label) && e3.label.is_dominated_by(&e2.label));
                    !remove
                })
            }
        }
    }

    ret
}


pub fn reduce_oc_arcs(arcs: &Vec<OCDeclareArc>) -> Vec<OCDeclareArc> {
    let mut ret = arcs.clone();

    for a in arcs {

        //         ret.retain(|b| {

        //     let remove = a.from == b.to && a.to == b.from && a.arc_type != b.arc_type  && b.label.is_dominated_by(&a.label) && !a.label.is_dominated_by(&b.label);

        //     !remove

        // });

        //         ret.retain(|b| {

        //     let remove = a.from == b.to && a.to == b.from && a.arc_type != b.arc_type && a.arc_type == OCDeclareArcType::EF  && b.label.is_dominated_by(&a.label);

        //     !remove

        // });


        for b in arcs {
            if a.from != a.to && b.from == a.to && a.from != b.to {

                ret.retain(|c| {
                    let remove = c.from == a.from
                        && c.to == b.to
                        && c.arc_type.is_dominated_by_or_eq(&a.arc_type) && c.arc_type.is_dominated_by_or_eq(&b.arc_type)
                        && (c.label.is_dominated_by(&a.label) && c.label.is_dominated_by(&b.label));

                    let is_strictly_dominated = c.label.any.iter().all(|any_label| {
                        let x = if c.arc_type == OCDeclareArcType::EF {
                            !b.label.any.iter().any(|l| l == any_label)
                        } else {
                            !a.label.any.iter().any(|l| l == any_label)
                        };

                        x
                    });


                    !remove || !is_strictly_dominated
                })
            }
        }
    }

    ret
}


#[cfg(test)]
mod tests
{
    use std::fs::File;

    use process_mining::{import_ocel_json_from_path, object_centric::oc_declare::OCDeclareDiscoveryOptions};

    use crate::{discover_behavior_constraints, preprocess_ocel, reduction::reduce_oc_arcs};


    
    #[test]
    fn transitive_reduction(){
        let ocel = import_ocel_json_from_path("/home/aarkue/dow/ocel/bpic2017-o2o-workflow-qualifier-index-no-ev-attrs-sm.json").unwrap();
        let locel = preprocess_ocel(ocel);
        let res = discover_behavior_constraints(&locel, OCDeclareDiscoveryOptions::default());
        let filtered_res = reduce_oc_arcs(&res);
        serde_json::to_writer(File::create("bpic2017-reduced-oc-DECLARE.json").unwrap(), &filtered_res).unwrap();
        println!("Reduced {} to {}",res.len(),filtered_res.len());
    }
}