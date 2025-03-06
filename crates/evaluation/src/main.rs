use std::{fs::File, hint::black_box, time::Instant};

use process_mining::{import_ocel_json_from_path, ocel::linked_ocel::IndexLinkedOCEL};
use serde::{Deserialize, Serialize};
use shared::discovery::{discover, O2OMode};

fn main() {
    let num_runs = 10;
    let noise_thresh = 0.2;
    let event_logs = vec![
        ("Logistics", "/home/aarkue/dow/ocel/ContainerLogistics.json"),
        ("P2P", "/home/aarkue/dow/ocel/ocel2-p2p.json"),
        ("O2C", "/home/aarkue/dow/ocel/order-management.json"),
        (
            "BPIC2017",
            "/home/aarkue/dow/ocel/bpic2017-o2o-workflow-qualifier-index-no-ev-attrs.json",
        ),
    ];
    for (name, path) in event_logs {
        println!("Evaluating on {name}.");
        let ocel = import_ocel_json_from_path(path).unwrap();
        let locel = IndexLinkedOCEL::from_ocel(ocel);
        for o2o_mode in [O2OMode::None, O2OMode::Direct] {
            println!("{:?}", o2o_mode);
            let mut eval_res = EvaluationResult {
                durations_seconds: Vec::new(),
                number_of_results: 0,
                mean_duration: 0.0,
            };
            let mut res = Vec::new();
            for i in 0..num_runs {
                let now = Instant::now();
                res = black_box(discover(&locel, noise_thresh, o2o_mode));
                let duration = now.elapsed();
                eval_res.durations_seconds.push(duration.as_secs_f64());
                if i == 0 {
                    eval_res.number_of_results = res.len();
                } else {
                    assert_eq!(eval_res.number_of_results, res.len());
                }
                println!("Got {} results in {:?}", res.len(), duration);
            }
            eval_res.mean_duration = eval_res.durations_seconds.iter().sum::<f64>() / eval_res.durations_seconds.len() as f64;
            let summary_file = File::create(format!("{}-{:?}-summary.json", name, o2o_mode)).unwrap();
            serde_json::to_writer_pretty(summary_file, &eval_res).unwrap();

            let results_file = File::create(format!("{}-{:?}-results.json", name, o2o_mode)).unwrap();
            serde_json::to_writer_pretty(results_file, &res).unwrap();
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EvaluationResult {
    durations_seconds: Vec<f64>,
    mean_duration: f64,
    number_of_results: usize,
}
