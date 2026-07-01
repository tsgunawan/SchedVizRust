use schedviz_rust::scheduler_core::{
    Process, run_selected, schedule_rr, schedule_srt, validate_processes,
};

fn make_example_1() -> Vec<Process> {
    vec![
        Process::new("A", 0.0, 5.0, 0),
        Process::new("B", 2.0, 3.0, 1),
        Process::new("C", 5.0, 7.0, 2),
    ]
}

#[test]
fn test_example_1_finish_times_all_algorithms() {
    let processes = make_example_1();
    let algos = vec![
        "FCFS".to_string(),
        "RR".to_string(),
        "SPN".to_string(),
        "SRT".to_string(),
        "HRRN".to_string(),
    ];
    let results = run_selected(&processes, &algos, 1).unwrap();

    let get_finish_times = |name: &str| {
        let result = results.get(name).unwrap();
        result
            .metrics
            .iter()
            .map(|(k, v)| (k.clone(), v.finish_time))
            .collect::<std::collections::HashMap<String, f64>>()
    };

    let fcfs = get_finish_times("FCFS");
    assert_eq!(fcfs["A"], 5.0);
    assert_eq!(fcfs["B"], 8.0);
    assert_eq!(fcfs["C"], 15.0);

    let rr = get_finish_times("RR q=1");
    assert_eq!(rr["A"], 9.0);
    assert_eq!(rr["B"], 8.0);
    assert_eq!(rr["C"], 15.0);

    let spn = get_finish_times("SPN");
    assert_eq!(spn["A"], 5.0);
    assert_eq!(spn["B"], 8.0);
    assert_eq!(spn["C"], 15.0);

    let srt = get_finish_times("SRT");
    assert_eq!(srt["A"], 5.0);
    assert_eq!(srt["B"], 8.0);
    assert_eq!(srt["C"], 15.0);

    let hrrn = get_finish_times("HRRN");
    assert_eq!(hrrn["A"], 5.0);
    assert_eq!(hrrn["B"], 8.0);
    assert_eq!(hrrn["C"], 15.0);
}

#[test]
fn test_example_1_average_normalized_turnaround_times() {
    let processes = make_example_1();
    let algos = vec![
        "FCFS".to_string(),
        "RR".to_string(),
        "SPN".to_string(),
        "SRT".to_string(),
        "HRRN".to_string(),
    ];
    let results = run_selected(&processes, &algos, 1).unwrap();

    let check_approx = |val: f64, expected: f64| {
        assert!(
            (val - expected).abs() < 0.01,
            "Expected approx {}, got {}",
            expected,
            val
        );
    };

    check_approx(results["FCFS"].average_normalized_turnaround_time, 1.48);
    check_approx(results["RR q=1"].average_normalized_turnaround_time, 1.74);
    check_approx(results["SPN"].average_normalized_turnaround_time, 1.48);
    check_approx(results["SRT"].average_normalized_turnaround_time, 1.48);
    check_approx(results["HRRN"].average_normalized_turnaround_time, 1.48);
}

#[test]
fn test_round_robin_same_time_arrival_before_expired_process() {
    let processes = vec![
        Process::new("A", 0.0, 3.0, 0),
        Process::new("B", 1.0, 1.0, 1),
    ];
    let result = schedule_rr(&processes, 1).unwrap();

    let slice_tuples: Vec<(String, f64, f64)> = result
        .slices
        .iter()
        .map(|s| (s.process.clone(), s.start, s.end))
        .collect();

    assert_eq!(
        slice_tuples,
        vec![
            ("A".to_string(), 0.0, 1.0),
            ("B".to_string(), 1.0, 2.0),
            ("A".to_string(), 2.0, 4.0),
        ]
    );
}

#[test]
fn test_srt_equal_remaining_time_keeps_current_process() {
    let processes = vec![
        Process::new("A", 0.0, 4.0, 0),
        Process::new("B", 2.0, 2.0, 1),
    ];
    let result = schedule_srt(&processes).unwrap();

    let slice_tuples: Vec<(String, f64, f64)> = result
        .slices
        .iter()
        .map(|s| (s.process.clone(), s.start, s.end))
        .collect();

    assert_eq!(
        slice_tuples,
        vec![("A".to_string(), 0.0, 4.0), ("B".to_string(), 4.0, 6.0),]
    );
}

#[test]
fn test_idle_gap_is_skipped_in_timeline_time() {
    let processes = vec![Process::new("A", 3.0, 2.0, 0)];
    let results = run_selected(&processes, &["FCFS".to_string()], 1).unwrap();
    let result = &results["FCFS"];

    assert_eq!(result.slices[0].start, 3.0);
    assert_eq!(result.slices[0].end, 5.0);
    assert_eq!(result.total_completion_time, 5.0);
}

#[test]
fn test_validation_errors() {
    let p1 = vec![Process::new("A", -1.0, 1.0, 0)];
    assert!(validate_processes(&p1).is_err());

    let p2 = vec![Process::new("A", 0.0, 0.0, 0)];
    assert!(validate_processes(&p2).is_err());

    let p3 = vec![
        Process::new("A", 0.0, 1.0, 0),
        Process::new("A", 1.0, 1.0, 1),
    ];
    assert!(validate_processes(&p3).is_err());
}

#[test]
fn test_round_robin_quantum_validation() {
    let processes = make_example_1();
    assert!(schedule_rr(&processes, 7).is_err());
}
