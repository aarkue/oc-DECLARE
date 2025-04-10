use crate::OCDeclareArc;

pub fn reduce_oc_arcs(arcs: Vec<OCDeclareArc>) -> Vec<OCDeclareArc> {
    let mut ret = arcs.clone();
    for a in &arcs {
        for b in &arcs {
            // if a.from.as_str() != "place order" && a.to.as_str() != "confirm order" && b.to.as_str() != "create package" {
            //     continue;
            // }
            if a.from != a.to && b.from == a.to && a.from != b.to && a.arc_type == b.arc_type {
                // Remove a.from -> b.to (IF object involvement + arc type match)
                let a_dominates_b = b.label.is_dominated_by(&a.label);
                let b_dominates_a = a.label.is_dominated_by(&b.label);
                if a_dominates_b || b_dominates_a || a.label == b.label {
                    ret.retain(|c| {
                        let remove = c.from == a.from
                        && c.to == b.to
                        && c.arc_type == a.arc_type
                        && (c.label == a.label || c.label == b.label || if a_dominates_b {
                            c.label.is_dominated_by(&b.label)
                        } else {
                            c.label.is_dominated_by(&a.label)
                        });
                        !remove
                    })
                }
            }
        }
    }
    ret
}

#[cfg(test)]
mod test {
    use process_mining::{import_ocel_json_from_path, ocel::linked_ocel::IndexLinkedOCEL};

    use crate::discovery::discover;


    #[test]
    fn test_reduction() {
        let ocel = import_ocel_json_from_path("/home/aarkue/dow/ocel/order-management.json").unwrap();
        // let ocel = import_ocel_json_from_path("/home/aarkue/dow/ocel/bpic2017-o2o-workflow-qualifier-index-no-ev-attrs-sm.json").unwrap();
        let locel = IndexLinkedOCEL::from(ocel);
        let res = discover(&locel, 0.2, crate::discovery::O2OMode::None);
        println!("Discovered {} constraints",res.len());
        let results_file =
            std::fs::File::create(format!("reduced.json")).unwrap();
        serde_json::to_writer_pretty(results_file, &res).unwrap();
    }
}